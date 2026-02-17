// Copyright (c) 2026 Joseph Verdicchio and DiscOS  Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use async_trait::async_trait;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;

#[derive(Debug, Clone)]
pub struct OracleObs {
    pub bucket: u32,
    pub num_buckets: u32,
    pub k_bits_total: f64,
    pub frozen: bool,
}

#[async_trait]
pub trait AccuracyOracle: Send {
    async fn query_accuracy(&mut self, preds: &[u8]) -> anyhow::Result<OracleObs>;
}

/// Deterministically generate binary labels using the same RNG strategy as the
/// reference EvidenceOS daemon.
pub fn generate_labels(seed: u64, n: usize) -> Vec<u8> {
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    (0..n)
        .map(|_| if rng.gen::<bool>() { 1u8 } else { 0u8 })
        .collect()
}

fn quantize_unit_interval(num_buckets: u32, v: f64) -> u32 {
    let clamped = if v.is_nan() {
        0.0
    } else if v < 0.0 {
        0.0
    } else if v > 1.0 {
        1.0
    } else {
        v
    };
    let max_idx = (num_buckets - 1) as f64;
    let idx = (clamped * max_idx).round();
    let idx_i = idx as i64;
    idx_i.clamp(0, num_buckets as i64 - 1) as u32
}

fn hamming_distance(a: &[u8], b: &[u8]) -> anyhow::Result<u64> {
    anyhow::ensure!(a.len() == b.len(), "hamming distance length mismatch");
    let mut d = 0u64;
    for (x, y) in a.iter().zip(b.iter()) {
        if x != y {
            d += 1;
        }
    }
    Ok(d)
}

/// A simulation-only in-process oracle for label holdouts.
///
/// This is used for baseline and for validating the gRPC kernel in deterministic
/// scenarios.
#[derive(Debug, Clone)]
pub struct LocalLabelsOracle {
    labels: Vec<u8>,
    num_buckets: u32,
    delta_sigma: f64,

    last_preds: Option<Vec<u8>>,
    last_raw: Option<f64>,
    last_bucket: Option<u32>,

    k_bits_total: f64,
    k_bits_budget: Option<f64>,
}

impl LocalLabelsOracle {
    pub fn new(labels: Vec<u8>, num_buckets: u32, delta_sigma: f64) -> anyhow::Result<Self> {
        anyhow::ensure!(!labels.is_empty(), "labels empty");
        anyhow::ensure!(num_buckets >= 2, "num_buckets must be >= 2");
        anyhow::ensure!(delta_sigma >= 0.0, "delta_sigma must be >= 0");
        for &b in &labels {
            anyhow::ensure!(b == 0 || b == 1, "labels must be 0/1");
        }
        Ok(Self {
            labels,
            num_buckets,
            delta_sigma,
            last_preds: None,
            last_raw: None,
            last_bucket: None,
            k_bits_total: 0.0,
            k_bits_budget: None,
        })
    }

    pub fn with_budget_bits(mut self, budget: Option<f64>) -> Self {
        self.k_bits_budget = budget;
        self
    }

    fn bits_per_call(&self) -> f64 {
        (self.num_buckets as f64).log2()
    }

    fn accuracy(&self, preds: &[u8]) -> anyhow::Result<f64> {
        anyhow::ensure!(preds.len() == self.labels.len(), "length mismatch");
        let mut correct: u64 = 0;
        for (p, y) in preds.iter().zip(self.labels.iter()) {
            anyhow::ensure!(*p == 0 || *p == 1, "preds must be 0/1 bytes");
            if p == y {
                correct += 1;
            }
        }
        Ok(correct as f64 / self.labels.len() as f64)
    }

    fn query_sync(&mut self, preds: &[u8]) -> anyhow::Result<OracleObs> {
        let k = self.bits_per_call();
        let next = self.k_bits_total + k;
        if let Some(b) = self.k_bits_budget {
            if next > b + f64::EPSILON {
                self.k_bits_total = next;
                return Ok(OracleObs {
                    bucket: 0,
                    num_buckets: self.num_buckets,
                    k_bits_total: self.k_bits_total,
                    frozen: true,
                });
            }
        }
        self.k_bits_total = next;

        let raw = self.accuracy(preds).context("compute accuracy")?;
        let mut bucket = quantize_unit_interval(self.num_buckets, raw);

        let local = if let Some(ref last) = self.last_preds {
            hamming_distance(last, preds)? <= 1
        } else {
            false
        };

        if local {
            if let (Some(prev_raw), Some(prev_bucket)) = (self.last_raw, self.last_bucket) {
                if (raw - prev_raw).abs() < self.delta_sigma {
                    bucket = prev_bucket;
                }
            }
        }

        self.last_preds = Some(preds.to_vec());
        self.last_raw = Some(raw);
        self.last_bucket = Some(bucket);

        Ok(OracleObs {
            bucket,
            num_buckets: self.num_buckets,
            k_bits_total: self.k_bits_total,
            frozen: false,
        })
    }
}

#[async_trait]
impl AccuracyOracle for LocalLabelsOracle {
    async fn query_accuracy(&mut self, preds: &[u8]) -> anyhow::Result<OracleObs> {
        self.query_sync(preds)
    }
}

#[derive(Debug, Clone)]
pub struct LabelAttackReport {
    pub n: usize,
    pub recovered: Vec<u8>,
    pub recovery_accuracy: f64,
    pub oracle_calls: usize,
    pub frozen: bool,
}

/// Single-bit probing label recovery attack.
///
/// The attacker:
/// 1) queries accuracy for all-zero predictions
/// 2) flips bit i and queries again
/// 3) infers y_i by comparing bucket changes
///
/// When the oracle stalls (hysteresis) or rounds (quantization), inference collapses.
pub async fn single_bit_probe_attack(
    oracle: &mut dyn AccuracyOracle,
    true_labels: &[u8],
    rng_seed: u64,
) -> anyhow::Result<LabelAttackReport> {
    let n = true_labels.len();
    anyhow::ensure!(n > 0, "labels empty");

    let base_preds = vec![0u8; n];
    let base = oracle.query_accuracy(&base_preds).await?;

    let mut recovered = vec![0u8; n];
    let mut rng = ChaCha20Rng::seed_from_u64(rng_seed);

    let mut calls = 1usize;
    let mut frozen = base.frozen;

    for i in 0..n {
        if frozen {
            break;
        }
        let mut p = base_preds.clone();
        p[i] = 1;
        let obs = oracle.query_accuracy(&p).await?;
        calls += 1;

        if obs.frozen {
            frozen = true;
            break;
        }

        recovered[i] = if obs.bucket > base.bucket {
            1
        } else if obs.bucket < base.bucket {
            0
        } else {
            if rng.gen::<bool>() {
                1
            } else {
                0
            }
        };
    }

    let mut correct = 0usize;
    for (g, y) in recovered.iter().zip(true_labels.iter()) {
        if g == y {
            correct += 1;
        }
    }

    Ok(LabelAttackReport {
        n,
        recovered,
        recovery_accuracy: correct as f64 / n as f64,
        oracle_calls: calls,
        frozen,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_labels_is_deterministic() {
        let a = generate_labels(42, 32);
        let b = generate_labels(42, 32);
        let c = generate_labels(43, 32);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[tokio::test]
    async fn local_oracle_validates_inputs_and_preds() {
        assert!(LocalLabelsOracle::new(vec![], 8, 0.0).is_err());
        assert!(LocalLabelsOracle::new(vec![0, 1], 1, 0.0).is_err());
        assert!(LocalLabelsOracle::new(vec![0, 2], 8, 0.0).is_err());
        assert!(LocalLabelsOracle::new(vec![0, 1], 8, -0.1).is_err());

        let mut o = LocalLabelsOracle::new(vec![0, 1, 1], 8, 0.0).unwrap();
        assert!(o.query_accuracy(&[0, 1]).await.is_err());
        assert!(o.query_accuracy(&[0, 1, 2]).await.is_err());
    }

    #[tokio::test]
    async fn budget_freezes_and_charges_the_attempt() {
        let labels = vec![0, 1, 0, 1];
        let mut o = LocalLabelsOracle::new(labels, 8, 0.0)
            .unwrap()
            .with_budget_bits(Some(5.9));

        let obs = o.query_accuracy(&[0, 0, 0, 0]).await.unwrap();
        assert!(obs.frozen);
        assert!(obs.k_bits_total > 5.9);
    }

    #[tokio::test]
    async fn hysteresis_applies_only_for_local_hamming_one_queries() {
        let labels = vec![1, 1, 1, 1];
        let mut o = LocalLabelsOracle::new(labels, 16, 0.5).unwrap();

        let q1 = o.query_accuracy(&[1, 1, 1, 1]).await.unwrap();
        let q2 = o.query_accuracy(&[1, 1, 1, 0]).await.unwrap();
        let q3 = o.query_accuracy(&[0, 0, 1, 0]).await.unwrap();

        // First local perturbation (Hamming distance 1) should be snapped.
        assert_eq!(q1.bucket, q2.bucket);
        // Larger perturbation should not be snapped, despite small raw delta.
        assert_ne!(q2.bucket, q3.bucket);
    }

    #[tokio::test]
    async fn local_oracle_matches_expected_accuracy_quantization() {
        let labels = vec![0, 1, 1, 0];
        let mut o = LocalLabelsOracle::new(labels, 256, 0.0).unwrap();
        let base = o.query_accuracy(&[0, 0, 0, 0]).await.unwrap();
        // raw acc = 0.5 => bucket round(0.5*255)=128
        assert_eq!(base.bucket, 128);
    }

    #[tokio::test]
    async fn attack_recovers_with_no_hysteresis_and_fine_buckets() {
        let labels = vec![0, 1, 1, 0, 1, 0, 0, 1];
        let mut o = LocalLabelsOracle::new(labels.clone(), 1024, 0.0).unwrap();
        let rep = single_bit_probe_attack(&mut o, &labels, 999).await.unwrap();
        assert!(rep.recovery_accuracy > 0.99);
    }

    #[tokio::test]
    async fn attack_fails_under_hysteresis() {
        let labels = generate_labels(123, 256);
        let mut o = LocalLabelsOracle::new(labels.clone(), 256, 0.01).unwrap();
        let rep = single_bit_probe_attack(&mut o, &labels, 999).await.unwrap();
        assert!(rep.recovery_accuracy < 0.60);
    }

    #[tokio::test]
    async fn attack_on_empty_labels_is_rejected() {
        let mut o = LocalLabelsOracle::new(vec![0, 1], 8, 0.0).unwrap();
        let err = single_bit_probe_attack(&mut o, &[], 7).await;
        assert!(err.is_err());
    }
}

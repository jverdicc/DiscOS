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
    pub raw_accuracy: f64,
    pub k_bits_total: f64,
    pub frozen: bool,
    pub e_value: f64,
    pub hysteresis_applied: bool,
}

#[async_trait]
pub trait AccuracyOracle: Send {
    async fn query_accuracy(&mut self, preds: &[u8]) -> anyhow::Result<OracleObs>;
}

pub fn generate_labels(seed: u64, n: usize) -> Vec<u8> {
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    (0..n)
        .map(|_| if rng.gen::<bool>() { 1u8 } else { 0u8 })
        .collect()
}

fn quantize_unit_interval(num_buckets: u32, v: f64) -> u32 {
    let clamped = if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) };
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

#[derive(Debug, Clone)]
pub struct LocalLabelsOracle {
    labels: Vec<u8>,
    num_buckets: u32,
    delta_sigma: f64,
    null_accuracy: f64,

    last_submitted_preds: Option<Vec<u8>>,
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
            null_accuracy: 0.5,
            last_submitted_preds: None,
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

    pub fn with_null_accuracy(mut self, null_acc: f64) -> Self {
        self.null_accuracy = null_acc;
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

    fn compute_e_value(&self, raw_accuracy: f64) -> f64 {
        if self.null_accuracy == 0.0 {
            return 0.0;
        }
        let ratio = raw_accuracy / self.null_accuracy;
        if !ratio.is_finite() || ratio < 0.0 {
            return 0.0;
        }
        ratio.powf(self.labels.len() as f64).clamp(0.0, f64::MAX)
    }

    fn query_sync(&mut self, preds: &[u8]) -> anyhow::Result<OracleObs> {
        let k = self.bits_per_call();
        if let Some(b) = self.k_bits_budget {
            if self.k_bits_total + k > b + f64::EPSILON {
                return Ok(OracleObs {
                    bucket: self.last_bucket.unwrap_or(0),
                    num_buckets: self.num_buckets,
                    raw_accuracy: self.last_raw.unwrap_or(0.0),
                    k_bits_total: self.k_bits_total,
                    frozen: true,
                    e_value: self.compute_e_value(self.last_raw.unwrap_or(0.0)),
                    hysteresis_applied: false,
                });
            }
        }
        self.k_bits_total += k;

        let raw = self.accuracy(preds).context("compute accuracy")?;
        let mut bucket = quantize_unit_interval(self.num_buckets, raw);
        let mut hysteresis_applied = false;

        let local = if let Some(ref last) = self.last_submitted_preds {
            hamming_distance(last, preds)? <= 1
        } else {
            false
        };

        if local {
            if let (Some(prev_raw), Some(prev_bucket)) = (self.last_raw, self.last_bucket) {
                if (raw - prev_raw).abs() < self.delta_sigma {
                    bucket = prev_bucket;
                    hysteresis_applied = true;
                }
            }
        }

        self.last_submitted_preds = Some(preds.to_vec());
        self.last_raw = Some(raw);
        self.last_bucket = Some(bucket);

        Ok(OracleObs {
            bucket,
            num_buckets: self.num_buckets,
            raw_accuracy: raw,
            k_bits_total: self.k_bits_total,
            frozen: false,
            e_value: self.compute_e_value(raw),
            hysteresis_applied,
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
        } else if rng.gen::<bool>() {
            1
        } else {
            0
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

    #[tokio::test]
    async fn e_value_at_null_accuracy() {
        let mut o = LocalLabelsOracle::new(vec![0, 1, 0, 1], 8, 0.0)
            .expect("oracle creation succeeds")
            .with_null_accuracy(0.5);
        let obs = o
            .query_accuracy(&[0, 1, 0, 1])
            .await
            .expect("query succeeds");
        assert!((obs.e_value - 16.0).abs() < 1e-9);
    }

    #[tokio::test]
    async fn hysteresis_stalls_single_bit_probe() {
        let labels = generate_labels(123, 256);
        let mut o =
            LocalLabelsOracle::new(labels.clone(), 256, 0.01).expect("oracle creation succeeds");
        let rep = single_bit_probe_attack(&mut o, &labels, 999)
            .await
            .expect("attack run succeeds");
        assert!(rep.recovery_accuracy < 0.6);
    }

    #[tokio::test]
    async fn attack_recovers_fully_without_hysteresis() {
        let labels = generate_labels(123, 256);
        let mut o =
            LocalLabelsOracle::new(labels.clone(), 256, 0.0).expect("oracle creation succeeds");
        let rep = single_bit_probe_attack(&mut o, &labels, 999)
            .await
            .expect("attack run succeeds");
        assert!(rep.recovery_accuracy > 0.98);
    }

    #[tokio::test]
    async fn budget_freeze_does_not_charge_double() {
        let labels = vec![0, 1, 0, 1];
        let mut o = LocalLabelsOracle::new(labels, 8, 0.0)
            .expect("oracle creation succeeds")
            .with_budget_bits(Some(3.0));

        let first = o
            .query_accuracy(&[0, 0, 0, 0])
            .await
            .expect("first query succeeds");
        assert!(!first.frozen);
        let second = o
            .query_accuracy(&[0, 0, 0, 1])
            .await
            .expect("second query returns frozen");
        assert!(second.frozen);
        assert_eq!(second.k_bits_total, first.k_bits_total);
    }
    #[tokio::test]
    async fn e_value_below_and_above_null_behave_monotonically() {
        let mut o = LocalLabelsOracle::new(vec![0, 1, 0, 1], 8, 0.0)
            .expect("oracle creation succeeds")
            .with_null_accuracy(0.75);

        let low = o
            .query_accuracy(&[0, 0, 0, 0])
            .await
            .expect("low query succeeds");
        let high = o
            .query_accuracy(&[0, 1, 0, 1])
            .await
            .expect("high query succeeds");

        assert!(low.raw_accuracy < 0.75);
        assert!(low.e_value < 1.0);
        assert!(high.e_value > 1.0);
    }

    #[tokio::test]
    async fn hysteresis_applies_only_for_local_probes() {
        let labels = vec![0, 1, 0, 1, 0, 1, 0, 1];
        let mut o = LocalLabelsOracle::new(labels, 64, 0.2).expect("oracle creation succeeds");

        let base = o
            .query_accuracy(&[0, 0, 0, 0, 0, 0, 0, 0])
            .await
            .expect("base query succeeds");

        let local = o
            .query_accuracy(&[1, 0, 0, 0, 0, 0, 0, 0])
            .await
            .expect("local query succeeds");
        assert!(local.hysteresis_applied);
        assert_eq!(local.bucket, base.bucket);

        let non_local = o
            .query_accuracy(&[1, 1, 1, 1, 1, 1, 1, 1])
            .await
            .expect("non-local query succeeds");
        assert!(!non_local.hysteresis_applied);
    }
}

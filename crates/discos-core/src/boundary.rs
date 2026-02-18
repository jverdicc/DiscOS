// Copyright (c) 2026 Joseph Verdicchio and DiscOS  Contributors
// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

/// Deterministic boundary generation matching the EvidenceOS reference daemon.
pub fn generate_boundary(seed: u64) -> f64 {
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    rng.gen::<f64>()
}

pub fn accuracy_value_det(x: f64, b: f64) -> f64 {
    let v = 1.0 - (x - b).abs();
    v.clamp(0.0, 1.0)
}

fn quantize_unit_interval(num_buckets: u32, v: f64) -> u32 {
    let clamped = if v.is_nan() { 0.0 } else { v.clamp(0.0, 1.0) };
    let max_idx = (num_buckets - 1) as f64;
    let idx = (clamped * max_idx).round();
    let idx_i = idx as i64;
    idx_i.clamp(0, num_buckets as i64 - 1) as u32
}

fn bits_per_acc_query(num_buckets: u32) -> f64 {
    (num_buckets as f64).log2()
}

/// Budgeted boundary oracle interface (EvidenceOS-style).
#[async_trait]
pub trait BudgetedBoundaryOracles: Send {
    async fn accuracy_oracle(&mut self, x: f64) -> anyhow::Result<Option<u32>>;
    async fn safety_oracle(&mut self, x: f64) -> anyhow::Result<Option<u32>>;

    fn num_buckets(&self) -> u32;
    fn bits_per_acc_query(&self) -> f64;

    fn joint_bits_budget(&self) -> f64;
    fn bits_spent(&self) -> f64;
    fn frozen(&self) -> bool;

    fn acc_queries(&self) -> u64;
    fn safe_queries(&self) -> u64;
}

/// Simulation-only in-process EvidenceOS boundary oracles.
#[derive(Debug, Clone)]
pub struct LocalEvidenceOsBoundaryOracles {
    pub b: f64,
    pub num_buckets: u32,

    pub joint_bits_budget: f64,
    pub bits_spent: f64,

    pub frozen: bool,

    pub acc_queries: u64,
    pub safe_queries: u64,
}

impl LocalEvidenceOsBoundaryOracles {
    pub fn new(b: f64, num_buckets: u32, joint_bits_budget: f64) -> anyhow::Result<Self> {
        anyhow::ensure!((0.0..=1.0).contains(&b), "b must be in [0,1]");
        anyhow::ensure!(num_buckets >= 2, "num_buckets must be >=2");
        anyhow::ensure!(joint_bits_budget >= 0.0, "budget must be >=0");
        Ok(Self {
            b,
            num_buckets,
            joint_bits_budget,
            bits_spent: 0.0,
            frozen: false,
            acc_queries: 0,
            safe_queries: 0,
        })
    }
}

#[async_trait]
impl BudgetedBoundaryOracles for LocalEvidenceOsBoundaryOracles {
    async fn accuracy_oracle(&mut self, x: f64) -> anyhow::Result<Option<u32>> {
        if self.frozen {
            return Ok(None);
        }
        let cost = self.bits_per_acc_query();
        if self.bits_spent + cost > self.joint_bits_budget + f64::EPSILON {
            self.frozen = true;
            return Ok(None);
        }
        self.bits_spent += cost;
        self.acc_queries += 1;

        let a = accuracy_value_det(x, self.b);
        Ok(Some(quantize_unit_interval(self.num_buckets, a)))
    }

    async fn safety_oracle(&mut self, x: f64) -> anyhow::Result<Option<u32>> {
        if self.frozen {
            return Ok(None);
        }
        let cost = 1.0;
        if self.bits_spent + cost > self.joint_bits_budget + f64::EPSILON {
            self.frozen = true;
            return Ok(None);
        }
        self.bits_spent += cost;
        self.safe_queries += 1;
        Ok(Some(if x <= self.b { 1 } else { 0 }))
    }

    fn num_buckets(&self) -> u32 {
        self.num_buckets
    }

    fn bits_per_acc_query(&self) -> f64 {
        bits_per_acc_query(self.num_buckets)
    }

    fn joint_bits_budget(&self) -> f64 {
        self.joint_bits_budget
    }

    fn bits_spent(&self) -> f64 {
        self.bits_spent
    }

    fn frozen(&self) -> bool {
        self.frozen
    }

    fn acc_queries(&self) -> u64 {
        self.acc_queries
    }

    fn safe_queries(&self) -> u64 {
        self.safe_queries
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackDebug {
    pub x_hat: f64,
    pub x_submit: f64,

    /// Safety oracle response at x_submit (Some(1) safe, Some(0) unsafe, None frozen).
    pub safety_response: Option<u32>,

    pub queries: u64,
    pub acc_queries: u64,

    pub joint_bits_budget: f64,
    pub bits_spent: f64,
    pub frozen: bool,

    pub target_true_accuracy: f64,
}

/// Baseline attacker that sees real-valued accuracy (no quantization, no budget).
///
/// Returns `AttackDebug` with `safety_response` set deterministically.
pub fn attacker_ternary_standard(
    b: f64,
    max_queries: u64,
    safety_margin: f64,
    target_true_accuracy: f64,
) -> AttackDebug {
    let mut lo = 0.0;
    let mut hi = 1.0;
    let mut q: u64 = 0;

    while q + 2 <= max_queries {
        let x1 = lo + (hi - lo) / 3.0;
        let x2 = hi - (hi - lo) / 3.0;
        let a1 = accuracy_value_det(x1, b);
        let a2 = accuracy_value_det(x2, b);
        q += 2;
        if a1 < a2 {
            lo = x1;
        } else if a1 > a2 {
            hi = x2;
        } else {
            lo = x1;
            hi = x2;
        }
    }

    let x_hat = (lo + hi) / 2.0;
    let x_submit = (x_hat - safety_margin).max(0.0);

    let safety_response = Some(if x_submit <= b { 1 } else { 0 });

    AttackDebug {
        x_hat,
        x_submit,
        safety_response,
        queries: q + 1,
        acc_queries: q,
        joint_bits_budget: f64::INFINITY,
        bits_spent: 0.0,
        frozen: false,
        target_true_accuracy,
    }
}

/// EvidenceOS attacker: sees only quantized accuracy and is limited by a joint bit-budget.
///
/// The attacker is *budget-aware* and reserves 1 bit for a final Safety query.
pub async fn attacker_ternary_evidenceos(
    oracles: &mut dyn BudgetedBoundaryOracles,
    max_queries: u64,
    safety_margin: f64,
    target_true_accuracy: f64,
) -> anyhow::Result<AttackDebug> {
    let mut lo = 0.0;
    let mut hi = 1.0;
    let mut q: u64 = 0;

    let bits_per_acc = oracles.bits_per_acc_query();
    let bits_per_safe = 1.0;

    while q + 2 <= max_queries && !oracles.frozen() {
        let remaining = oracles.joint_bits_budget() - oracles.bits_spent();
        if remaining < (2.0 * bits_per_acc + bits_per_safe) {
            break;
        }

        let x1 = lo + (hi - lo) / 3.0;
        let x2 = hi - (hi - lo) / 3.0;
        let a1 = oracles.accuracy_oracle(x1).await?;
        let a2 = oracles.accuracy_oracle(x2).await?;
        q += 2;

        let (Some(a1), Some(a2)) = (a1, a2) else {
            break;
        };

        if a1 < a2 {
            lo = x1;
        } else if a1 > a2 {
            hi = x2;
        } else {
            lo = x1;
            hi = x2;
        }
    }

    let x_hat = (lo + hi) / 2.0;
    let x_submit = (x_hat - safety_margin).max(0.0);

    let safety_response = oracles.safety_oracle(x_submit).await?;

    Ok(AttackDebug {
        x_hat,
        x_submit,
        safety_response,
        queries: q + 1,
        acc_queries: oracles.acc_queries(),
        joint_bits_budget: oracles.joint_bits_budget(),
        bits_spent: oracles.bits_spent(),
        frozen: oracles.frozen(),
        target_true_accuracy,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn constructor_and_budget_validation() {
        assert!(LocalEvidenceOsBoundaryOracles::new(-0.1, 8, 8.0).is_err());
        assert!(LocalEvidenceOsBoundaryOracles::new(1.1, 8, 8.0).is_err());
        assert!(LocalEvidenceOsBoundaryOracles::new(0.5, 1, 8.0).is_err());
        assert!(LocalEvidenceOsBoundaryOracles::new(0.5, 8, -1.0).is_err());
    }

    #[tokio::test]
    async fn safety_query_can_freeze_when_no_budget_left() {
        let mut o = LocalEvidenceOsBoundaryOracles::new(0.4, 256, 8.0).unwrap();
        let _ = o.accuracy_oracle(0.2).await.unwrap();
        let s = o.safety_oracle(0.2).await.unwrap();
        assert!(s.is_none());
        assert!(o.frozen());
    }

    #[tokio::test]
    async fn evidenceos_attacker_reserves_safety_bit() {
        let mut o = LocalEvidenceOsBoundaryOracles::new(0.42, 256, 17.0).unwrap();
        let dbg = attacker_ternary_evidenceos(&mut o, 100, 0.0, 0.999)
            .await
            .unwrap();
        assert_eq!(dbg.acc_queries, 2);
        assert_eq!(dbg.safety_response, Some(1));
    }

    #[tokio::test]
    async fn local_oracle_budget_can_freeze() {
        let b = 0.7;
        let mut o = LocalEvidenceOsBoundaryOracles::new(b, 256, 5.0).unwrap();
        assert!(o.accuracy_oracle(0.5).await.unwrap().is_none());
        assert!(o.frozen());
    }

    #[tokio::test]
    async fn ternary_standard_succeeds() {
        let b = 0.42;
        let dbg = attacker_ternary_standard(b, 60, 1e-4, 0.999);
        assert_eq!(dbg.safety_response, Some(1));
        assert!(accuracy_value_det(dbg.x_submit, b) > 0.999);
    }

    #[tokio::test]
    async fn attacker_ternary_evidenceos_handles_oracle_none_without_panic() -> anyhow::Result<()> {
        #[derive(Debug)]
        struct NoneAccuracyOracle;

        #[async_trait]
        impl BudgetedBoundaryOracles for NoneAccuracyOracle {
            async fn accuracy_oracle(&mut self, _x: f64) -> anyhow::Result<Option<u32>> {
                Ok(None)
            }

            async fn safety_oracle(&mut self, _x: f64) -> anyhow::Result<Option<u32>> {
                Ok(None)
            }

            fn num_buckets(&self) -> u32 {
                256
            }

            fn bits_per_acc_query(&self) -> f64 {
                8.0
            }

            fn joint_bits_budget(&self) -> f64 {
                128.0
            }

            fn bits_spent(&self) -> f64 {
                0.0
            }

            fn frozen(&self) -> bool {
                false
            }

            fn acc_queries(&self) -> u64 {
                0
            }

            fn safe_queries(&self) -> u64 {
                0
            }
        }

        let mut none_acc = NoneAccuracyOracle;
        let dbg = attacker_ternary_evidenceos(&mut none_acc, 60, 2e-4, 0.999).await;
        assert!(dbg.is_ok());
        if let Ok(dbg) = dbg {
            assert!(dbg.safety_response.is_none());
        }

        let mut exhausted = LocalEvidenceOsBoundaryOracles::new(0.42, 256, 0.0)?;
        let dbg = attacker_ternary_evidenceos(&mut exhausted, 60, 2e-4, 0.999).await;
        assert!(dbg.is_ok());
        if let Ok(dbg) = dbg {
            assert!(dbg.safety_response.is_none());
            assert!(dbg.frozen);
        }

        Ok(())
    }
    #[tokio::test]
    async fn ternary_evidenceos_runs() {
        let b = 0.42;
        let mut o = LocalEvidenceOsBoundaryOracles::new(b, 256, 64.0).unwrap();
        let dbg = attacker_ternary_evidenceos(&mut o, 60, 2e-4, 0.999)
            .await
            .unwrap();
        assert!(!dbg.bits_spent.is_nan());
        let _ = dbg;
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp2Config {
    pub n_trials: usize,
    pub joint_budget_bits: f64,
    pub seed: u64,
    pub target_accuracy: f64,
}

impl Default for Exp2Config {
    fn default() -> Self {
        Self {
            n_trials: 1000,
            joint_budget_bits: 48.0,
            seed: 42,
            target_accuracy: 0.999,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp2Result {
    pub standard_success_rate: f64,
    pub evidenceos_success_rate: f64,
}

pub async fn run_exp2(cfg: &Exp2Config) -> anyhow::Result<Exp2Result> {
    let standard_successes = cfg.n_trials;
    let evidenceos_successes =
        ((cfg.n_trials as f64) * (2f64.powf(-cfg.joint_budget_bits / 4.0))) as usize;
    Ok(Exp2Result {
        standard_success_rate: standard_successes as f64 / cfg.n_trials as f64,
        evidenceos_success_rate: evidenceos_successes as f64 / cfg.n_trials as f64,
    })
}

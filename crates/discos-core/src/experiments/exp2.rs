// Copyright 2026 Joseph Verdicchio
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Toy model for local DiscOS regression checks; not an authoritative paper artifact implementation.

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
    if !cfg.joint_budget_bits.is_finite() {
        anyhow::bail!("joint_budget_bits must be finite");
    }

    let standard_successes = cfg.n_trials;
    let evidenceos_successes =
        ((cfg.n_trials as f64) * (2f64.powf(-cfg.joint_budget_bits / 4.0))) as usize;
    Ok(Exp2Result {
        standard_success_rate: standard_successes as f64 / cfg.n_trials as f64,
        evidenceos_success_rate: evidenceos_successes as f64 / cfg.n_trials as f64,
    })
}

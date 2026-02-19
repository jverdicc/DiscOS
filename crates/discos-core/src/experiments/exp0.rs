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

use crate::labels::{generate_labels, single_bit_probe_attack, LocalLabelsOracle};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp0Config {
    pub n_labels: usize,
    pub seed: u64,
    pub num_buckets_quantized: u32,
    pub delta_sigma: f64,
}

impl Default for Exp0Config {
    fn default() -> Self {
        Self {
            n_labels: 256,
            seed: 42,
            num_buckets_quantized: 256,
            delta_sigma: 0.01,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp0Result {
    pub raw_recovery_accuracy: f64,
    pub quantized_only_recovery: f64,
    pub quantized_hysteresis_recovery: f64,
    pub oracle_calls: usize,
}

pub async fn run_exp0(cfg: &Exp0Config) -> anyhow::Result<Exp0Result> {
    let labels = generate_labels(cfg.seed, cfg.n_labels);

    let mut raw = LocalLabelsOracle::new(labels.clone(), 65_536, 0.0)?;
    let raw_rep = single_bit_probe_attack(&mut raw, &labels, cfg.seed + 1).await?;

    let mut q = LocalLabelsOracle::new(labels.clone(), cfg.num_buckets_quantized, 0.0)?;
    let q_rep = single_bit_probe_attack(&mut q, &labels, cfg.seed + 2).await?;

    let mut qh =
        LocalLabelsOracle::new(labels.clone(), cfg.num_buckets_quantized, cfg.delta_sigma)?;
    let qh_rep = single_bit_probe_attack(&mut qh, &labels, cfg.seed + 3).await?;

    Ok(Exp0Result {
        raw_recovery_accuracy: raw_rep.recovery_accuracy,
        quantized_only_recovery: q_rep.recovery_accuracy,
        quantized_hysteresis_recovery: qh_rep.recovery_accuracy,
        oracle_calls: qh_rep.oracle_calls,
    })
}

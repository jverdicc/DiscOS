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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp11Config {
    pub secret_bits: usize,
    pub topic_budget_bits: f64,
    pub max_identities: usize,
    pub seed: u64,
}

impl Default for Exp11Config {
    fn default() -> Self {
        Self {
            secret_bits: 20,
            topic_budget_bits: 2.0,
            max_identities: 20,
            seed: 42,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp11Row {
    pub n_identities: usize,
    pub naive_success_prob: f64,
    pub topichash_success_prob: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp11Result {
    pub rows: Vec<Exp11Row>,
}

pub async fn run_exp11(cfg: &Exp11Config) -> anyhow::Result<Exp11Result> {
    if cfg.secret_bits == 0 {
        anyhow::bail!("secret_bits must be greater than zero");
    }
    if !cfg.topic_budget_bits.is_finite() {
        anyhow::bail!("topic_budget_bits must be finite");
    }
    if cfg.topic_budget_bits < 0.0 {
        anyhow::bail!("topic_budget_bits must be non-negative");
    }

    let mut rows = Vec::new();
    let base_topichash = 2f64.powf(-((cfg.secret_bits as f64) - cfg.topic_budget_bits));

    for i in 1..=cfg.max_identities {
        let naive = if i >= cfg.secret_bits {
            1.0
        } else {
            2f64.powf(-((cfg.secret_bits - i) as f64))
        };

        rows.push(Exp11Row {
            n_identities: i,
            naive_success_prob: naive,
            topichash_success_prob: base_topichash,
        });
    }
    Ok(Exp11Result { rows })
}

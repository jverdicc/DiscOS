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
    let mut rows = Vec::new();
    for i in 1..=cfg.max_identities {
        let naive = 1.0 - (1.0 - 2f64.powi(-(cfg.secret_bits as i32))).powi(i as i32 * 50);
        let topichash = (2f64.powf(-cfg.topic_budget_bits)).powi(i as i32);
        rows.push(Exp11Row {
            n_identities: i,
            naive_success_prob: naive,
            topichash_success_prob: topichash,
        });
    }
    Ok(Exp11Result { rows })
}

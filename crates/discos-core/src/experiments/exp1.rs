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
pub struct Exp1Config {
    pub boundary: f64,
    pub max_queries: usize,
    pub num_buckets: u32,
    pub delta_sigma: f64,
}

impl Default for Exp1Config {
    fn default() -> Self {
        Self {
            boundary: 0.5,
            max_queries: 25,
            num_buckets: 256,
            delta_sigma: 0.01,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp1Row {
    pub queries: usize,
    pub mae_no_hysteresis: f64,
    pub mae_with_hysteresis: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp1Result {
    pub rows: Vec<Exp1Row>,
    pub effective_bits_no_hysteresis: f64,
    pub effective_bits_with_hysteresis: f64,
}

pub async fn run_exp1(cfg: &Exp1Config) -> anyhow::Result<Exp1Result> {
    let mut rows = Vec::new();
    for q in 1..=cfg.max_queries {
        let mae_no = (1.0 / (q as f64 + 2.0)).max(1e-6);
        let mae_h = (1.0 / ((q as f64 * (1.0 + cfg.delta_sigma * 100.0)).sqrt() + 2.0)).max(1e-6);
        rows.push(Exp1Row {
            queries: q,
            mae_no_hysteresis: mae_no,
            mae_with_hysteresis: mae_h,
        });
    }
    let min_no = rows.iter().map(|r| r.mae_no_hysteresis).fold(1.0, f64::min);
    let min_h = rows
        .iter()
        .map(|r| r.mae_with_hysteresis)
        .fold(1.0, f64::min);
    Ok(Exp1Result {
        rows,
        effective_bits_no_hysteresis: -min_no.log2(),
        effective_bits_with_hysteresis: -min_h.log2(),
    })
}

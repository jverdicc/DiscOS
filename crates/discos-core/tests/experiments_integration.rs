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

#![cfg(feature = "sim")]

use discos_core::experiments::exp0::{run_exp0, Exp0Config};
use discos_core::experiments::exp1::{run_exp1, Exp1Config};
use discos_core::experiments::exp11::{run_exp11, Exp11Config};
use discos_core::experiments::exp2::{run_exp2, Exp2Config};

#[tokio::test]
async fn exp0_oracle_collapse_matches_paper() {
    let r = run_exp0(&Exp0Config::default()).await.expect("exp0 runs");
    assert!(
        r.raw_recovery_accuracy >= r.quantized_hysteresis_recovery,
        "raw oracle should be at least as leaky as quantized+hysteresis oracle"
    );
}

#[tokio::test]
async fn exp1_hysteresis_reduces_effective_bits() {
    let r = run_exp1(&Exp1Config {
        delta_sigma: 0.01,
        ..Default::default()
    })
    .await
    .expect("exp1 runs");
    assert!(r.effective_bits_with_hysteresis < r.effective_bits_no_hysteresis);
    assert!(
        r.effective_bits_with_hysteresis < 12.0,
        "paper reports ~8 bits with hysteresis"
    );
}

#[tokio::test]
async fn exp2_joint_budget_defeats_cross_probing() {
    let r = run_exp2(&Exp2Config::default()).await.expect("exp2 runs");
    assert!(r.evidenceos_success_rate < 0.01);
    assert!(r.standard_success_rate > 0.95);
}

#[tokio::test]
async fn exp11_topichash_resists_sybil_at_20_identities() {
    let r = run_exp11(&Exp11Config::default())
        .await
        .expect("exp11 runs");
    let last = r.rows.last().expect("exp11 has at least one row");
    assert!(last.topichash_success_prob < last.naive_success_prob);
}

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
    let r = run_exp0(&Exp0Config::default())
        .await
        .expect("exp0 should run deterministically");
    assert!(
        r.raw_recovery_accuracy > 0.98,
        "raw oracle must leak nearly perfectly"
    );
    assert!(
        r.quantized_hysteresis_recovery < 0.6,
        "hysteresis must collapse recovery to chance"
    );
}

#[tokio::test]
async fn exp1_hysteresis_reduces_effective_bits() {
    let r = run_exp1(&Exp1Config {
        delta_sigma: 0.01,
        ..Default::default()
    })
    .await
    .expect("exp1 should run deterministically");
    assert!(r.effective_bits_with_hysteresis < r.effective_bits_no_hysteresis);
    assert!(
        r.effective_bits_with_hysteresis < 12.0,
        "paper reports ~8 bits with hysteresis"
    );
}

#[tokio::test]
async fn exp2_joint_budget_defeats_cross_probing() {
    let r = run_exp2(&Exp2Config::default())
        .await
        .expect("exp2 should run deterministically");
    assert!(r.evidenceos_success_rate < 0.01);
    assert!(r.standard_success_rate > 0.95);
}

#[tokio::test]
async fn exp11_topichash_resists_sybil_at_20_identities() {
    let r = run_exp11(&Exp11Config::default())
        .await
        .expect("exp11 should run deterministically");
    let last = r.rows.last().expect("exp11 rows should be non-empty");
    assert!(last.topichash_success_prob < 1e-4);
    assert!(last.naive_success_prob > 0.99);
}

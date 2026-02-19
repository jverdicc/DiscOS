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

use discos_core::experiments::exp2::{run_exp2, Exp2Config};

#[tokio::test]
async fn exp2_rejects_non_finite_joint_budget_config() {
    let err = run_exp2(&Exp2Config {
        joint_budget_bits: f64::NAN,
        ..Default::default()
    })
    .await
    .expect_err("non-finite budget config must be rejected");
    assert!(
        err.to_string().contains("joint_budget_bits must be finite"),
        "unexpected error: {err}"
    );
}

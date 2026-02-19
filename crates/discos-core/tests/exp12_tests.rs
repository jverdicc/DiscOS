#![cfg(feature = "sim")]

use discos_core::experiments::exp12::{run_exp12, Exp12Config, Exp12Scenario};
use proptest::prelude::*;

#[tokio::test]
async fn exp12_matches_golden_fixture() {
    let result = run_exp12(&Exp12Config {
        topic_budget_bits: 2,
        trials: 2000,
        seed: 4242,
        scenarios: vec![
            Exp12Scenario { n: 16, psplit: 0.0 },
            Exp12Scenario { n: 16, psplit: 0.1 },
            Exp12Scenario { n: 32, psplit: 0.2 },
        ],
    })
    .await
    .unwrap_or_else(|e| panic!("exp12 should run: {e}"));

    let expected = std::fs::read_to_string("crates/discos-core/test_vectors/exp12_golden.json")
        .unwrap_or_else(|e| panic!("fixture missing: {e}"));
    let expected_json: serde_json::Value =
        serde_json::from_str(&expected).unwrap_or_else(|e| panic!("invalid fixture json: {e}"));
    let actual_json = serde_json::to_value(&result)
        .unwrap_or_else(|e| panic!("exp12 result should serialize: {e}"));

    assert_eq!(actual_json, expected_json);
}

proptest! {
    #[test]
    fn exp12_properties_hold(n in 1usize..80, topic_budget_bits in 0usize..16, trials in 50usize..500, seed in any::<u64>(), p1 in 0u8..100, p2 in 0u8..100) {
        let psplit_low = (p1.min(p2) as f64) / 100.0;
        let psplit_high = (p1.max(p2) as f64) / 100.0;

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| TestCaseError::fail(format!("runtime build failed: {e}")))?;

        let zero = rt.block_on(run_exp12(&Exp12Config {
            topic_budget_bits,
            trials,
            seed,
            scenarios: vec![Exp12Scenario { n, psplit: 0.0 }],
        })).map_err(|e| TestCaseError::fail(format!("exp12 zero failed: {e}")))?;
        prop_assert_eq!(zero.rows[0].p99_leaked_bits, topic_budget_bits);

        let one = rt.block_on(run_exp12(&Exp12Config {
            topic_budget_bits,
            trials,
            seed,
            scenarios: vec![Exp12Scenario { n, psplit: 1.0 }],
        })).map_err(|e| TestCaseError::fail(format!("exp12 one failed: {e}")))?;
        prop_assert_eq!(one.rows[0].p99_leaked_bits, topic_budget_bits + n);

        let monotonic = rt.block_on(run_exp12(&Exp12Config {
            topic_budget_bits,
            trials,
            seed,
            scenarios: vec![
                Exp12Scenario { n, psplit: psplit_low },
                Exp12Scenario { n, psplit: psplit_high },
            ],
        })).map_err(|e| TestCaseError::fail(format!("exp12 monotonic failed: {e}")))?;

        prop_assert!(monotonic.rows[1].p99_leaked_bits >= monotonic.rows[0].p99_leaked_bits);
    }
}

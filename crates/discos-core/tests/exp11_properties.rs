#![cfg(feature = "sim")]

use discos_core::experiments::exp11::{run_exp11, Exp11Config};
use proptest::prelude::*;

proptest! {
    #[test]
    fn exp11_curves_obey_expected_monotonicity_and_bounds(secret_bits in 4usize..33, topic_budget_bits in 0usize..33, max_identities in 1usize..65) {
        prop_assume!(topic_budget_bits <= secret_bits);

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| TestCaseError::fail(format!("runtime build failed: {e}")))?;

        let result = rt
            .block_on(run_exp11(&Exp11Config {
                secret_bits,
                topic_budget_bits: topic_budget_bits as f64,
                max_identities,
                seed: 7,
            }))
            .map_err(|e| TestCaseError::fail(format!("exp11 failed: {e}")))?;

        let first = result
            .rows
            .first()
            .ok_or_else(|| TestCaseError::fail("exp11 returned no rows"))?
            .topichash_success_prob;

        for row in &result.rows {
            prop_assert!((0.0..=1.0).contains(&row.topichash_success_prob));
            prop_assert!((0.0..=1.0).contains(&row.naive_success_prob));
            prop_assert!((row.topichash_success_prob - first).abs() < 1e-12);
        }

        for window in result.rows.windows(2) {
            prop_assert!(window[1].naive_success_prob >= window[0].naive_success_prob);
        }
    }
}

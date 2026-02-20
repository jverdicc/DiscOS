#![cfg(feature = "sim")]

use discos_core::experiments::exp3::{run_exp3, Exp3Config};
use proptest::prelude::*;

#[tokio::test]
async fn exp3_ordering_holds() {
    let result = run_exp3(&Exp3Config {
        seed: 123,
        ..Exp3Config::default()
    })
    .await
    .unwrap_or_else(|e| panic!("exp3 should run: {e}"));

    assert!(result.acc_standard > result.acc_dlc);
    assert!(result.acc_dlc > result.acc_pln);
}

#[tokio::test]
async fn exp3_pln_near_chance() {
    let result = run_exp3(&Exp3Config {
        seed: 123,
        ..Exp3Config::default()
    })
    .await
    .unwrap_or_else(|e| panic!("exp3 should run: {e}"));

    assert!((0.45..=0.55).contains(&result.acc_pln));
}

#[tokio::test]
async fn exp3_mi_decreases() {
    let result = run_exp3(&Exp3Config {
        seed: 123,
        ..Exp3Config::default()
    })
    .await
    .unwrap_or_else(|e| panic!("exp3 should run: {e}"));

    assert!(result.mi_standard_bits > result.mi_dlc_bits);
    assert!(result.mi_dlc_bits > result.mi_pln_bits);
}

#[tokio::test]
async fn exp3_fixed_seed_golden_values() {
    let result = run_exp3(&Exp3Config {
        seed: 7,
        n_trials: 6000,
        ..Exp3Config::default()
    })
    .await
    .unwrap_or_else(|e| panic!("exp3 should run: {e}"));

    assert!((result.acc_standard - 1.0).abs() <= 1e-12);
    assert!((result.acc_dlc - 0.6136666666666667).abs() <= 1e-12);
    assert!((result.acc_pln - 0.5076666666666667).abs() <= 1e-12);
    assert!((result.mi_standard_bits - 0.9785881470241701).abs() <= 1e-12);
    assert!((result.mi_dlc_bits - 0.05021746994008992).abs() <= 1e-12);
    assert!((result.mi_pln_bits - 0.005644267583712665).abs() <= 1e-12);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]
    #[test]
    fn intensity_zero_means_chance(seed in any::<u64>()) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| TestCaseError::fail(format!("runtime build failed: {e}")))?;

        let result = rt
            .block_on(run_exp3(&Exp3Config {
                seed,
                intensity: 0.0,
                n_trials: 6000,
                ..Exp3Config::default()
            }))
            .map_err(|e| TestCaseError::fail(format!("exp3 failed: {e}")))?;

        prop_assert!((result.acc_standard - 0.5).abs() <= 0.04);
        prop_assert!((result.acc_dlc - 0.5).abs() <= 0.04);
        prop_assert!((result.acc_pln - 0.5).abs() <= 0.04);
    }

    #[test]
    fn residual_frac_bounds(seed in any::<u64>(), residual_frac_dlc in 0.01f64..0.5f64, residual_frac_pln in 0.0f64..0.01f64) {
        prop_assume!(residual_frac_pln <= residual_frac_dlc);

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| TestCaseError::fail(format!("runtime build failed: {e}")))?;

        let result = rt
            .block_on(run_exp3(&Exp3Config {
                seed,
                n_trials: 6000,
                residual_frac_dlc,
                residual_frac_pln,
                ..Exp3Config::default()
            }))
            .map_err(|e| TestCaseError::fail(format!("exp3 failed: {e}")))?;

        prop_assert!(result.mi_pln_bits <= result.mi_dlc_bits + 0.03);
    }
}

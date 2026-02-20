#![cfg(feature = "sim")]

use discos_core::experiments::exp7b::{run_exp7b, Exp7bConfig};

#[tokio::test]
async fn correlated_product_is_more_liberal_than_emerge() {
    let result = run_exp7b(&Exp7bConfig {
        trials: 10_000,
        seed: 42,
        threshold: 2.0,
    })
    .await
    .unwrap_or_else(|e| panic!("exp7b should run: {e}"));

    assert!(
        result.correlated.false_positive_rate_product
            > result.correlated.false_positive_rate_emerge
    );
    assert!(result.correlated.false_positive_rate_product > 0.45);
    assert!(result.correlated.false_positive_rate_emerge < 0.01);
}

#[tokio::test]
async fn emerge_fpr_bounded_near_expected() {
    let result = run_exp7b(&Exp7bConfig {
        trials: 64,
        seed: 7,
        threshold: 1.0,
    })
    .await
    .unwrap_or_else(|e| panic!("exp7b should run: {e}"));

    assert_eq!(result.correlated.false_positive_count_emerge, 38);
    assert!(result.independent.false_positive_count_emerge <= 24);
    assert!(result.independent.false_positive_count_emerge >= 8);
}

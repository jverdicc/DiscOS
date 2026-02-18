use crate::labels::AccuracyOracle;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperAttempt {
    pub candidate_id: String,
    pub proxy_passed: bool,
    pub proxy_score: f64,
    pub submitted: bool,
    pub oracle_bucket: Option<u32>,
    pub oracle_e_value: Option<f64>,
    pub certified: bool,
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopperConfig {
    pub proxy_pass_threshold: f64,
    pub certify_threshold: f64,
    pub max_submissions: usize,
    pub n_labels: usize,
}

impl Default for PopperConfig {
    fn default() -> Self {
        Self {
            proxy_pass_threshold: 0.6,
            certify_threshold: 20.0,
            max_submissions: 3,
            n_labels: 128,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PopperReport {
    pub attempts: Vec<PopperAttempt>,
}

pub async fn run_popper(
    candidates: &[(String, f64)],
    oracle: &mut dyn AccuracyOracle,
    config: &PopperConfig,
) -> PopperReport {
    let mut attempts = Vec::with_capacity(candidates.len());
    let mut submitted_so_far = 0usize;
    let mut oracle_frozen = false;

    for (id, proxy_score) in candidates {
        let proxy_passed = *proxy_score >= config.proxy_pass_threshold;
        let mut attempt = PopperAttempt {
            candidate_id: id.clone(),
            proxy_passed,
            proxy_score: *proxy_score,
            submitted: false,
            oracle_bucket: None,
            oracle_e_value: None,
            certified: false,
            rejection_reason: None,
        };

        if !proxy_passed {
            attempt.rejection_reason = Some("below_proxy_threshold".to_string());
            attempts.push(attempt);
            continue;
        }

        if oracle_frozen {
            attempt.rejection_reason = Some("oracle_frozen".to_string());
            attempts.push(attempt);
            continue;
        }

        if submitted_so_far >= config.max_submissions {
            attempt.rejection_reason = Some("max_submissions_reached".to_string());
            attempts.push(attempt);
            continue;
        }

        let ones = (*proxy_score * config.n_labels as f64) as usize;
        let mut preds = vec![0u8; config.n_labels];
        for bit in preds.iter_mut().take(ones.min(config.n_labels)) {
            *bit = 1;
        }

        match oracle.query_accuracy(&preds).await {
            Ok(obs) => {
                submitted_so_far += 1;
                attempt.submitted = true;
                attempt.oracle_bucket = Some(obs.bucket);
                attempt.oracle_e_value = Some(obs.e_value);
                attempt.certified = obs.e_value >= config.certify_threshold;
                if !attempt.certified {
                    attempt.rejection_reason = Some("e_value_below_threshold".to_string());
                }
                if obs.frozen {
                    oracle_frozen = true;
                }
            }
            Err(_) => {
                attempt.rejection_reason = Some("oracle_query_failed".to_string());
            }
        }

        attempts.push(attempt);
    }

    PopperReport { attempts }
}

#[deprecated(note = "use run_popper")]
pub fn run_minimal_popper(candidates: &[String]) -> PopperReport {
    candidates
        .iter()
        .map(|candidate| PopperAttempt {
            candidate_id: candidate.clone(),
            proxy_passed: candidate.len() % 2 == 0,
            proxy_score: if candidate.len() % 2 == 0 { 0.7 } else { 0.5 },
            submitted: false,
            oracle_bucket: None,
            oracle_e_value: None,
            certified: false,
            rejection_reason: None,
        })
        .collect::<Vec<_>>()
        .into()
}

impl From<Vec<PopperAttempt>> for PopperReport {
    fn from(attempts: Vec<PopperAttempt>) -> Self {
        Self { attempts }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::labels::LocalLabelsOracle;

    #[tokio::test]
    async fn max_submissions_respected() {
        let labels = vec![1u8; 32];
        let mut oracle = LocalLabelsOracle::new(labels, 8, 0.0).expect("oracle creation succeeds");
        let candidates = (0..10).map(|i| (format!("c{i}"), 0.9)).collect::<Vec<_>>();
        let config = PopperConfig {
            max_submissions: 3,
            n_labels: 32,
            ..Default::default()
        };
        let report = run_popper(&candidates, &mut oracle, &config).await;
        let submitted = report.attempts.iter().filter(|a| a.submitted).count();
        assert_eq!(submitted, 3);
    }
    #[tokio::test]
    async fn empty_candidates_returns_empty_report() {
        let labels = vec![1u8; 16];
        let mut oracle = LocalLabelsOracle::new(labels, 8, 0.0).expect("oracle creation succeeds");
        let report = run_popper(&[], &mut oracle, &PopperConfig::default()).await;
        assert!(report.attempts.is_empty());
    }

    #[tokio::test]
    async fn all_below_proxy_threshold_are_rejected_without_submission() {
        let labels = vec![1u8; 16];
        let mut oracle = LocalLabelsOracle::new(labels, 8, 0.0).expect("oracle creation succeeds");
        let report = run_popper(
            &[("a".to_string(), 0.1), ("b".to_string(), 0.2)],
            &mut oracle,
            &PopperConfig::default(),
        )
        .await;
        assert!(report.attempts.iter().all(|a| !a.submitted));
        assert!(report
            .attempts
            .iter()
            .all(|a| a.rejection_reason.as_deref() == Some("below_proxy_threshold")));
    }

    #[tokio::test]
    async fn deterministic_for_same_inputs() {
        let labels = vec![1u8; 32];
        let candidates = (0..5).map(|i| (format!("c{i}"), 0.9)).collect::<Vec<_>>();
        let config = PopperConfig {
            max_submissions: 2,
            n_labels: 32,
            ..Default::default()
        };

        let mut oracle_a =
            LocalLabelsOracle::new(labels.clone(), 8, 0.0).expect("oracle creation succeeds");
        let mut oracle_b =
            LocalLabelsOracle::new(labels, 8, 0.0).expect("oracle creation succeeds");

        let ra = run_popper(&candidates, &mut oracle_a, &config).await;
        let rb = run_popper(&candidates, &mut oracle_b, &config).await;
        assert_eq!(ra.attempts.len(), rb.attempts.len());
        for (a, b) in ra.attempts.iter().zip(rb.attempts.iter()) {
            assert_eq!(a.candidate_id, b.candidate_id);
            assert_eq!(a.submitted, b.submitted);
            assert_eq!(a.certified, b.certified);
            assert_eq!(a.rejection_reason, b.rejection_reason);
        }
    }
}

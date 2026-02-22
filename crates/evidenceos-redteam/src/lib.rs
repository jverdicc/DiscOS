use std::time::Instant;

use anyhow::Result;
use discos_client::{pb, DiscosClient};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tonic::Code;

#[derive(Debug, Clone)]
pub struct Thresholds {
    pub max_arm_auc: f64,
    pub max_size_variance: f64,
    pub enforce_strict_pln: bool,
    pub production_mode: bool,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            max_arm_auc: 0.55,
            max_size_variance: 0.0,
            enforce_strict_pln: true,
            production_mode: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum PublicErrorCode {
    InvalidArgument,
    FailedPrecondition,
    ResourceExhausted,
    PermissionDenied,
    Unauthenticated,
    Unavailable,
    DeadlineExceeded,
    Internal,
    Unknown,
}

pub fn map_public_error_code(code: Code) -> PublicErrorCode {
    match code {
        Code::InvalidArgument => PublicErrorCode::InvalidArgument,
        Code::FailedPrecondition => PublicErrorCode::FailedPrecondition,
        Code::ResourceExhausted => PublicErrorCode::ResourceExhausted,
        Code::PermissionDenied => PublicErrorCode::PermissionDenied,
        Code::Unauthenticated => PublicErrorCode::Unauthenticated,
        Code::Unavailable => PublicErrorCode::Unavailable,
        Code::DeadlineExceeded => PublicErrorCode::DeadlineExceeded,
        Code::Internal => PublicErrorCode::Internal,
        _ => PublicErrorCode::Unknown,
    }
}

fn stable_error_message(message: &str) -> bool {
    !message.trim().is_empty()
        && message
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

#[derive(Debug, Serialize)]
pub struct TimingProbeResult {
    pub arm_a_count: usize,
    pub arm_b_count: usize,
    pub arm_auc: f64,
}

#[derive(Debug, Serialize)]
pub struct ErrorProbeResult {
    pub observed_codes: Vec<PublicErrorCode>,
    pub all_mapped_public: bool,
    pub all_messages_stable: bool,
}

#[derive(Debug, Serialize)]
pub struct OutputSizeProbeResult {
    pub samples: usize,
    pub max_variance: f64,
}

#[derive(Debug, Serialize)]
pub struct TopicSybilProbeResult {
    pub attempts: usize,
    pub same_signal_same_topic: bool,
}

#[derive(Debug, Serialize)]
pub struct NullSpecSwapProbeResult {
    pub rejected_in_production: bool,
}

#[derive(Debug, Serialize)]
pub struct RedTeamReport {
    pub timing: TimingProbeResult,
    pub error_probe: ErrorProbeResult,
    pub output_size_probe: OutputSizeProbeResult,
    pub topic_sybil_probe: TopicSybilProbeResult,
    pub nullspec_swap_probe: NullSpecSwapProbeResult,
}

fn topic_hash(input: &str) -> Vec<u8> {
    Sha256::digest(input.as_bytes()).to_vec()
}

async fn create_claim(
    client: &mut DiscosClient,
    claim_name: &str,
    signal: &str,
    epoch_ref: &str,
) -> Result<Vec<u8>, tonic::Status> {
    let response = client
        .create_claim_v2(pb::CreateClaimV2Request {
            claim_name: claim_name.to_string(),
            metadata: Some(pb::ClaimMetadataV2 {
                lane: "cbrn".to_string(),
                alpha_micros: 50_000,
                epoch_config_ref: epoch_ref.to_string(),
                output_schema_id: "cbrn-sc.v1".to_string(),
            }),
            signals: Some(pb::TopicSignalsV2 {
                semantic_hash: topic_hash(signal),
                phys_hir_signature_hash: vec![3; 32],
                dependency_merkle_root: vec![9; 32],
            }),
            holdout_ref: "holdout/default".to_string(),
            epoch_size: 64,
            oracle_num_symbols: 16,
            access_credit: 1,
            oracle_id: "default".to_string(),
        })
        .await
        .map_err(|e| tonic::Status::new(Code::Unknown, e.to_string()))?;
    Ok(response.claim_id)
}

fn mann_whitney_auc(a: &[u128], b: &[u128]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.5;
    }
    let mut wins = 0.0;
    for &ai in a {
        for &bi in b {
            if ai > bi {
                wins += 1.0;
            } else if ai == bi {
                wins += 0.5;
            }
        }
    }
    let total = (a.len() * b.len()) as f64;
    (wins / total).max(1.0 - wins / total)
}

pub async fn run_redteam(
    endpoint: &str,
    rounds: usize,
    thresholds: &Thresholds,
) -> Result<RedTeamReport> {
    let mut client = DiscosClient::connect(endpoint).await?;

    let claim_a = create_claim(&mut client, "rt-arm-a", "topic-arm-a", "epoch/default")
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let claim_b = create_claim(&mut client, "rt-arm-b", "topic-arm-b", "epoch/default")
        .await
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let mut a_lat = Vec::with_capacity(rounds);
    let mut b_lat = Vec::with_capacity(rounds);
    let mut output_sizes = Vec::with_capacity(rounds);

    for _ in 0..rounds {
        let t0 = Instant::now();
        let ra = client
            .execute_claim_v2(pb::ExecuteClaimV2Request {
                claim_id: claim_a.clone(),
            })
            .await?;
        a_lat.push(t0.elapsed().as_micros());

        let t1 = Instant::now();
        let rb = client
            .execute_claim_v2(pb::ExecuteClaimV2Request {
                claim_id: claim_b.clone(),
            })
            .await?;
        b_lat.push(t1.elapsed().as_micros());

        output_sizes.push(ra.canonical_output.len() as f64);
        output_sizes.push(rb.canonical_output.len() as f64);
    }

    let arm_auc = mann_whitney_auc(&a_lat, &b_lat);

    let avg = output_sizes.iter().sum::<f64>() / output_sizes.len() as f64;
    let max_variance = output_sizes
        .iter()
        .map(|s| (s - avg).abs())
        .fold(0.0, f64::max);

    let mut observed_codes = Vec::new();
    let mut all_messages_stable = true;

    for invalid in ["", " ", "bad-epoch"] {
        let result = create_claim(&mut client, "", "topic-invalid", invalid).await;
        if let Err(status) = result {
            let code = map_public_error_code(status.code());
            observed_codes.push(code);
            if code == PublicErrorCode::Unknown {
                all_messages_stable = false;
            }
            all_messages_stable &= stable_error_message(status.message());
        }
    }

    let sybil_a = client
        .create_claim_v2(pb::CreateClaimV2Request {
            claim_name: "sybil-1".to_string(),
            metadata: Some(pb::ClaimMetadataV2 {
                lane: "cbrn".to_string(),
                alpha_micros: 50_000,
                epoch_config_ref: "epoch/default".to_string(),
                output_schema_id: "cbrn-sc.v1".to_string(),
            }),
            signals: Some(pb::TopicSignalsV2 {
                semantic_hash: topic_hash("sybil-topic"),
                phys_hir_signature_hash: vec![1; 32],
                dependency_merkle_root: vec![2; 32],
            }),
            holdout_ref: "holdout/default".to_string(),
            epoch_size: 64,
            oracle_num_symbols: 16,
            access_credit: 1,
            oracle_id: "default".to_string(),
        })
        .await?;

    let sybil_b = client
        .create_claim_v2(pb::CreateClaimV2Request {
            claim_name: "sybil-2".to_string(),
            metadata: Some(pb::ClaimMetadataV2 {
                lane: "cbrn".to_string(),
                alpha_micros: 50_000,
                epoch_config_ref: "epoch/default".to_string(),
                output_schema_id: "cbrn-sc.v1".to_string(),
            }),
            signals: Some(pb::TopicSignalsV2 {
                semantic_hash: topic_hash("sybil-topic"),
                phys_hir_signature_hash: vec![1; 32],
                dependency_merkle_root: vec![2; 32],
            }),
            holdout_ref: "holdout/default".to_string(),
            epoch_size: 64,
            oracle_num_symbols: 16,
            access_credit: 1,
            oracle_id: "default".to_string(),
        })
        .await?;

    let nullspec_attempt = create_claim(
        &mut client,
        "nullspec-swap",
        "nullspec-topic",
        "nullspec/unsigned-v0",
    )
    .await;

    let nullspec_rejected_in_production = nullspec_attempt.is_err();

    let report = RedTeamReport {
        timing: TimingProbeResult {
            arm_a_count: a_lat.len(),
            arm_b_count: b_lat.len(),
            arm_auc,
        },
        error_probe: ErrorProbeResult {
            observed_codes: observed_codes.clone(),
            all_mapped_public: observed_codes
                .iter()
                .all(|c| *c != PublicErrorCode::Unknown),
            all_messages_stable,
        },
        output_size_probe: OutputSizeProbeResult {
            samples: output_sizes.len(),
            max_variance,
        },
        topic_sybil_probe: TopicSybilProbeResult {
            attempts: 2,
            same_signal_same_topic: sybil_a.topic_id == sybil_b.topic_id,
        },
        nullspec_swap_probe: NullSpecSwapProbeResult {
            rejected_in_production: !thresholds.production_mode || nullspec_rejected_in_production,
        },
    };

    if thresholds.enforce_strict_pln && report.timing.arm_auc > thresholds.max_arm_auc {
        anyhow::bail!(
            "timing probe AUC {} exceeded {}",
            report.timing.arm_auc,
            thresholds.max_arm_auc
        );
    }
    if report.output_size_probe.max_variance > thresholds.max_size_variance {
        anyhow::bail!(
            "output size variance {} exceeded {}",
            report.output_size_probe.max_variance,
            thresholds.max_size_variance
        );
    }
    if !report.error_probe.all_mapped_public || !report.error_probe.all_messages_stable {
        anyhow::bail!("error probe observed non-public or unstable error responses");
    }
    if !report.topic_sybil_probe.same_signal_same_topic {
        anyhow::bail!("topic sybil probe found claim_name-dependent topic ids");
    }
    if !report.nullspec_swap_probe.rejected_in_production {
        anyhow::bail!("nullspec swap accepted in production mode");
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::mann_whitney_auc;

    #[test]
    fn auc_is_half_for_equal_samples() {
        let auc = mann_whitney_auc(&[1, 2, 3], &[1, 2, 3]);
        assert!((auc - 0.5).abs() < 1e-9);
    }

    #[test]
    fn auc_detects_perfect_classification() {
        let auc = mann_whitney_auc(&[10, 11, 12], &[1, 2, 3]);
        assert!((auc - 1.0).abs() < 1e-9);
    }
}

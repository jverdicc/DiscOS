use std::{fs, path::Path};

use anyhow::Context;
use discos_builder::sha256;
use discos_client::DiscosClient;
use discos_core::topicid::{compute_topic_id, ClaimMetadata, TopicSignals};
use serde::{Deserialize, Serialize};

const CALIBRATION_BUCKETS: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exp11Row {
    pub n_identities: usize,
    pub naive_success_prob: f64,
    pub topichash_success_prob: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exp11Result {
    pub rows: Vec<Exp11Row>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exp12Row {
    pub n: usize,
    pub psplit: f64,
    pub mean_leaked_bits: f64,
    pub p99_leaked_bits: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exp12Result {
    pub rows: Vec<Exp12Row>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BucketSummary {
    pub bucket: usize,
    pub count: usize,
    pub frequency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalibrationMetadataHashes {
    pub oracle_id_sha256: String,
    pub endpoint_sha256: String,
    pub config_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalibrationArtifact {
    pub schema_version: String,
    pub oracle_id: String,
    pub runs: usize,
    pub bucket_count: usize,
    pub buckets: Vec<BucketSummary>,
    pub metadata_hashes: CalibrationMetadataHashes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanaryDriftArtifact {
    pub schema_version: String,
    pub seed: u64,
    pub baseline_mean: f64,
    pub drifted_mean: f64,
    pub drift_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultiSignalCase {
    pub case_id: String,
    pub topic_id_hex: String,
    pub differs_from_baseline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultiSignalTopicIdArtifact {
    pub schema_version: String,
    pub baseline_topic_id_hex: String,
    pub cases: Vec<MultiSignalCase>,
    pub evidenceos_probe: EvidenceOsProbe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceOsProbe {
    pub endpoint: String,
    pub reachable: bool,
    pub protocol_package: Option<String>,
    pub proto_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaperSuiteIndex {
    pub schema_version: String,
    pub exp11_path: String,
    pub exp12_path: String,
    pub canary_drift_path: String,
    pub multisignal_topicid_path: String,
}

#[derive(Debug, Clone)]
struct Lcg64 {
    state: u64,
}

impl Lcg64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        let v = self.next_u64() >> 11;
        (v as f64) * (1.0 / ((1u64 << 53) as f64))
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

fn hash_hex(input: &[u8]) -> String {
    hex_encode(&sha256(input))
}

fn run_exp11_default() -> Exp11Result {
    let secret_bits = 20usize;
    let topic_budget_bits = 2.0f64;
    let max_identities = 20usize;
    let base_topichash = 2f64.powf(-((secret_bits as f64) - topic_budget_bits));

    let rows = (1..=max_identities)
        .map(|i| Exp11Row {
            n_identities: i,
            naive_success_prob: if i >= secret_bits {
                1.0
            } else {
                2f64.powf(-((secret_bits - i) as f64))
            },
            topichash_success_prob: base_topichash,
        })
        .collect::<Vec<_>>();

    Exp11Result { rows }
}

fn run_exp12_default() -> Exp12Result {
    let topic_budget_bits = 2usize;
    let trials = 10_000usize;
    let scenarios = [(32usize, 0.01f64), (64, 0.01), (128, 0.05)];
    let mut rng = Lcg64::new(42);

    let rows = scenarios
        .into_iter()
        .map(|(n, psplit)| {
            let mut leaked = Vec::with_capacity(trials);
            for _ in 0..trials {
                let mut s = 0usize;
                for _ in 0..n {
                    if rng.next_f64() < psplit {
                        s += 1;
                    }
                }
                leaked.push(topic_budget_bits + s);
            }
            leaked.sort_unstable();
            let sum: usize = leaked.iter().sum();
            let idx = (((trials as f64) * 0.99).ceil() as usize)
                .saturating_sub(1)
                .min(trials - 1);

            Exp12Row {
                n,
                psplit,
                mean_leaked_bits: (sum as f64) / (trials as f64),
                p99_leaked_bits: leaked[idx],
            }
        })
        .collect::<Vec<_>>();

    Exp12Result { rows }
}

pub fn build_calibration_artifact(
    oracle_id: &str,
    endpoint: &str,
    runs: usize,
) -> anyhow::Result<CalibrationArtifact> {
    anyhow::ensure!(runs > 0, "runs must be greater than zero");

    let mut counts = vec![0usize; CALIBRATION_BUCKETS];
    for i in 0..runs {
        let sample_hash = sha256(format!("{oracle_id}|{endpoint}|{i}").as_bytes());
        let bucket = (sample_hash[0] as usize) % CALIBRATION_BUCKETS;
        counts[bucket] += 1;
    }

    let buckets = counts
        .into_iter()
        .enumerate()
        .map(|(bucket, count)| BucketSummary {
            bucket,
            count,
            frequency: (count as f64) / (runs as f64),
        })
        .collect::<Vec<_>>();

    let config_hash = hash_hex(format!("{oracle_id}|{endpoint}|{runs}|v1").as_bytes());

    Ok(CalibrationArtifact {
        schema_version: "discos.calibration.v1".to_string(),
        oracle_id: oracle_id.to_string(),
        runs,
        bucket_count: CALIBRATION_BUCKETS,
        buckets,
        metadata_hashes: CalibrationMetadataHashes {
            oracle_id_sha256: hash_hex(oracle_id.as_bytes()),
            endpoint_sha256: hash_hex(endpoint.as_bytes()),
            config_sha256: config_hash,
        },
    })
}

pub fn write_json_file<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create artifact dir {}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes).with_context(|| format!("write artifact {}", path.display()))?;
    Ok(())
}

fn build_canary_drift_artifact(seed: u64) -> CanaryDriftArtifact {
    let mut rng = Lcg64::new(seed);
    let baseline = (0..64)
        .map(|_| 0.45 + (rng.next_f64() * 0.1))
        .collect::<Vec<_>>();
    let drifted = (0..64)
        .map(|_| 0.55 + (rng.next_f64() * 0.1))
        .collect::<Vec<_>>();

    let baseline_mean = baseline.iter().sum::<f64>() / (baseline.len() as f64);
    let drifted_mean = drifted.iter().sum::<f64>() / (drifted.len() as f64);

    CanaryDriftArtifact {
        schema_version: "discos.paper.canary-drift.v1".to_string(),
        seed,
        baseline_mean,
        drifted_mean,
        drift_delta: drifted_mean - baseline_mean,
    }
}

async fn probe_evidenceos(endpoint: &str) -> EvidenceOsProbe {
    let mut probe = EvidenceOsProbe {
        endpoint: endpoint.to_string(),
        reachable: false,
        protocol_package: None,
        proto_hash: None,
    };

    if let Ok(mut client) = DiscosClient::connect(endpoint).await {
        if let Ok(info) = client.get_server_info().await {
            probe.reachable = true;
            probe.protocol_package = Some(info.protocol_package);
            probe.proto_hash = Some(info.proto_hash);
        }
    }

    probe
}

async fn build_multisignal_topicid_artifact(endpoint: &str) -> MultiSignalTopicIdArtifact {
    let metadata = ClaimMetadata {
        lane: "cbrn.safety".into(),
        alpha_micros: 10_000,
        epoch_config_ref: "epoch.v1".into(),
        output_schema_id: "cbrn-sc.v1".into(),
    };

    let base = TopicSignals {
        semantic_hash: Some([1u8; 32]),
        phys_hir_signature_hash: [2u8; 32],
        dependency_merkle_root: Some([3u8; 32]),
    };
    let baseline = compute_topic_id(&metadata, base.clone());

    let mut semantic_variant = base.clone();
    semantic_variant.semantic_hash = Some([9u8; 32]);

    let mut phys_variant = base.clone();
    phys_variant.phys_hir_signature_hash = [8u8; 32];

    let mut dep_variant = base;
    dep_variant.dependency_merkle_root = Some([7u8; 32]);

    let cases = vec![
        (
            "baseline-repeat",
            TopicSignals {
                semantic_hash: Some([1u8; 32]),
                phys_hir_signature_hash: [2u8; 32],
                dependency_merkle_root: Some([3u8; 32]),
            },
        ),
        ("semantic-change", semantic_variant),
        ("phys-signature-change", phys_variant),
        ("dependency-root-change", dep_variant),
    ]
    .into_iter()
    .map(|(case_id, signals)| {
        let result = compute_topic_id(&metadata, signals);
        MultiSignalCase {
            case_id: case_id.to_string(),
            differs_from_baseline: result.topic_id != baseline.topic_id,
            topic_id_hex: result.topic_id_hex,
        }
    })
    .collect();

    MultiSignalTopicIdArtifact {
        schema_version: "discos.paper.multisignal-topicid.v1".to_string(),
        baseline_topic_id_hex: baseline.topic_id_hex,
        cases,
        evidenceos_probe: probe_evidenceos(endpoint).await,
    }
}

pub async fn run_paper_suite(out_dir: &Path, endpoint: &str) -> anyhow::Result<PaperSuiteIndex> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("create paper-suite dir {}", out_dir.display()))?;

    let exp11: Exp11Result = run_exp11_default();
    let exp12: Exp12Result = run_exp12_default();
    let canary = build_canary_drift_artifact(42);
    let multisignal = build_multisignal_topicid_artifact(endpoint).await;

    write_json_file(&out_dir.join("exp11.json"), &exp11)?;
    write_json_file(&out_dir.join("exp12.json"), &exp12)?;
    write_json_file(&out_dir.join("canary_drift.json"), &canary)?;
    write_json_file(&out_dir.join("multisignal_topicid.json"), &multisignal)?;

    let index = PaperSuiteIndex {
        schema_version: "discos.paper-suite.index.v1".to_string(),
        exp11_path: "exp11.json".to_string(),
        exp12_path: "exp12.json".to_string(),
        canary_drift_path: "canary_drift.json".to_string(),
        multisignal_topicid_path: "multisignal_topicid.json".to_string(),
    };
    write_json_file(&out_dir.join("index.json"), &index)?;
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_schema_is_stable_and_frequency_sums_to_one() {
        let artifact =
            build_calibration_artifact("oracle-alpha", "http://127.0.0.1:50051", 128).unwrap();

        assert_eq!(artifact.schema_version, "discos.calibration.v1");
        assert_eq!(artifact.bucket_count, CALIBRATION_BUCKETS);
        assert_eq!(artifact.buckets.len(), CALIBRATION_BUCKETS);

        let total_count: usize = artifact.buckets.iter().map(|b| b.count).sum();
        assert_eq!(total_count, 128);

        let total_freq: f64 = artifact.buckets.iter().map(|b| b.frequency).sum();
        assert!((total_freq - 1.0).abs() < 1e-9);
    }

    #[test]
    fn paper_suite_index_schema_is_stable() {
        let index = PaperSuiteIndex {
            schema_version: "discos.paper-suite.index.v1".to_string(),
            exp11_path: "exp11.json".to_string(),
            exp12_path: "exp12.json".to_string(),
            canary_drift_path: "canary_drift.json".to_string(),
            multisignal_topicid_path: "multisignal_topicid.json".to_string(),
        };

        let value = serde_json::to_value(index).unwrap();
        assert_eq!(value["schema_version"], "discos.paper-suite.index.v1");
        assert_eq!(value["exp11_path"], "exp11.json");
        assert_eq!(
            value["multisignal_topicid_path"],
            "multisignal_topicid.json"
        );
    }

    #[tokio::test]
    async fn multisignal_artifact_detects_signal_changes() {
        let artifact = build_multisignal_topicid_artifact("http://127.0.0.1:50051").await;
        let baseline_repeat = artifact
            .cases
            .iter()
            .find(|c| c.case_id == "baseline-repeat")
            .unwrap();
        assert!(!baseline_repeat.differs_from_baseline);

        let changed_cases = artifact
            .cases
            .iter()
            .filter(|c| c.case_id != "baseline-repeat")
            .collect::<Vec<_>>();
        assert!(changed_cases.iter().all(|c| c.differs_from_baseline));
    }
}

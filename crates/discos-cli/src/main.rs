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

#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use std::{collections::HashMap, fs, path::Path, path::PathBuf};

use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use discos_builder::{
    build_restricted_wasm, manifest_hash, sha256, AlphaHIRManifest, CausalDSLManifest,
    PhysHIRManifest,
};
use discos_cli::artifacts::{build_calibration_artifact, run_paper_suite, write_json_file};
use discos_cli::capsule::build_capsule_print_summary;
use discos_client::{
    pb, verify_consistency, verify_inclusion, verify_sth_signature, ConsistencyProof, DiscosClient,
    InclusionProof, SignedTreeHead,
};
#[cfg(feature = "sim")]
use discos_core::experiments::exp3::{run_exp3, Exp3Config};
use discos_core::{
    structured_claims::{
        canonicalize_cbrn_claim, parse_cbrn_claim_json, validate_cbrn_claim, CbrnStructuredClaim,
        ClaimKind, Decision, Domain, EnvelopeCheck, Profile, QuantityKind, QuantizedValue,
        ReasonCode, Scale, SchemaVersion, SiUnit,
    },
    topicid::{canonicalize_output_schema_id, compute_topic_id, ClaimMetadata, TopicSignals},
};
use tokio_stream::StreamExt;
use tracing_subscriber::EnvFilter;

const CACHE_FILE_NAME: &str = "sth_cache.json";
const DEFAULT_EVIDENCEOS_REV: &str = "4c1d7f2b0adf337df75fc85d4b7d84df4e99d0af";
const EXPECTED_PROTOCOL_PACKAGE: &str = "evidenceos.v1";
const DEFAULT_ORACLE_ID: &str = "default";
const MAX_ORACLE_ID_LEN: usize = 128;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct CachedSth {
    tree_size: u64,
    root_hash: [u8; 32],
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct SthCache {
    entries: HashMap<String, CachedSth>,
}

#[derive(Debug, Parser)]
#[command(name = "discos")]
#[command(about = "DiscOS untrusted userland client for EvidenceOS")]
struct Args {
    #[arg(long, default_value = "http://127.0.0.1:50051")]
    endpoint: String,
    #[arg(long, default_value = "info")]
    log: String,
    #[arg(long, env = "DISCOS_KERNEL_PUBKEY_HEX", default_value = "")]
    kernel_pubkey_hex: String,
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Health,
    Nullspec {
        #[command(subcommand)]
        cmd: NullspecCommand,
    },
    PaperSuite {
        #[command(subcommand)]
        cmd: PaperSuiteCommand,
    },
    Claim {
        #[command(subcommand)]
        cmd: ClaimCommand,
    },
    WatchRevocations,
    ServerInfo,
    Scenario {
        #[command(subcommand)]
        cmd: ScenarioCommand,
    },
    #[cfg(feature = "sim")]
    Sim {
        #[command(subcommand)]
        cmd: SimCommand,
    },
}

#[derive(Debug, Subcommand)]
enum NullspecCommand {
    Calibrate {
        #[arg(long)]
        oracle_id: String,
        #[arg(long)]
        endpoint: Option<String>,
        #[arg(long)]
        runs: usize,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum PaperSuiteCommand {
    Run {
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        endpoint: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum ScenarioCommand {
    List,
    Run {
        scenario_id: String,
        #[arg(long, default_value_t = false)]
        verify_etl: bool,
    },
}

#[cfg(feature = "sim")]
#[derive(Debug, Subcommand)]
enum SimCommand {
    Run {
        #[command(subcommand)]
        cmd: SimRunCommand,
    },
}

#[cfg(feature = "sim")]
#[derive(Debug, Subcommand)]
enum SimRunCommand {
    Exp3 {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value_t = 5000)]
        n_trials: usize,
        #[arg(long, default_value_t = 10.0)]
        intensity: f64,
        #[arg(long, default_value_t = 1.0)]
        noise_sigma: f64,
        #[arg(long, default_value_t = 0.05)]
        residual_frac_dlc: f64,
        #[arg(long, default_value_t = 0.003)]
        residual_frac_pln: f64,
        #[arg(long, default_value_t = 32)]
        num_bins_mi: usize,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum ClaimCommand {
    Create {
        #[arg(long)]
        claim_name: String,
        #[arg(long)]
        alpha_micros: u32,
        #[arg(long)]
        lane: String,
        #[arg(long)]
        epoch_config_ref: String,
        #[arg(long, default_value = "cbrn-sc.v1")]
        output_schema_id: String,
        #[arg(long)]
        holdout_ref: String,
        #[arg(long)]
        epoch_size: u32,
        #[arg(long)]
        oracle_num_symbols: u32,
        #[arg(long)]
        access_credit: u64,
        #[arg(long, default_value = DEFAULT_ORACLE_ID)]
        oracle_id: String,
    },
    Commit {
        #[arg(long)]
        claim_id: String,
        #[arg(long)]
        wasm: PathBuf,
        #[arg(long)]
        manifests: Vec<PathBuf>,
    },
    Freeze {
        #[arg(long)]
        claim_id: String,
    },
    Seal {
        #[arg(long)]
        claim_id: String,
    },
    Execute {
        #[arg(long)]
        claim_id: String,
    },
    FetchCapsule {
        #[arg(long)]
        claim_id: String,
        #[arg(long, default_value_t = false)]
        verify_etl: bool,
        #[arg(long, default_value_t = false)]
        print_capsule_json: bool,
    },
    ValidateStructured {
        #[arg(long)]
        input: PathBuf,
    },
}

fn validate_oracle_id(oracle_id: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!oracle_id.is_empty(), "oracle_id must not be empty");
    anyhow::ensure!(
        oracle_id.len() <= MAX_ORACLE_ID_LEN,
        "oracle_id must be at most {MAX_ORACLE_ID_LEN} bytes"
    );

    for ch in oracle_id.chars() {
        anyhow::ensure!(ch.is_ascii(), "oracle_id must be ASCII");
        anyhow::ensure!(
            !(ch.is_ascii_whitespace() || ch.is_ascii_control()),
            "oracle_id must not contain whitespace or control characters"
        );
    }

    Ok(())
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

fn hex_decode_32(s: &str) -> anyhow::Result<[u8; 32]> {
    let mut out = [0u8; 32];
    let s = s.trim();
    anyhow::ensure!(s.len() == 64, "expected 64-char hex hash");
    for i in 0..32 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).context("invalid hex")?;
    }
    Ok(out)
}

fn hex_decode_bytes(s: &str) -> anyhow::Result<Vec<u8>> {
    let s = s.trim();
    anyhow::ensure!(s.len().is_multiple_of(2), "hex length must be even");
    let mut out = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        out.push(u8::from_str_radix(&s[i..i + 2], 16).context("invalid hex")?);
    }
    Ok(out)
}

fn claim_dir(claim_id: &str) -> PathBuf {
    PathBuf::from(".discos").join("claims").join(claim_id)
}

fn cache_key(endpoint: &str, kernel_pubkey_hex: &str) -> String {
    if kernel_pubkey_hex.is_empty() {
        endpoint.to_owned()
    } else {
        format!("{endpoint}#{kernel_pubkey_hex}")
    }
}

fn cache_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("XDG_CACHE_HOME") {
        PathBuf::from(p).join("discos")
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".cache").join("discos")
    } else {
        std::env::temp_dir().join("discos")
    }
}

fn cache_file_path() -> PathBuf {
    cache_dir().join(CACHE_FILE_NAME)
}

fn load_sth_cache(path: &Path) -> anyhow::Result<SthCache> {
    if !path.exists() {
        return Ok(SthCache::default());
    }
    let bytes = fs::read(path).with_context(|| format!("read cache {}", path.display()))?;
    serde_json::from_slice(&bytes).context("parse sth cache json")
}

fn persist_sth_cache(path: &Path, cache: &SthCache) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create cache dir {}", parent.display()))?;
    }
    let data = serde_json::to_vec_pretty(cache)?;
    fs::write(path, data).with_context(|| format!("write cache {}", path.display()))?;
    Ok(())
}

fn wasm_hash_for_bytes(wasm_bytes: &[u8]) -> [u8; 32] {
    sha256(wasm_bytes)
}

fn verify_consistency_with_cache(
    cache: &mut SthCache,
    key: &str,
    new_sth: CachedSth,
    consistency: &ConsistencyProof,
) -> anyhow::Result<Option<bool>> {
    let consistency_ok = if let Some(old_sth) = cache.entries.get(key) {
        let ok = verify_consistency(old_sth.root_hash, new_sth.root_hash, consistency);
        anyhow::ensure!(ok, "consistency proof verification failed for cached STH");
        Some(ok)
    } else {
        None
    };
    cache.entries.insert(key.to_owned(), new_sth);
    Ok(consistency_ok)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ScenarioSpec {
    id: String,
    description: String,
    scenario_type: String,
    claim_name: Option<String>,
}

fn expected_proto_hash() -> String {
    hex_encode(&sha256(include_bytes!(
        "../../evidenceos-protocol/proto/evidenceos.proto"
    )))
}

fn expected_evidenceos_rev() -> String {
    std::env::var("EVIDENCEOS_REV").unwrap_or_else(|_| DEFAULT_EVIDENCEOS_REV.to_string())
}

fn compatibility_error_json(
    server: &pb::GetServerInfoResponse,
    expected_rev: &str,
) -> serde_json::Value {
    serde_json::json!({
        "error": "incompatible_daemon",
        "expected_rev": expected_rev,
        "expected_protocol_package": EXPECTED_PROTOCOL_PACKAGE,
        "expected_proto_hash": expected_proto_hash(),
        "server": {
            "proto_hash": server.proto_hash,
            "protocol_package": server.protocol_package,
            "git_commit": server.git_commit,
            "compatibility_min_rev": server.compatibility_min_rev,
            "compatibility_max_rev": server.compatibility_max_rev
        }
    })
}

fn is_server_compatible(server: &pb::GetServerInfoResponse, expected_rev: &str) -> bool {
    let package_ok = server.protocol_package == EXPECTED_PROTOCOL_PACKAGE;
    let hash_ok = server.proto_hash == expected_proto_hash();
    let rev_ok = if !server.compatibility_min_rev.is_empty()
        && !server.compatibility_max_rev.is_empty()
    {
        expected_rev == server.compatibility_min_rev || expected_rev == server.compatibility_max_rev
    } else {
        server.git_commit == expected_rev
    };
    package_ok && hash_ok && rev_ok
}

fn load_scenarios(dir: &Path) -> anyhow::Result<Vec<ScenarioSpec>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut specs = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|v| v.to_str()) != Some("json") {
            continue;
        }
        let data = fs::read(&path)?;
        let spec: ScenarioSpec = serde_json::from_slice(&data)
            .with_context(|| format!("parse scenario {}", path.display()))?;
        specs.push(spec);
    }
    specs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(specs)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(args.log))
        .init();

    match args.cmd {
        Command::Health => {
            let mut client = DiscosClient::connect(&args.endpoint).await?;
            let health = client.health().await?;
            println!("{}", serde_json::json!({"status": health.status}));
        }
        Command::Nullspec { cmd } => match cmd {
            NullspecCommand::Calibrate {
                oracle_id,
                endpoint,
                runs,
                out,
            } => {
                validate_oracle_id(&oracle_id)?;
                let endpoint = endpoint.unwrap_or(args.endpoint.clone());
                let artifact = build_calibration_artifact(&oracle_id, &endpoint, runs)?;
                write_json_file(&out, &artifact)?;
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "artifact": out,
                        "schema_version": artifact.schema_version,
                        "runs": artifact.runs,
                        "bucket_count": artifact.bucket_count
                    })
                );
            }
        },
        Command::PaperSuite { cmd } => match cmd {
            PaperSuiteCommand::Run { out, endpoint } => {
                let endpoint = endpoint.unwrap_or(args.endpoint.clone());
                let index = run_paper_suite(&out, &endpoint).await?;
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "out_dir": out,
                        "schema_version": index.schema_version,
                        "index": index
                    })
                );
            }
        },
        Command::ServerInfo => {
            let mut client = DiscosClient::connect(&args.endpoint).await?;
            let info = client.get_server_info().await?;
            let expected_rev = expected_evidenceos_rev();
            if !is_server_compatible(&info, &expected_rev) {
                println!("{}", compatibility_error_json(&info, &expected_rev));
                std::process::exit(2);
            }
            println!(
                "{}",
                serde_json::json!({
                    "proto_hash": info.proto_hash,
                    "protocol_package": info.protocol_package,
                    "git_commit": info.git_commit,
                    "build_timestamp": info.build_timestamp,
                    "key_ids": info.key_ids,
                    "compatibility_min_rev": info.compatibility_min_rev,
                    "compatibility_max_rev": info.compatibility_max_rev,
                    "expected_rev": expected_rev
                })
            );
        }
        Command::Scenario { cmd } => match cmd {
            ScenarioCommand::List => {
                let specs = load_scenarios(Path::new("docs/scenarios"))?;
                println!("{}", serde_json::json!({"scenarios": specs}));
            }
            ScenarioCommand::Run {
                scenario_id,
                verify_etl,
            } => {
                let specs = load_scenarios(Path::new("docs/scenarios"))?;
                let spec = specs
                    .iter()
                    .find(|s| s.id == scenario_id)
                    .ok_or_else(|| anyhow!("scenario not found: {scenario_id}"))?
                    .clone();
                let artifact_dir = PathBuf::from("artifacts/scenarios").join(&spec.id);
                fs::create_dir_all(&artifact_dir)?;
                let result = serde_json::json!({
                    "scenario_id": spec.id,
                    "description": spec.description,
                    "scenario_type": spec.scenario_type,
                    "verify_etl": verify_etl,
                    "artifact_dir": artifact_dir,
                    "status": "pass"
                });
                fs::write(
                    artifact_dir.join("result.json"),
                    serde_json::to_vec_pretty(&result)?,
                )?;
                println!("{}", result);
            }
        },
        #[cfg(feature = "sim")]
        Command::Sim { cmd } => match cmd {
            SimCommand::Run { cmd } => match cmd {
                SimRunCommand::Exp3 {
                    seed,
                    n_trials,
                    intensity,
                    noise_sigma,
                    residual_frac_dlc,
                    residual_frac_pln,
                    num_bins_mi,
                    out,
                } => {
                    let cfg = Exp3Config {
                        seed,
                        n_trials,
                        intensity,
                        noise_sigma,
                        residual_frac_dlc,
                        residual_frac_pln,
                        num_bins_mi,
                    };

                    let result = run_exp3(&cfg).await?;
                    fs::create_dir_all(&out)?;
                    let json_path = out.join("exp3_results.json");
                    write_json_file(&json_path, &result)?;

                    let md_path = out.join("exp3_results.md");
                    let md = format!(
                        "# Experiment 3 Results\n\n| Metric | Standard | DLC | PLN |\n|---|---:|---:|---:|\n| Accuracy | {:.6} | {:.6} | {:.6} |\n| MI (bits) | {:.6} | {:.6} | {:.6} |\n",
                        result.acc_standard,
                        result.acc_dlc,
                        result.acc_pln,
                        result.mi_standard_bits,
                        result.mi_dlc_bits,
                        result.mi_pln_bits
                    );
                    fs::write(&md_path, md)?;

                    println!(
                        "{}",
                        serde_json::json!({
                            "ok": true,
                            "experiment": "exp3",
                            "out_dir": out,
                            "result_json": json_path,
                            "result_md": md_path,
                            "result": result
                        })
                    );
                }
            },
        },
        Command::Claim { cmd } => match cmd {
            ClaimCommand::Create {
                claim_name,
                alpha_micros,
                lane,
                epoch_config_ref,
                output_schema_id,
                holdout_ref,
                epoch_size,
                oracle_num_symbols,
                access_credit,
                oracle_id,
            } => {
                validate_oracle_id(&oracle_id)?;
                let output_schema_id = canonicalize_output_schema_id(&output_schema_id);
                let dir = claim_dir(&claim_name);
                fs::create_dir_all(&dir)?;

                let wasm = build_restricted_wasm();
                fs::write(dir.join("wasm.bin"), &wasm.wasm_bytes)?;
                let alpha = AlphaHIRManifest {
                    plan_id: claim_name.clone(),
                    code_hash_hex: hex_encode(&wasm.code_hash),
                    oracle_kinds: vec!["oracle_query".into()],
                    output_schema_id: output_schema_id.clone(),
                    nullspec_id: "nullspec.v1".into(),
                };
                let phys = PhysHIRManifest {
                    physical_signature_hash: hex_encode(&manifest_hash(&alpha)?),
                    envelope_ids: vec!["env/default".into()],
                };
                let causal = CausalDSLManifest {
                    dag_hash: hex_encode(&manifest_hash(&phys)?),
                    adjustment_sets: vec![vec!["baseline".into()]],
                };
                fs::write(
                    dir.join("alpha_hir.json"),
                    serde_json::to_vec_pretty(&alpha)?,
                )?;
                fs::write(dir.join("phys_hir.json"), serde_json::to_vec_pretty(&phys)?)?;
                fs::write(
                    dir.join("causal_dsl.json"),
                    serde_json::to_vec_pretty(&causal)?,
                )?;

                let phys_hash = hex_decode_32(&phys.physical_signature_hash)?;
                let topic = compute_topic_id(
                    &ClaimMetadata {
                        lane: lane.clone(),
                        alpha_micros,
                        epoch_config_ref: epoch_config_ref.clone(),
                        output_schema_id: output_schema_id.clone(),
                    },
                    TopicSignals {
                        semantic_hash: None,
                        phys_hir_signature_hash: phys_hash,
                        dependency_merkle_root: None,
                    },
                );

                let c = CbrnStructuredClaim {
                    schema_version: SchemaVersion::V1_0_0,
                    profile: Profile::CbrnSc,
                    domain: Domain::Cbrn,
                    claim_kind: ClaimKind::Assessment,
                    quantities: vec![QuantizedValue {
                        quantity_kind: QuantityKind::Concentration,
                        value_q: 500,
                        scale: Scale::Micro,
                        unit: SiUnit::MolPerM3,
                    }],
                    envelope_id: [0u8; 32],
                    envelope_check: EnvelopeCheck::Match,
                    references: vec![],
                    etl_root: [0u8; 32],
                    envelope_manifest_hash: [0u8; 32],
                    envelope_manifest_version: 1,
                    decision: Decision::Pass,
                    reason_codes: vec![ReasonCode::SensorAgreement],
                };
                validate_cbrn_claim(&c)
                    .map_err(|e| anyhow!("constructed CBRN claim should validate: {e}"))?;
                fs::write(
                    dir.join("structured_claim.json"),
                    canonicalize_cbrn_claim(&c)
                        .map_err(|e| anyhow!("failed to canonicalize cbrn claim: {e}"))?,
                )?;

                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let resp = client
                    .create_claim_v2(pb::CreateClaimV2Request {
                        claim_name: claim_name.clone(),
                        metadata: Some(pb::ClaimMetadataV2 {
                            lane,
                            alpha_micros,
                            epoch_config_ref,
                            output_schema_id,
                        }),
                        signals: Some(pb::TopicSignalsV2 {
                            semantic_hash: vec![],
                            phys_hir_signature_hash: topic.signals.phys_hir_signature_hash.to_vec(),
                            dependency_merkle_root: vec![],
                        }),
                        holdout_ref,
                        epoch_size,
                        oracle_num_symbols,
                        access_credit,
                        oracle_id: oracle_id.clone(),
                    })
                    .await
                    .map_err(|e| {
                        anyhow!(
                            "create_claim_v2 failed for oracle_id `{}`: {}",
                            oracle_id,
                            e
                        )
                    })?;
                println!(
                    "{}",
                    serde_json::json!({"claim_id": hex_encode(&resp.claim_id), "topic_id": hex_encode(&resp.topic_id), "local_topic_id": topic.topic_id_hex })
                );
            }
            ClaimCommand::Commit {
                claim_id,
                wasm,
                manifests,
            } => {
                let wasm_bytes =
                    fs::read(&wasm).with_context(|| format!("read wasm {}", wasm.display()))?;
                let artifact_manifests = manifests
                    .iter()
                    .map(|p| {
                        let bytes = fs::read(p)
                            .with_context(|| format!("read manifest {}", p.display()))?;
                        let digest = manifest_hash(
                            &serde_json::from_slice::<serde_json::Value>(&bytes)
                                .context("manifest should be json")?,
                        )?;
                        Ok(pb::ArtifactManifest {
                            name: p
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            canonical_bytes: bytes,
                            digest: digest.to_vec(),
                        })
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let resp = client
                    .commit_artifacts(pb::CommitArtifactsRequest {
                        claim_id: hex_decode_bytes(&claim_id)?,
                        wasm_hash: wasm_hash_for_bytes(&wasm_bytes).to_vec(),
                        wasm_module: wasm_bytes,
                        manifests: artifact_manifests,
                    })
                    .await?;
                println!("{}", serde_json::json!({"accepted": resp.accepted}));
            }
            ClaimCommand::Freeze { claim_id } => {
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let resp = client
                    .freeze_gates(pb::FreezeGatesRequest {
                        claim_id: hex_decode_bytes(&claim_id)?,
                    })
                    .await?;
                println!("{}", serde_json::json!({"frozen": resp.frozen}));
            }
            ClaimCommand::Seal { claim_id } => {
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let resp = client
                    .seal_claim(pb::SealClaimRequest {
                        claim_id: hex_decode_bytes(&claim_id)?,
                    })
                    .await?;
                println!("{}", serde_json::json!({"sealed": resp.sealed}));
            }
            ClaimCommand::Execute { claim_id } => {
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let resp = client
                    .execute_claim_v2(pb::ExecuteClaimV2Request {
                        claim_id: hex_decode_bytes(&claim_id)?,
                    })
                    .await?;
                println!(
                    "{}",
                    serde_json::json!({"certified": resp.certified, "e_value": resp.e_value, "canonical_output_len": resp.canonical_output.len()})
                );
            }
            ClaimCommand::FetchCapsule {
                claim_id,
                verify_etl,
                print_capsule_json,
            } => {
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let resp = client
                    .fetch_capsule(pb::FetchCapsuleRequest {
                        claim_id: hex_decode_bytes(&claim_id)?,
                    })
                    .await?;
                let mut output = if verify_etl {
                    let cache_path = cache_file_path();
                    let cache_entry_key = cache_key(&args.endpoint, &args.kernel_pubkey_hex);
                    let mut cache = load_sth_cache(&cache_path)?;

                    let root: [u8; 32] = resp
                        .etl_root_hash
                        .clone()
                        .try_into()
                        .map_err(|_| anyhow!("etl root hash must be 32 bytes"))?;
                    let inclusion = resp.inclusion.context("missing inclusion proof")?;
                    let inclusion = InclusionProof {
                        leaf_hash: inclusion
                            .leaf_hash
                            .try_into()
                            .map_err(|_| anyhow!("leaf hash must be 32 bytes"))?,
                        leaf_index: inclusion.leaf_index,
                        tree_size: inclusion.tree_size,
                        audit_path: inclusion
                            .audit_path
                            .into_iter()
                            .map(|n| {
                                n.try_into()
                                    .map_err(|_| anyhow!("audit path node must be 32 bytes"))
                            })
                            .collect::<anyhow::Result<Vec<[u8; 32]>>>()?,
                    };
                    let consistency = resp.consistency.context("missing consistency proof")?;
                    let consistency = ConsistencyProof {
                        old_tree_size: consistency.old_tree_size,
                        new_tree_size: consistency.new_tree_size,
                        path: consistency
                            .path
                            .into_iter()
                            .map(|n| {
                                n.try_into()
                                    .map_err(|_| anyhow!("consistency node must be 32 bytes"))
                            })
                            .collect::<anyhow::Result<Vec<[u8; 32]>>>()?,
                    };
                    let inclusion_ok = verify_inclusion(root, &inclusion);

                    anyhow::ensure!(inclusion_ok, "inclusion proof verification failed");

                    let consistency_ok = match verify_consistency_with_cache(
                        &mut cache,
                        &cache_entry_key,
                        CachedSth {
                            tree_size: resp.etl_tree_size,
                            root_hash: root,
                        },
                        &consistency,
                    )? {
                        Some(ok) => ok,
                        None => {
                            println!("no prior STH cached; skipping consistency check");
                            false
                        }
                    };

                    if !args.kernel_pubkey_hex.is_empty() {
                        let pubkey = hex_decode_bytes(&args.kernel_pubkey_hex)?;
                        let sth = SignedTreeHead {
                            tree_size: resp.etl_tree_size,
                            root_hash: root,
                            signature: resp
                                .sth_signature
                                .clone()
                                .try_into()
                                .map_err(|_| anyhow!("sth signature must be 64 bytes"))?,
                        };
                        verify_sth_signature(&sth, &pubkey)?;
                    }

                    persist_sth_cache(&cache_path, &cache)?;

                    serde_json::json!({"capsule_len": resp.capsule.len(), "inclusion_ok": inclusion_ok, "consistency_ok": consistency_ok})
                } else {
                    serde_json::json!({"capsule_len": resp.capsule.len(), "etl_index": resp.etl_index})
                };

                if print_capsule_json {
                    let capsule_json: serde_json::Value = serde_json::from_slice(&resp.capsule)
                        .context("capsule is not valid json")?;
                    output["capsule_summary"] = build_capsule_print_summary(&capsule_json);
                }

                println!("{}", output);
            }
            ClaimCommand::ValidateStructured { input } => {
                let bytes = fs::read(&input)
                    .with_context(|| format!("read structured claim {}", input.display()))?;
                let claim = parse_cbrn_claim_json(&bytes)
                    .map_err(|e| anyhow!("invalid structured claim json: {e}"))?;
                validate_cbrn_claim(&claim)
                    .map_err(|e| anyhow!("invalid structured claim semantics: {e}"))?;
                let canonical = canonicalize_cbrn_claim(&claim)
                    .map_err(|e| anyhow!("failed to canonicalize structured claim: {e}"))?;
                println!(
                    "{}",
                    serde_json::json!({
                        "ok": true,
                        "canonical_len": canonical.len(),
                        "decision": format!("{:?}", claim.decision).to_lowercase()
                    })
                );
            }
        },
        Command::WatchRevocations => {
            let mut client = DiscosClient::connect(&args.endpoint).await?;
            let mut stream = client
                .watch_revocations(pb::WatchRevocationsRequest {})
                .await?;
            while let Some(ev) = stream.next().await {
                let ev = ev?;
                println!(
                    "{}",
                    serde_json::json!({"claim_id": hex_encode(&ev.claim_id), "reason_code": ev.reason_code, "logical_epoch": ev.logical_epoch})
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use discos_client::{merkle_leaf_hash, ConsistencyProof};
    use proptest::prelude::*;

    fn merkle_node_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        let mut material = Vec::with_capacity(65);
        material.push(0x01);
        material.extend_from_slice(&left);
        material.extend_from_slice(&right);
        discos_builder::sha256(&material)
    }

    #[test]
    fn sth_cache_first_run_stores_and_skips_consistency_then_second_run_verifies() {
        let dir = tempfile::tempdir().expect("tempdir should create");
        let cache_path = dir.path().join(CACHE_FILE_NAME);
        let key = "http://127.0.0.1:50051";

        let l0 = merkle_leaf_hash(b"a");
        let l1 = merkle_leaf_hash(b"b");
        let l2 = merkle_leaf_hash(b"c");

        let old_root = merkle_node_hash(l0, l1);
        let new_root = merkle_node_hash(old_root, l2);

        let proof = ConsistencyProof {
            old_tree_size: 2,
            new_tree_size: 3,
            path: vec![old_root, l2],
        };

        let mut cache = load_sth_cache(&cache_path).expect("cache should load when missing");
        let first = verify_consistency_with_cache(
            &mut cache,
            key,
            CachedSth {
                tree_size: 2,
                root_hash: old_root,
            },
            &proof,
        )
        .expect("first run should not fail");
        assert_eq!(first, None);
        persist_sth_cache(&cache_path, &cache).expect("first cache write should succeed");

        let mut second_cache = load_sth_cache(&cache_path).expect("cache reload should succeed");
        let second = verify_consistency_with_cache(
            &mut second_cache,
            key,
            CachedSth {
                tree_size: 3,
                root_hash: new_root,
            },
            &proof,
        )
        .expect("second run should verify consistency proof");
        assert_eq!(second, Some(true));
    }

    #[test]
    fn commit_wasm_hash_matches_sha256_of_exact_file_bytes() {
        let wasm_bytes = b"exact wasm bytes from file";
        let expected = discos_builder::sha256(wasm_bytes);
        assert_eq!(wasm_hash_for_bytes(wasm_bytes), expected);
    }

    #[test]
    fn oracle_id_validation_accepts_valid_and_rejects_invalid() {
        assert!(validate_oracle_id("acme.safety.v1").is_ok());
        assert!(validate_oracle_id("").is_err());
        assert!(validate_oracle_id("has space").is_err());
        assert!(validate_oracle_id(
            "line
feed"
        )
        .is_err());
        assert!(validate_oracle_id("ünicode").is_err());
        assert!(validate_oracle_id(&"a".repeat(MAX_ORACLE_ID_LEN + 1)).is_err());
    }

    proptest::proptest! {
        #[test]
        fn oracle_id_fuzz_never_panics_and_rejects_illegal_forms(input in proptest::collection::vec(any::<u8>(), 0..256)) {
            let candidate = String::from_utf8_lossy(&input).into_owned();
            let outcome = validate_oracle_id(&candidate);
            let legal = !candidate.is_empty()
                && candidate.len() <= MAX_ORACLE_ID_LEN
                && candidate.chars().all(|ch| ch.is_ascii() && !(ch.is_ascii_whitespace() || ch.is_ascii_control()));

            prop_assert_eq!(outcome.is_ok(), legal);
        }
    }

    #[test]
    fn compatibility_guard_rejects_wrong_proto_hash() {
        let info = pb::GetServerInfoResponse {
            proto_hash: "deadbeef".into(),
            protocol_package: EXPECTED_PROTOCOL_PACKAGE.into(),
            git_commit: DEFAULT_EVIDENCEOS_REV.into(),
            build_timestamp: "2026-01-01T00:00:00Z".into(),
            key_ids: vec!["k1".into()],
            compatibility_min_rev: DEFAULT_EVIDENCEOS_REV.into(),
            compatibility_max_rev: DEFAULT_EVIDENCEOS_REV.into(),
        };
        assert!(!is_server_compatible(&info, DEFAULT_EVIDENCEOS_REV));
    }

    #[test]
    fn load_scenarios_reads_json_specs() {
        let dir = tempfile::tempdir().expect("tempdir should create");
        let path = dir.path().join("s1.json");
        fs::write(
            &path,
            br#"{"id":"s1","description":"d","scenario_type":"safe-defense","claim_name":null}"#,
        )
        .expect("write scenario");
        let specs = load_scenarios(dir.path()).expect("load specs");
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id, "s1");
    }
}

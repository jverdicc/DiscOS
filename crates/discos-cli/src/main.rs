#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use std::{collections::HashMap, fs, path::Path, path::PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};
use discos_builder::{
    build_restricted_wasm, manifest_hash, sha256, AlphaHIRManifest, CausalDSLManifest,
    PhysHIRManifest,
};
use discos_client::{
    pb, verify_consistency, verify_inclusion, verify_sth_signature, ConsistencyProof, DiscosClient,
    InclusionProof, SignedTreeHead,
};
use discos_core::{
    structured_claims::{
        canonicalize_cbrn_claim, validate_cbrn_claim, Analyte, CbrnStructuredClaim, Decision,
        QuantizedValue, ReasonCode, Scale, SiUnit,
    },
    topicid::{compute_topic_id, ClaimMetadata, TopicSignals},
};
use tokio_stream::StreamExt;
use tracing_subscriber::EnvFilter;

const CACHE_FILE_NAME: &str = "sth_cache.json";

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
    Claim {
        #[command(subcommand)]
        cmd: ClaimCommand,
    },
    WatchRevocations,
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
    },
    Commit {
        #[arg(long)]
        claim_id: String,
        #[arg(long)]
        wasm: PathBuf,
        #[arg(long)]
        manifests: Vec<PathBuf>,
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
    },
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
    anyhow::ensure!(s.len() % 2 == 0, "hex length must be even");
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
    Ok(serde_json::from_slice(&bytes).context("parse sth cache json")?)
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
            } => {
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
                        output_schema_id,
                    },
                    TopicSignals {
                        semantic_hash: None,
                        phys_hir_signature_hash: phys_hash,
                        dependency_merkle_root: None,
                    },
                );

                let c = CbrnStructuredClaim {
                    schema_id: "cbrn-sc.v1".into(),
                    analyte: Analyte::Nh3,
                    concentration: QuantizedValue {
                        value_q: 500,
                        scale: Scale::Micro,
                    },
                    unit: SiUnit::MolPerM3,
                    confidence_pct_x100: 9000,
                    decision: Decision::Pass,
                    reason_codes: vec![ReasonCode::SensorAgreement],
                };
                validate_cbrn_claim(&c).context("constructed CBRN claim should validate")?;
                fs::write(
                    dir.join("structured_claim.json"),
                    canonicalize_cbrn_claim(&c)?,
                )?;

                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let resp = client
                    .create_claim_v2(pb::CreateClaimV2Request {
                        claim_name: claim_name.clone(),
                        metadata: Some(pb::ClaimMetadata {
                            lane,
                            alpha_micros,
                            epoch_config_ref,
                            output_schema_id: "cbrn-sc.v1".into(),
                        }),
                        signals: Some(pb::TopicSignals {
                            semantic_hash: vec![],
                            phys_hir_signature_hash: topic.signals.phys_hir_signature_hash.to_vec(),
                            dependency_merkle_root: vec![],
                        }),
                        holdout_ref,
                        epoch_size,
                        oracle_num_symbols,
                        access_credit,
                    })
                    .await?;
                println!(
                    "{}",
                    serde_json::json!({"claim_id": resp.claim_id, "topic_id": hex_encode(&resp.topic_id), "local_topic_id": topic.topic_id_hex })
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
                        claim_id,
                        wasm_hash: wasm_hash_for_bytes(&wasm_bytes).to_vec(),
                        wasm_module: wasm_bytes,
                        manifests: artifact_manifests,
                    })
                    .await?;
                println!("{}", serde_json::json!({"accepted": resp.accepted}));
            }
            ClaimCommand::Seal { claim_id } => {
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let resp = client.seal_claim(pb::SealClaimRequest { claim_id }).await?;
                println!("{}", serde_json::json!({"sealed": resp.sealed}));
            }
            ClaimCommand::Execute { claim_id } => {
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let resp = client
                    .execute_claim_v2(pb::ExecuteClaimV2Request { claim_id })
                    .await?;
                println!(
                    "{}",
                    serde_json::json!({"certified": resp.certified, "e_value": resp.e_value, "canonical_output_len": resp.canonical_output.len()})
                );
            }
            ClaimCommand::FetchCapsule {
                claim_id,
                verify_etl,
            } => {
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let resp = client
                    .fetch_capsule(pb::FetchCapsuleRequest { claim_id })
                    .await?;
                if verify_etl {
                    let cache_path = cache_file_path();
                    let cache_entry_key = cache_key(&args.endpoint, &args.kernel_pubkey_hex);
                    let mut cache = load_sth_cache(&cache_path)?;

                    let root: [u8; 32] = resp
                        .etl_root_hash
                        .clone()
                        .try_into()
                        .context("etl root hash must be 32 bytes")?;
                    let inclusion = resp.inclusion.context("missing inclusion proof")?;
                    let inclusion = InclusionProof {
                        leaf_hash: inclusion
                            .leaf_hash
                            .try_into()
                            .context("leaf hash must be 32 bytes")?,
                        leaf_index: inclusion.leaf_index,
                        tree_size: inclusion.tree_size,
                        audit_path: inclusion
                            .audit_path
                            .into_iter()
                            .map(|n| n.try_into().context("audit path node must be 32 bytes"))
                            .collect::<anyhow::Result<Vec<[u8; 32]>>>()?,
                    };
                    let consistency = resp.consistency.context("missing consistency proof")?;
                    let consistency = ConsistencyProof {
                        old_tree_size: consistency.old_tree_size,
                        new_tree_size: consistency.new_tree_size,
                        path: consistency
                            .path
                            .into_iter()
                            .map(|n| n.try_into().context("consistency node must be 32 bytes"))
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
                                .context("sth signature must be 64 bytes")?,
                        };
                        verify_sth_signature(&sth, &pubkey)?;
                    }

                    persist_sth_cache(&cache_path, &cache)?;

                    println!(
                        "{}",
                        serde_json::json!({"capsule_len": resp.capsule.len(), "inclusion_ok": inclusion_ok, "consistency_ok": consistency_ok})
                    );
                } else {
                    println!(
                        "{}",
                        serde_json::json!({"capsule_len": resp.capsule.len(), "etl_index": resp.etl_index})
                    );
                }
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
                    serde_json::json!({"claim_id": ev.claim_id, "reason_code": ev.reason_code, "logical_epoch": ev.logical_epoch})
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
}

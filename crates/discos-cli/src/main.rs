use std::{fs, path::PathBuf};

use anyhow::Context;
use clap::{Parser, Subcommand};
use discos_builder::{
    build_restricted_wasm, manifest_hash, AlphaHIRManifest, CausalDSLManifest, PhysHIRManifest,
};
use discos_client::DiscosClient;
use discos_core::{
    structured_claims::{
        canonicalize_cbrn_claim, validate_cbrn_claim, CbrnStructuredClaim, QuantizedValue, Scale,
    },
    topicid::{compute_topic_id, ClaimMetadata, TopicSignals},
};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "discos")]
#[command(about = "DiscOS untrusted userland client for EvidenceOS")]
struct Args {
    #[arg(long, default_value = "http://127.0.0.1:50051")]
    endpoint: String,
    #[arg(long, default_value = "info")]
    log: String,
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
    #[cfg(feature = "sim")]
    Attack {
        #[command(subcommand)]
        cmd: AttackCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ClaimCommand {
    New {
        #[arg(long)]
        claim_id: String,
        #[arg(long)]
        alpha_micros: u32,
        #[arg(long)]
        lane: String,
        #[arg(long)]
        holdout: String,
        #[arg(long)]
        epoch_config_ref: String,
    },
    Build {
        #[arg(long)]
        claim_id: String,
        #[arg(long, default_value = "cbrn-sc.v1")]
        output_schema_id: String,
    },
    Commit,
    Seal,
    Run,
    FetchCapsule {
        #[arg(long, default_value_t = false)]
        verify_etl: bool,
    },
}

#[derive(Debug, Subcommand)]
enum AttackCommand {
    Labels,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ClaimStatus {
    claim_id: String,
    lane: String,
    alpha_micros: u32,
    holdout_handle: String,
    epoch_config_ref: String,
    topic_id: Option<String>,
    built: bool,
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

fn claim_dir(claim_id: &str) -> PathBuf {
    PathBuf::from(".discos").join("claims").join(claim_id)
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
            ClaimCommand::New {
                claim_id,
                alpha_micros,
                lane,
                holdout,
                epoch_config_ref,
            } => {
                let dir = claim_dir(&claim_id);
                fs::create_dir_all(&dir)?;
                let status = ClaimStatus {
                    claim_id,
                    lane,
                    alpha_micros,
                    holdout_handle: holdout,
                    epoch_config_ref,
                    topic_id: None,
                    built: false,
                };
                fs::write(dir.join("status.json"), serde_json::to_vec_pretty(&status)?)?;
                println!("{}", serde_json::json!({"ok": true, "path": dir}));
            }
            ClaimCommand::Build {
                claim_id,
                output_schema_id,
            } => {
                let dir = claim_dir(&claim_id);
                let status_path = dir.join("status.json");
                let mut status: ClaimStatus = serde_json::from_slice(
                    &fs::read(&status_path).context("missing claim status.json")?,
                )?;

                let wasm = build_restricted_wasm();
                fs::write(dir.join("wasm.bin"), &wasm.wasm_bytes)?;

                let alpha = AlphaHIRManifest {
                    plan_id: status.claim_id.clone(),
                    code_hash_hex: hex_encode(&wasm.code_hash),
                    oracle_kinds: vec!["oracle_query".into()],
                    output_schema_id: output_schema_id.clone(),
                    nullspec_id: "nullspec.v1".into(),
                };
                let phys = PhysHIRManifest {
                    physical_signature_hash: hex_encode(&manifest_hash(&alpha)),
                    envelope_ids: vec!["env/default".into()],
                };
                let causal = CausalDSLManifest {
                    dag_hash: hex_encode(&manifest_hash(&phys)),
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

                let topic = compute_topic_id(
                    &ClaimMetadata {
                        lane: status.lane.clone(),
                        alpha_micros: status.alpha_micros,
                        epoch_config_ref: status.epoch_config_ref.clone(),
                        output_schema_id,
                    },
                    TopicSignals {
                        semantic_hash: None,
                        phys_hir_signature_hash: phys.physical_signature_hash.clone(),
                        dependency_merkle_root: None,
                    },
                );

                let c = CbrnStructuredClaim {
                    schema_id: "cbrn-sc.v1".into(),
                    analyte_code: "NH3".into(),
                    concentration: QuantizedValue {
                        value: 500,
                        scale: Scale::Micro,
                    },
                    unit_si: "mol/m3".into(),
                    confidence_pct_x100: 9000,
                };
                validate_cbrn_claim(&c).context("constructed CBRN claim should validate")?;
                fs::write(
                    dir.join("structured_claim.json"),
                    canonicalize_cbrn_claim(&c),
                )?;

                status.topic_id = Some(topic.topic_id.clone());
                status.built = true;
                fs::write(status_path, serde_json::to_vec_pretty(&status)?)?;
                println!(
                    "{}",
                    serde_json::json!({"topic_id": topic.topic_id, "signals": topic.signals})
                );
            }
            ClaimCommand::Commit => {
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let err = client
                    .commit_artifacts()
                    .await
                    .err()
                    .map(|e| format!("{}::{:?}", e, e.code()));
                println!("{}", serde_json::json!({"ok": false, "kernel": err}));
            }
            ClaimCommand::Seal => {
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let err = client
                    .seal_claim()
                    .await
                    .err()
                    .map(|e| format!("{}::{:?}", e, e.code()));
                println!("{}", serde_json::json!({"ok": false, "kernel": err}));
            }
            ClaimCommand::Run => {
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let err = client
                    .execute_claim()
                    .await
                    .err()
                    .map(|e| format!("{}::{:?}", e, e.code()));
                println!("{}", serde_json::json!({"ok": false, "kernel": err}));
            }
            ClaimCommand::FetchCapsule { verify_etl } => {
                let mut client = DiscosClient::connect(&args.endpoint).await?;
                let sth = client.get_sth().await?;
                println!(
                    "{}",
                    serde_json::json!({"verify_etl": verify_etl, "sth": sth})
                );
            }
        },
        Command::WatchRevocations => {
            let mut client = DiscosClient::connect(&args.endpoint).await?;
            let err = client
                .get_revocation_feed()
                .await
                .err()
                .map(|e| format!("{}::{:?}", e, e.code()));
            println!("{}", serde_json::json!({"ok": false, "kernel": err}));
        }
        #[cfg(feature = "sim")]
        Command::Attack { cmd } => match cmd {
            AttackCommand::Labels => {
                println!(
                    "{}",
                    serde_json::json!({"mode":"sim", "attack":"labels", "status":"placeholder"})
                );
            }
        },
    }

    Ok(())
}

use discos_builder::{
    build_restricted_wasm, manifest_hash, AlphaHIRManifest, CausalDSLManifest, PhysHIRManifest,
};
use discos_client::{pb, DiscosClient};
use sha2::Digest;

fn manifest_entry(name: &str, bytes: Vec<u8>) -> pb::ArtifactManifest {
    pb::ArtifactManifest {
        name: name.to_string(),
        digest: sha2::Sha256::digest(&bytes).to_vec(),
        canonical_bytes: bytes,
    }
}

#[tokio::test]
#[ignore = "requires running evidenceos-daemon; set EVIDENCEOS_DAEMON_ADDR to enable"]
async fn builder_generated_wasm_commits_executes_and_fetches_capsule() -> anyhow::Result<()> {
    let endpoint = std::env::var("EVIDENCEOS_DAEMON_ADDR")
        .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
    let mut client = match DiscosClient::connect(&endpoint).await {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };

    let wasm = build_restricted_wasm();
    let alpha = AlphaHIRManifest {
        plan_id: "builder-system".to_string(),
        code_hash_hex: hex::encode(wasm.code_hash),
        oracle_kinds: vec!["oracle_query".to_string()],
        output_schema_id: "cbrn-sc.v1".to_string(),
        nullspec_id: "nullspec.v1".to_string(),
    };
    let phys = PhysHIRManifest {
        physical_signature_hash: hex::encode(manifest_hash(&alpha)?),
        envelope_ids: vec!["env/default".to_string()],
    };
    let causal = CausalDSLManifest {
        dag_hash: hex::encode(manifest_hash(&phys)?),
        adjustment_sets: vec![vec!["baseline".to_string()]],
    };

    let create = client
        .create_claim_v2(pb::CreateClaimV2Request {
            claim_name: "builder-system".to_string(),
            metadata: Some(pb::ClaimMetadataV2 {
                lane: "high_assurance".to_string(),
                alpha_micros: 50000,
                epoch_config_ref: "epoch/default".to_string(),
                output_schema_id: "cbrn-sc.v1".to_string(),
            }),
            signals: Some(pb::TopicSignalsV2 {
                semantic_hash: [1u8; 32].to_vec(),
                phys_hir_signature_hash: [2u8; 32].to_vec(),
                dependency_merkle_root: [3u8; 32].to_vec(),
            }),
            holdout_ref: "holdout/default".to_string(),
            epoch_size: 1,
            oracle_num_symbols: 4,
            access_credit: 1,
            oracle_id: "default".into(),
        })
        .await?;

    client
        .commit_artifacts(pb::CommitArtifactsRequest {
            claim_id: create.claim_id.clone(),
            wasm_hash: wasm.code_hash.to_vec(),
            wasm_module: wasm.wasm_bytes,
            manifests: vec![
                manifest_entry("alpha_hir.json", serde_json::to_vec(&alpha)?),
                manifest_entry("phys_hir.json", serde_json::to_vec(&phys)?),
                manifest_entry("causal_dsl.json", serde_json::to_vec(&causal)?),
            ],
        })
        .await?;

    client
        .freeze_gates(pb::FreezeGatesRequest {
            claim_id: create.claim_id.clone(),
        })
        .await?;
    client
        .execute_claim_v2(pb::ExecuteClaimV2Request {
            claim_id: create.claim_id.clone(),
        })
        .await?;
    client
        .seal_claim(pb::SealClaimRequest {
            claim_id: create.claim_id.clone(),
        })
        .await?;
    let capsule = client
        .fetch_capsule(pb::FetchCapsuleRequest {
            claim_id: create.claim_id,
        })
        .await?;

    anyhow::ensure!(!capsule.capsule.is_empty(), "capsule payload is empty");
    Ok(())
}

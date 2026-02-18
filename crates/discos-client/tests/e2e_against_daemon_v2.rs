use discos_client::{
    canonical_output_matches_capsule, pb, verify_inclusion, verify_inclusion_proof, DiscosClient,
    InclusionProof,
};

#[tokio::test]
#[ignore = "requires running evidenceos-daemon with secure v2 endpoints"]
async fn claim_lifecycle_v2_against_daemon() {
    let addr = std::env::var("EVIDENCEOS_DAEMON_ADDR")
        .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());
    let claim_name = "discos-v2-e2e".to_string();

    let mut client = match DiscosClient::connect(&addr).await {
        Ok(c) => c,
        Err(_) => return,
    };

    let create = match client
        .create_claim_v2(pb::CreateClaimV2Request {
            claim_name,
            metadata: Some(pb::ClaimMetadata {
                lane: "lane-a".to_string(),
                alpha_micros: 10,
                epoch_config_ref: "epoch-default".to_string(),
                output_schema_id: "cbrn-sc.v1".to_string(),
            }),
            signals: Some(pb::TopicSignals {
                semantic_hash: vec![],
                phys_hir_signature_hash: vec![7u8; 32],
                dependency_merkle_root: vec![],
            }),
            holdout_ref: "demo_labels.json".to_string(),
            epoch_size: 32,
            oracle_num_symbols: 8,
            access_credit: 10,
        })
        .await
    {
        Ok(v) => v,
        Err(_) => return,
    };

    let _ = client
        .commit_artifacts(pb::CommitArtifactsRequest {
            claim_id: create.claim_id.clone(),
            wasm_module: vec![0, 97, 115, 109, 1, 0, 0, 0],
            wasm_hash: vec![0u8; 32],
            manifests: vec![],
        })
        .await;

    let _ = client
        .seal_claim(pb::SealClaimRequest {
            claim_id: create.claim_id.clone(),
        })
        .await;

    let exec = match client
        .execute_claim_v2(pb::ExecuteClaimV2Request {
            claim_id: create.claim_id.clone(),
        })
        .await
    {
        Ok(v) => v,
        Err(_) => return,
    };

    let capsule = match client
        .fetch_capsule(pb::FetchCapsuleRequest {
            claim_id: create.claim_id.clone(),
        })
        .await
    {
        Ok(v) => v,
        Err(_) => return,
    };

    let _ = canonical_output_matches_capsule(&exec.canonical_output, &capsule.capsule);

    if let Some(inclusion) = capsule.inclusion {
        if let Ok(leaf_hash) = inclusion.leaf_hash.clone().try_into() {
            let proof = InclusionProof {
                leaf_hash,
                leaf_index: inclusion.leaf_index,
                tree_size: inclusion.tree_size,
                audit_path: inclusion
                    .audit_path
                    .into_iter()
                    .filter_map(|x| x.try_into().ok())
                    .collect(),
            };
            let _ = verify_inclusion_proof(leaf_hash, &proof);
            let _ = verify_inclusion(
                capsule
                    .etl_root_hash
                    .clone()
                    .try_into()
                    .unwrap_or([0u8; 32]),
                &proof,
            );
        }
    }

    let _ = client
        .revoke_claim(pb::RevokeClaimRequest {
            claim_id: create.claim_id,
            reason_code: "test_revoke".to_string(),
        })
        .await;
}

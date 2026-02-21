use std::{collections::HashMap, sync::Arc};

use discos_builder::{
    build_restricted_wasm, manifest_hash, AlphaHIRManifest, CausalDSLManifest, PhysHIRManifest,
};
use discos_client::{pb, DiscosClient};
use ed25519_dalek::{Signer, SigningKey};
use evidenceos_core::wasm_aspec::verify_restricted_wasm;
use evidenceos_verifier as verifier;
use sha2::Digest;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Request, Response, Status};

fn manifest_entry(name: &str, bytes: Vec<u8>) -> pb::ArtifactManifest {
    pb::ArtifactManifest {
        name: name.to_string(),
        digest: sha2::Sha256::digest(&bytes).to_vec(),
        canonical_bytes: bytes,
    }
}

#[derive(Clone)]
struct State {
    claims: HashMap<Vec<u8>, Vec<u8>>,
    signing_key: SigningKey,
    root: [u8; 32],
    sig: [u8; 64],
}

#[derive(Clone)]
struct TestDaemon(Arc<Mutex<State>>);

#[tonic::async_trait]
impl pb::evidence_os_server::EvidenceOs for TestDaemon {
    async fn health(
        &self,
        _: Request<pb::HealthRequest>,
    ) -> Result<Response<pb::HealthResponse>, Status> {
        Ok(Response::new(pb::HealthResponse {
            status: "ok".into(),
        }))
    }

    async fn create_claim_v2(
        &self,
        req: Request<pb::CreateClaimV2Request>,
    ) -> Result<Response<pb::CreateClaimV2Response>, Status> {
        let r = req.into_inner();
        let claim_id = sha2::Sha256::digest(r.claim_name.as_bytes()).to_vec();
        let topic_id = sha2::Sha256::digest(b"topic").to_vec();
        self.0.lock().await.claims.insert(claim_id.clone(), vec![]);
        Ok(Response::new(pb::CreateClaimV2Response {
            claim_id,
            topic_id,
        }))
    }

    async fn commit_artifacts(
        &self,
        _: Request<pb::CommitArtifactsRequest>,
    ) -> Result<Response<pb::CommitArtifactsResponse>, Status> {
        Ok(Response::new(pb::CommitArtifactsResponse {
            accepted: true,
        }))
    }

    async fn commit_wasm(
        &self,
        _: Request<pb::CommitWasmRequest>,
    ) -> Result<Response<pb::CommitWasmResponse>, Status> {
        Ok(Response::new(pb::CommitWasmResponse { accepted: true }))
    }

    async fn freeze(
        &self,
        _: Request<pb::FreezeRequest>,
    ) -> Result<Response<pb::FreezeResponse>, Status> {
        Ok(Response::new(pb::FreezeResponse { frozen: true }))
    }

    async fn execute_claim_v2(
        &self,
        _: Request<pb::ExecuteClaimV2Request>,
    ) -> Result<Response<pb::ExecuteClaimV2Response>, Status> {
        Ok(Response::new(pb::ExecuteClaimV2Response {
            certified: true,
            e_value: 1.0,
            canonical_output: b"{}".to_vec(),
        }))
    }

    async fn fetch_capsule(
        &self,
        req: Request<pb::FetchCapsuleRequest>,
    ) -> Result<Response<pb::FetchCapsuleResponse>, Status> {
        let claim_id = req.into_inner().claim_id;
        let capsule = serde_json::to_vec(&serde_json::json!({
            "claim_id_hex": hex::encode(&claim_id),
            "topic_id_hex": hex::encode([2u8;32]),
            "structured_output_hash_hex": hex::encode(sha2::Sha256::digest(b"{}")),
        }))
        .map_err(|e| Status::internal(e.to_string()))?;
        let leaf = verifier::etl_leaf_hash(&capsule);
        let mut st = self.0.lock().await;
        st.root = leaf;
        st.sig = st
            .signing_key
            .sign(&verifier::sth_signature_digest(1, st.root))
            .to_bytes();

        Ok(Response::new(pb::FetchCapsuleResponse {
            claim_id,
            capsule,
            etl_index: 0,
            etl_tree_size: 1,
            etl_root_hash: st.root.to_vec(),
            sth_signature: st.sig.to_vec(),
            inclusion: Some(pb::MerkleInclusionProof {
                leaf_hash: leaf.to_vec(),
                leaf_index: 0,
                tree_size: 1,
                audit_path: vec![],
            }),
            consistency: Some(pb::MerkleConsistencyProof {
                old_tree_size: 1,
                new_tree_size: 1,
                path: vec![],
            }),
        }))
    }

    async fn get_public_key(
        &self,
        _: Request<pb::GetPublicKeyRequest>,
    ) -> Result<Response<pb::GetPublicKeyResponse>, Status> {
        let st = self.0.lock().await;
        Ok(Response::new(pb::GetPublicKeyResponse {
            pubkey: st.signing_key.verifying_key().to_bytes().to_vec(),
            key_id: "test-key".into(),
        }))
    }

    async fn get_signed_tree_head(
        &self,
        _: Request<pb::GetSignedTreeHeadRequest>,
    ) -> Result<Response<pb::GetSignedTreeHeadResponse>, Status> {
        let st = self.0.lock().await;
        Ok(Response::new(pb::GetSignedTreeHeadResponse {
            sth: Some(pb::SignedTreeHead {
                tree_size: 1,
                root_hash: st.root.to_vec(),
                signature: st.sig.to_vec(),
            }),
        }))
    }

    async fn get_inclusion_proof(
        &self,
        _: Request<pb::GetInclusionProofRequest>,
    ) -> Result<Response<pb::GetInclusionProofResponse>, Status> {
        Ok(Response::new(pb::GetInclusionProofResponse {
            proof: Some(pb::MerkleInclusionProof {
                leaf_hash: [3u8; 32].to_vec(),
                leaf_index: 0,
                tree_size: 1,
                audit_path: vec![],
            }),
        }))
    }

    async fn get_consistency_proof(
        &self,
        _: Request<pb::GetConsistencyProofRequest>,
    ) -> Result<Response<pb::GetConsistencyProofResponse>, Status> {
        Ok(Response::new(pb::GetConsistencyProofResponse {
            proof: Some(pb::MerkleConsistencyProof {
                old_tree_size: 1,
                new_tree_size: 1,
                path: vec![],
            }),
        }))
    }

    async fn revoke_claim(
        &self,
        _: Request<pb::RevokeClaimRequest>,
    ) -> Result<Response<pb::RevokeClaimResponse>, Status> {
        Ok(Response::new(pb::RevokeClaimResponse { revoked: true }))
    }

    async fn get_server_info(
        &self,
        _: Request<pb::GetServerInfoRequest>,
    ) -> Result<Response<pb::GetServerInfoResponse>, Status> {
        Ok(Response::new(pb::GetServerInfoResponse {
            protocol_semver: evidenceos_protocol::PROTOCOL_SEMVER.into(),
            proto_hash: evidenceos_protocol::PROTO_SHA256.into(),
            build_git_commit: "test".into(),
            build_time_utc: "2026-01-01T00:00:00Z".into(),
            daemon_version: "evidenceosd/test".into(),
            feature_flags: vec!["tls_enabled".into()],
        }))
    }

    type WatchRevocationsStream = tokio_stream::Empty<Result<pb::RevocationEvent, Status>>;
    async fn watch_revocations(
        &self,
        _: Request<pb::WatchRevocationsRequest>,
    ) -> Result<Response<Self::WatchRevocationsStream>, Status> {
        Ok(Response::new(tokio_stream::empty()))
    }
}

#[tokio::test]
async fn builder_generated_wasm_passes_aspec_and_executes_in_daemon() -> anyhow::Result<()> {
    let wasm = build_restricted_wasm();
    verify_restricted_wasm(&wasm.wasm_bytes)?;

    let key = SigningKey::from_bytes(&[9u8; 32]);
    let daemon = TestDaemon(Arc::new(Mutex::new(State {
        claims: HashMap::new(),
        signing_key: key,
        root: [0; 32],
        sig: [0; 64],
    })));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        Server::builder()
            .add_service(pb::evidence_os_server::EvidenceOsServer::new(daemon))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("server exited cleanly")
    });

    let mut client = DiscosClient::connect(&format!("http://{addr}")).await?;

    let alpha = AlphaHIRManifest {
        plan_id: "builder-system".to_string(),
        code_hash_hex: hex::encode(wasm.code_hash),
        oracle_kinds: vec![discos_builder::VAULT_IMPORT_ORACLE_QUERY.to_string()],
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
            manifests: vec![
                manifest_entry(
                    "alpha_hir.json",
                    discos_builder::canonical_json(&alpha)?.into_bytes(),
                ),
                manifest_entry(
                    "phys_hir.json",
                    discos_builder::canonical_json(&phys)?.into_bytes(),
                ),
                manifest_entry(
                    "causal_dsl.json",
                    discos_builder::canonical_json(&causal)?.into_bytes(),
                ),
            ],
        })
        .await?;

    client
        .commit_wasm(pb::CommitWasmRequest {
            claim_id: create.claim_id.clone(),
            wasm_hash: wasm.code_hash.to_vec(),
            wasm_module: wasm.wasm_bytes,
        })
        .await?;

    client
        .freeze(pb::FreezeRequest {
            claim_id: create.claim_id.clone(),
        })
        .await?;

    let execute = client
        .execute_claim_v2(pb::ExecuteClaimV2Request {
            claim_id: create.claim_id.clone(),
        })
        .await?;
    anyhow::ensure!(execute.certified, "execution was not certified");

    let capsule = client
        .fetch_capsule(pb::FetchCapsuleRequest {
            claim_id: create.claim_id,
        })
        .await?;

    anyhow::ensure!(!capsule.capsule.is_empty(), "capsule payload is empty");
    Ok(())
}

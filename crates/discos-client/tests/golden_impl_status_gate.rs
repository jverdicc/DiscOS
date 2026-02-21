use std::{collections::HashMap, sync::Arc};

use discos_client::{pb, verify_capsule_response, DiscosClient, SignedTreeHead};
use ed25519_dalek::{Signer, SigningKey};
use evidenceos_verifier as verifier;
use evidenceos_verifier::{revocation_entry_digest, verify_revocations_snapshot, RevocationEntry};
use sha2::Digest;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Clone)]
struct State {
    claims: HashMap<Vec<u8>, Vec<u8>>,
    signing_key: SigningKey,
    root: [u8; 32],
    sig: [u8; 64],
}

#[derive(Clone)]
struct GoldenDaemon(Arc<Mutex<State>>);

#[tonic::async_trait]
impl pb::evidence_os_server::EvidenceOs for GoldenDaemon {
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
        let topic_id = [2u8; 32].to_vec();
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
            key_id: "k1".into(),
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
        Err(Status::unimplemented("not needed"))
    }

    async fn get_consistency_proof(
        &self,
        _: Request<pb::GetConsistencyProofRequest>,
    ) -> Result<Response<pb::GetConsistencyProofResponse>, Status> {
        Err(Status::unimplemented("not needed"))
    }

    async fn revoke_claim(
        &self,
        _: Request<pb::RevokeClaimRequest>,
    ) -> Result<Response<pb::RevokeClaimResponse>, Status> {
        Ok(Response::new(pb::RevokeClaimResponse { revoked: true }))
    }

    type WatchRevocationsStream = tokio_stream::Empty<Result<pb::RevocationEvent, Status>>;

    async fn watch_revocations(
        &self,
        _: Request<pb::WatchRevocationsRequest>,
    ) -> Result<Response<Self::WatchRevocationsStream>, Status> {
        Ok(Response::new(tokio_stream::empty()))
    }

    async fn get_server_info(
        &self,
        _: Request<pb::GetServerInfoRequest>,
    ) -> Result<Response<pb::GetServerInfoResponse>, Status> {
        Ok(Response::new(pb::GetServerInfoResponse {
            protocol_semver: evidenceos_protocol::PROTOCOL_SEMVER.into(),
            proto_hash: evidenceos_protocol::PROTO_SHA256.into(),
            build_git_commit: "4c1d7f2b0adf337df75fc85d4b7d84df4e99d0af".into(),
            build_time_utc: "2026-01-01T00:00:00Z".into(),
            daemon_version: "evidenceosd/2.1.0".into(),
            feature_flags: vec!["tls_enabled".into()],
        }))
    }
}

#[tokio::test]
async fn golden_status_gate_integration_protocol_capsule_and_revocations() {
    let key = SigningKey::from_bytes(&[9u8; 32]);
    let daemon = GoldenDaemon(Arc::new(Mutex::new(State {
        claims: HashMap::new(),
        signing_key: key,
        root: [0; 32],
        sig: [0; 64],
    })));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("read local addr");

    tokio::spawn(async move {
        Server::builder()
            .add_service(pb::evidence_os_server::EvidenceOsServer::new(daemon))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("server exited cleanly")
    });

    let mut client = DiscosClient::connect(&format!("http://{addr}"))
        .await
        .expect("connect client");

    let info = client.get_server_info().await.expect("server info");
    assert_eq!(info.protocol_semver, evidenceos_protocol::PROTOCOL_SEMVER);

    let created = client
        .create_claim_v2(pb::CreateClaimV2Request {
            claim_name: "golden".into(),
            metadata: None,
            signals: None,
            holdout_ref: "holdout/default".into(),
            epoch_size: 1,
            oracle_num_symbols: 1,
            access_credit: 1,
            oracle_id: "default".into(),
        })
        .await
        .expect("create claim");

    let fetched = client
        .fetch_capsule(pb::FetchCapsuleRequest {
            claim_id: created.claim_id.clone(),
        })
        .await
        .expect("fetch capsule");

    let key = client.get_public_key().await.expect("get public key");
    verify_capsule_response(
        &fetched,
        b"{}",
        &created.claim_id,
        &created.topic_id,
        &key.pubkey,
        None,
    )
    .expect("capsule verification");

    let sth = SignedTreeHead {
        tree_size: fetched.etl_tree_size,
        root_hash: fetched
            .etl_root_hash
            .as_slice()
            .try_into()
            .expect("32-byte root hash"),
        signature: fetched
            .sth_signature
            .as_slice()
            .try_into()
            .expect("64-byte signature"),
    };

    let mut rev_entry = RevocationEntry {
        claim_id: created.claim_id,
        reason_code: "policy_violation".to_string(),
        logical_epoch: 7,
        signature: [0u8; 64],
    };
    let signer = SigningKey::from_bytes(&[11u8; 32]);
    rev_entry.signature = signer.sign(&revocation_entry_digest(&rev_entry)).to_bytes();

    let entries = vec![rev_entry.clone()];
    let digest = verifier::revocations_snapshot_digest(
        &entries,
        &verifier::SignedTreeHead {
            tree_size: sth.tree_size,
            root_hash: sth.root_hash,
            signature: sth.signature,
        },
    );

    assert!(verify_revocations_snapshot(
        &entries,
        &verifier::SignedTreeHead {
            tree_size: sth.tree_size,
            root_hash: sth.root_hash,
            signature: sth.signature,
        },
        digest,
    ));

    let mut tampered = entries;
    tampered[0].logical_epoch += 1;
    assert!(!verify_revocations_snapshot(
        &tampered,
        &verifier::SignedTreeHead {
            tree_size: sth.tree_size,
            root_hash: sth.root_hash,
            signature: sth.signature,
        },
        digest,
    ));
}

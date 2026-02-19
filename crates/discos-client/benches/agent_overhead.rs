use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use discos_client::{pb, DiscosClient};
use ed25519_dalek::{Signer, SigningKey};
use serde::Serialize;
use sha2::Digest;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Clone)]
struct BenchDaemon {
    signing_key: SigningKey,
    topic_id: Vec<u8>,
}

#[tonic::async_trait]
impl pb::evidence_os_server::EvidenceOs for BenchDaemon {
    async fn health(
        &self,
        _: Request<pb::HealthRequest>,
    ) -> Result<Response<pb::HealthResponse>, Status> {
        Ok(Response::new(pb::HealthResponse {
            status: "ok".to_string(),
        }))
    }

    async fn create_claim(
        &self,
        _: Request<pb::CreateClaimRequest>,
    ) -> Result<Response<pb::CreateClaimResponse>, Status> {
        Err(Status::unimplemented("v2 only"))
    }

    async fn create_claim_v2(
        &self,
        req: Request<pb::CreateClaimV2Request>,
    ) -> Result<Response<pb::CreateClaimV2Response>, Status> {
        let claim_name = req.into_inner().claim_name;
        let claim_id = sha2::Sha256::digest(claim_name.as_bytes()).to_vec();
        Ok(Response::new(pb::CreateClaimV2Response {
            claim_id,
            topic_id: self.topic_id.clone(),
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

    async fn freeze_gates(
        &self,
        _: Request<pb::FreezeGatesRequest>,
    ) -> Result<Response<pb::FreezeGatesResponse>, Status> {
        Ok(Response::new(pb::FreezeGatesResponse { frozen: true }))
    }

    async fn seal_claim(
        &self,
        _: Request<pb::SealClaimRequest>,
    ) -> Result<Response<pb::SealClaimResponse>, Status> {
        Ok(Response::new(pb::SealClaimResponse { sealed: true }))
    }

    async fn execute_claim(
        &self,
        _: Request<pb::ExecuteClaimRequest>,
    ) -> Result<Response<pb::ExecuteClaimResponse>, Status> {
        Err(Status::unimplemented("v2 only"))
    }

    async fn execute_claim_v2(
        &self,
        _: Request<pb::ExecuteClaimV2Request>,
    ) -> Result<Response<pb::ExecuteClaimV2Response>, Status> {
        let canonical_output = br#"{"agent":"ok","score":7}"#.to_vec();
        Ok(Response::new(pb::ExecuteClaimV2Response {
            certified: true,
            e_value: 1.0,
            canonical_output,
        }))
    }

    async fn fetch_capsule(
        &self,
        req: Request<pb::FetchCapsuleRequest>,
    ) -> Result<Response<pb::FetchCapsuleResponse>, Status> {
        let claim_id = req.into_inner().claim_id;
        let canonical_output = br#"{"agent":"ok","score":7}"#;
        let capsule = serde_json::to_vec(&serde_json::json!({
            "claim_id_hex": hex::encode(&claim_id),
            "topic_id_hex": hex::encode(&self.topic_id),
            "structured_output_hash_hex": hex::encode(sha2::Sha256::digest(canonical_output)),
        }))
        .map_err(|e| Status::internal(e.to_string()))?;

        let leaf_hash = [9u8; 32];
        let mut signature_payload = Vec::new();
        signature_payload.extend_from_slice(&1u64.to_be_bytes());
        signature_payload.extend_from_slice(&leaf_hash);
        let sth_signature = self
            .signing_key
            .sign(&discos_client::sha256_domain(
                evidenceos_protocol::domains::STH_SIGNATURE_V1,
                &signature_payload,
            ))
            .to_bytes()
            .to_vec();

        Ok(Response::new(pb::FetchCapsuleResponse {
            claim_id,
            capsule,
            etl_index: 0,
            etl_tree_size: 1,
            etl_root_hash: leaf_hash.to_vec(),
            sth_signature,
            inclusion: Some(pb::MerkleInclusionProof {
                leaf_hash: leaf_hash.to_vec(),
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
        Ok(Response::new(pb::GetPublicKeyResponse {
            pubkey: self.signing_key.verifying_key().to_bytes().to_vec(),
            key_id: "bench-k1".to_string(),
        }))
    }

    async fn get_signed_tree_head(
        &self,
        _: Request<pb::GetSignedTreeHeadRequest>,
    ) -> Result<Response<pb::GetSignedTreeHeadResponse>, Status> {
        let root_hash = [9u8; 32];
        let mut signature_payload = Vec::new();
        signature_payload.extend_from_slice(&1u64.to_be_bytes());
        signature_payload.extend_from_slice(&root_hash);
        let signature = self
            .signing_key
            .sign(&discos_client::sha256_domain(
                evidenceos_protocol::domains::STH_SIGNATURE_V1,
                &signature_payload,
            ))
            .to_bytes()
            .to_vec();

        Ok(Response::new(pb::GetSignedTreeHeadResponse {
            sth: Some(pb::SignedTreeHead {
                tree_size: 1,
                root_hash: root_hash.to_vec(),
                signature,
            }),
        }))
    }

    async fn get_inclusion_proof(
        &self,
        _: Request<pb::GetInclusionProofRequest>,
    ) -> Result<Response<pb::GetInclusionProofResponse>, Status> {
        Ok(Response::new(pb::GetInclusionProofResponse {
            proof: Some(pb::MerkleInclusionProof {
                leaf_hash: [9u8; 32].to_vec(),
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
            proto_hash: "test-proto-hash".into(),
            protocol_package: "evidenceos.v1".into(),
            git_commit: "4c1d7f2b0adf337df75fc85d4b7d84df4e99d0af".into(),
            build_timestamp: "2026-01-01T00:00:00Z".into(),
            key_ids: vec!["k1".into()],
            compatibility_min_rev: "4c1d7f2b0adf337df75fc85d4b7d84df4e99d0af".into(),
            compatibility_max_rev: "4c1d7f2b0adf337df75fc85d4b7d84df4e99d0af".into(),
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

#[derive(Clone)]
struct BenchContext {
    runtime: Arc<Runtime>,
    client: Arc<Mutex<DiscosClient>>,
    claim_id: Vec<u8>,
    topic_id: Vec<u8>,
    canonical_output: Vec<u8>,
}

impl BenchContext {
    fn new() -> Self {
        let runtime = Arc::new(Runtime::new().expect("create tokio runtime"));
        let key = SigningKey::from_bytes(&[11u8; 32]);
        let topic_id = sha2::Sha256::digest(b"topic").to_vec();
        let daemon = BenchDaemon {
            signing_key: key,
            topic_id: topic_id.clone(),
        };

        let (client, claim_id) = runtime.block_on(async move {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind benchmark listener");
            let addr = listener
                .local_addr()
                .expect("read benchmark listener address");
            tokio::spawn(async move {
                let incoming = TcpListenerStream::new(listener);
                Server::builder()
                    .add_service(pb::evidence_os_server::EvidenceOsServer::new(daemon))
                    .serve_with_incoming(incoming)
                    .await
                    .expect("benchmark daemon should run until process exit")
            });

            let mut client = DiscosClient::connect(&format!("http://{addr}"))
                .await
                .expect("connect benchmark client");
            let created = client
                .create_claim_v2(pb::CreateClaimV2Request {
                    claim_name: "bench-claim".to_string(),
                    metadata: None,
                    signals: None,
                    holdout_ref: "bench-holdout".to_string(),
                    epoch_size: 1,
                    oracle_num_symbols: 1,
                    access_credit: 1,
                    oracle_id: "default".to_string(),
                })
                .await
                .expect("create benchmark claim");

            (client, created.claim_id)
        });

        Self {
            runtime,
            client: Arc::new(Mutex::new(client)),
            claim_id,
            topic_id,
            canonical_output: br#"{"agent":"ok","score":7}"#.to_vec(),
        }
    }

    fn verified_execution_round_trip(&self) {
        self.runtime.block_on(async {
            let mut client = self.client.lock().await;
            let execution = client
                .execute_claim_v2(pb::ExecuteClaimV2Request {
                    claim_id: self.claim_id.clone(),
                })
                .await
                .expect("execute claim v2");

            let fetched = client
                .fetch_capsule(pb::FetchCapsuleRequest {
                    claim_id: self.claim_id.clone(),
                })
                .await
                .expect("fetch capsule");

            discos_client::canonical_output_matches_capsule(
                &execution.canonical_output,
                &fetched.capsule,
                &self.claim_id,
                &self.topic_id,
            )
            .expect("verify fetched capsule");
        });
    }

    fn direct_execution(&self) {
        black_box(simulated_agent_execution(&self.canonical_output));
    }
}

#[derive(Clone)]
struct RawAgentState {
    claim_id: [u8; 32],
    topic_id: [u8; 32],
    canonical_output: Vec<u8>,
}

#[derive(Serialize)]
struct ClaimCapsule {
    structured_output_hash_hex: String,
    claim_id_hex: String,
    topic_id_hex: String,
}

fn build_claim_capsule(state: &RawAgentState) -> ClaimCapsule {
    ClaimCapsule {
        structured_output_hash_hex: hex::encode(sha2::Sha256::digest(&state.canonical_output)),
        claim_id_hex: hex::encode(state.claim_id),
        topic_id_hex: hex::encode(state.topic_id),
    }
}

fn simulated_agent_execution(input: &[u8]) -> [u8; 32] {
    discos_client::sha256(input)
}

fn bench_interaction_tax(c: &mut Criterion) {
    let context = BenchContext::new();
    let mut group = c.benchmark_group("agent_overhead");

    group.bench_function("interaction_tax_verified_round_trip", |b| {
        b.iter(|| context.verified_execution_round_trip())
    });

    group.bench_function("comparison_direct_execution", |b| {
        b.iter(|| context.direct_execution())
    });

    group.bench_function("comparison_verified_execution", |b| {
        b.iter(|| context.verified_execution_round_trip())
    });

    let serialization_state_sizes = [512usize, 4 * 1024usize, 64 * 1024usize];
    for state_size in serialization_state_sizes {
        let state = RawAgentState {
            claim_id: [1u8; 32],
            topic_id: [2u8; 32],
            canonical_output: vec![42u8; state_size],
        };

        group.bench_with_input(
            BenchmarkId::new("state_serialization_cost_claim_capsule", state_size),
            &state,
            |b, s| {
                b.iter(|| {
                    let capsule = build_claim_capsule(s);
                    black_box(serde_json::to_vec(&capsule).expect("serialize claim capsule"));
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_interaction_tax);
criterion_main!(benches);

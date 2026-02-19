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

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Code, Request, Response, Status};

use discos_client::pb;

const N_CLIENTS: usize = 25;
const K_BITS_TOTAL: f64 = 30.0;
const K_BITS_PER_ATTEMPT: f64 = 2.0;
const RNG_SEED: u64 = 0xD15C0;
const ARTIFACT_PATH: &str = "artifacts/system-test/operation_swarm_results.json";

#[derive(Clone, Debug)]
struct AttemptResult {
    identity: String,
    operation_id: String,
    k_bits_remaining: f64,
    lane: String,
    state: String,
    attempt_number: usize,
}

#[derive(Default)]
struct TopicState {
    k_bits_spent: f64,
    attempts: usize,
}

#[derive(Default)]
struct DaemonState {
    claims: HashMap<Vec<u8>, Vec<u8>>,
    topics: HashMap<Vec<u8>, TopicState>,
}

#[derive(Clone)]
struct SwarmDaemon(Arc<Mutex<DaemonState>>);

#[derive(Debug, Serialize, Deserialize)]
struct SystemArtifact {
    n_clients: usize,
    n_total_attempts: usize,
    first_escalation_at_attempt: Option<usize>,
    final_state: String,
    final_k_remaining: f64,
}

#[tonic::async_trait]
impl pb::evidence_os_server::EvidenceOs for SwarmDaemon {
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
        let identity = req
            .metadata()
            .get("x-identity")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let req_inner = req.into_inner();
        let signals = req_inner
            .signals
            .ok_or_else(|| Status::invalid_argument("signals missing"))?;

        let mut topic_hasher = sha2::Sha256::new();
        topic_hasher.update(signals.semantic_hash);
        topic_hasher.update(signals.phys_hir_signature_hash);
        topic_hasher.update(signals.dependency_merkle_root);
        let topic_id = topic_hasher.finalize().to_vec();

        let mut claim_hasher = sha2::Sha256::new();
        claim_hasher.update(req_inner.claim_name.as_bytes());
        claim_hasher.update(identity.as_bytes());
        let claim_id = claim_hasher.finalize().to_vec();

        let mut guard = self.0.lock().await;
        guard.claims.insert(claim_id.clone(), topic_id.clone());

        Ok(Response::new(pb::CreateClaimV2Response { claim_id, topic_id }))
    }

    async fn commit_artifacts(
        &self,
        _: Request<pb::CommitArtifactsRequest>,
    ) -> Result<Response<pb::CommitArtifactsResponse>, Status> {
        Ok(Response::new(pb::CommitArtifactsResponse { accepted: true }))
    }

    async fn freeze_gates(
        &self,
        _: Request<pb::FreezeGatesRequest>,
    ) -> Result<Response<pb::FreezeGatesResponse>, Status> {
        Ok(Response::new(pb::FreezeGatesResponse { frozen: false }))
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
        req: Request<pb::ExecuteClaimV2Request>,
    ) -> Result<Response<pb::ExecuteClaimV2Response>, Status> {
        let claim_id = req.into_inner().claim_id;
        let mut guard = self.0.lock().await;

        let topic_id = guard
            .claims
            .get(&claim_id)
            .cloned()
            .ok_or_else(|| Status::not_found("claim not found"))?;

        let topic_state = guard.topics.entry(topic_id.clone()).or_default();

        if topic_state.k_bits_spent + K_BITS_PER_ATTEMPT > K_BITS_TOTAL + f64::EPSILON {
            let operation_id = hex::encode(sha2::Sha256::digest(&topic_id));
            let payload = serde_json::json!({
                "operation_id": operation_id,
                "k_bits_remaining": (K_BITS_TOTAL - topic_state.k_bits_spent).max(0.0),
                "lane": "high_assurance",
                "state": "REJECT",
                "attempt_number": topic_state.attempts,
            });
            return Err(Status::new(Code::ResourceExhausted, payload.to_string()));
        }

        topic_state.k_bits_spent += K_BITS_PER_ATTEMPT;
        topic_state.attempts += 1;

        let k_bits_remaining = (K_BITS_TOTAL - topic_state.k_bits_spent).max(0.0);
        let lane = if topic_state.k_bits_spent >= 16.0 {
            "high_assurance"
        } else if topic_state.k_bits_spent >= 8.0 {
            "medium_assurance"
        } else {
            "base_assurance"
        };
        let state = if k_bits_remaining <= f64::EPSILON {
            "FROZEN"
        } else {
            "ACTIVE"
        };
        let operation_id = hex::encode(sha2::Sha256::digest(&topic_id));

        let canonical_output = serde_json::to_vec(&serde_json::json!({
            "operation_id": operation_id,
            "k_bits_remaining": k_bits_remaining,
            "lane": lane,
            "state": state,
            "attempt_number": topic_state.attempts,
        }))
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(pb::ExecuteClaimV2Response {
            certified: state != "REJECT",
            e_value: 1.0,
            canonical_output,
        }))
    }

    async fn fetch_capsule(
        &self,
        _: Request<pb::FetchCapsuleRequest>,
    ) -> Result<Response<pb::FetchCapsuleResponse>, Status> {
        Err(Status::unimplemented("not used in swarm test"))
    }

    async fn get_public_key(
        &self,
        _: Request<pb::GetPublicKeyRequest>,
    ) -> Result<Response<pb::GetPublicKeyResponse>, Status> {
        Err(Status::unimplemented("not used in swarm test"))
    }

    async fn get_signed_tree_head(
        &self,
        _: Request<pb::GetSignedTreeHeadRequest>,
    ) -> Result<Response<pb::GetSignedTreeHeadResponse>, Status> {
        Err(Status::unimplemented("not used in swarm test"))
    }

    async fn get_inclusion_proof(
        &self,
        _: Request<pb::GetInclusionProofRequest>,
    ) -> Result<Response<pb::GetInclusionProofResponse>, Status> {
        Err(Status::unimplemented("not used in swarm test"))
    }

    async fn get_consistency_proof(
        &self,
        _: Request<pb::GetConsistencyProofRequest>,
    ) -> Result<Response<pb::GetConsistencyProofResponse>, Status> {
        Err(Status::unimplemented("not used in swarm test"))
    }

    async fn revoke_claim(
        &self,
        _: Request<pb::RevokeClaimRequest>,
    ) -> Result<Response<pb::RevokeClaimResponse>, Status> {
        Err(Status::unimplemented("not used in swarm test"))
    }

    type WatchRevocationsStream = tokio_stream::Empty<Result<pb::RevocationEvent, Status>>;
    async fn watch_revocations(
        &self,
        _: Request<pb::WatchRevocationsRequest>,
    ) -> Result<Response<Self::WatchRevocationsStream>, Status> {
        Err(Status::unimplemented("not used in swarm test"))
    }

    async fn get_server_info(
        &self,
        _: Request<pb::GetServerInfoRequest>,
    ) -> Result<Response<pb::GetServerInfoResponse>, Status> {
        Ok(Response::new(pb::GetServerInfoResponse {
            proto_hash: "swarm-proto-hash".into(),
            protocol_package: "evidenceos.v1".into(),
            git_commit: "local".into(),
            build_timestamp: "local".into(),
            key_ids: vec!["local".into()],
            compatibility_min_rev: "local".into(),
            compatibility_max_rev: "local".into(),
        }))
    }
}

fn parse_attempt_from_json(identity: &str, body: &[u8]) -> AttemptResult {
    let payload: serde_json::Value =
        serde_json::from_slice(body).expect("daemon canonical payload must be JSON");
    AttemptResult {
        identity: identity.to_string(),
        operation_id: payload["operation_id"]
            .as_str()
            .expect("operation_id must exist")
            .to_string(),
        k_bits_remaining: payload["k_bits_remaining"]
            .as_f64()
            .expect("k_bits_remaining must exist"),
        lane: payload["lane"].as_str().expect("lane must exist").to_string(),
        state: payload["state"].as_str().expect("state must exist").to_string(),
        attempt_number: payload["attempt_number"]
            .as_u64()
            .expect("attempt_number must exist") as usize,
    }
}

fn parse_attempt_from_status(identity: &str, status: &Status) -> Option<AttemptResult> {
    if status.code() != Code::ResourceExhausted {
        return None;
    }
    Some(parse_attempt_from_json(identity, status.message().as_bytes()))
}

async fn wait_for_health(endpoint: &str) {
    for _ in 0..30 {
        if let Ok(mut client) =
            pb::evidence_os_client::EvidenceOsClient::connect(endpoint.to_string()).await
        {
            if client.health(pb::HealthRequest {}).await.is_ok() {
                return;
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    panic!("daemon did not become healthy in time");
}

#[tokio::test]
async fn operation_swarm_system_topic_budget_shared_and_stops() {
    if Path::new(ARTIFACT_PATH).exists() {
        std::fs::remove_file(ARTIFACT_PATH).expect("clear previous artifact");
    }
    std::fs::create_dir_all("artifacts/system-test").expect("create artifact directory");

    let daemon = SwarmDaemon(Arc::new(Mutex::new(DaemonState::default())));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind daemon listener");
    let addr = listener.local_addr().expect("resolve daemon listener addr");

    tokio::spawn(async move {
        Server::builder()
            .add_service(pb::evidence_os_server::EvidenceOsServer::new(daemon))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("daemon should run for test duration")
    });

    let endpoint = format!("http://{addr}");
    wait_for_health(&endpoint).await;

    let mut rng = ChaCha20Rng::seed_from_u64(RNG_SEED);
    let mut topic_input = [0u8; 32];
    rng.fill_bytes(&mut topic_input);

    let identities: Vec<String> = (0..N_CLIENTS).map(|i| format!("identity-{i:02}")) .collect();

    let mut tasks = Vec::with_capacity(N_CLIENTS);
    for identity in identities.clone() {
        let endpoint = endpoint.clone();
        let claim_nonce = rng.next_u64();
        let semantic_hash = topic_input.to_vec();
        tasks.push(tokio::spawn(async move {
            let mut client = pb::evidence_os_client::EvidenceOsClient::connect(endpoint)
                .await
                .expect("connect client");

            let mut create_req = Request::new(pb::CreateClaimV2Request {
                claim_name: format!("swarm-claim-{claim_nonce}"),
                metadata: Some(pb::ClaimMetadataV2 {
                    lane: "base_assurance".into(),
                    alpha_micros: 100,
                    epoch_config_ref: "swarm-fixed-epoch".into(),
                    output_schema_id: "swarm-test/v1".into(),
                }),
                signals: Some(pb::TopicSignalsV2 {
                    semantic_hash,
                    phys_hir_signature_hash: vec![7u8; 32],
                    dependency_merkle_root: vec![9u8; 32],
                }),
                holdout_ref: "swarm-holdout".into(),
                epoch_size: 16,
                oracle_num_symbols: 8,
                access_credit: 1,
                oracle_id: "default".into(),
            });
            let md = tonic::metadata::MetadataValue::try_from(identity.as_str())
                .expect("identity metadata should be valid");
            create_req.metadata_mut().insert("x-identity", md);
            let created = client
                .create_claim_v2(create_req)
                .await
                .expect("create_claim_v2 should succeed")
                .into_inner();

            let exec_result = client
                .execute_claim_v2(Request::new(pb::ExecuteClaimV2Request {
                    claim_id: created.claim_id,
                }))
                .await;

            match exec_result {
                Ok(resp) => parse_attempt_from_json(&identity, &resp.into_inner().canonical_output),
                Err(status) => parse_attempt_from_status(&identity, &status)
                    .expect("resource exhausted should return parseable JSON status"),
            }
        }));
    }

    let mut attempts: Vec<AttemptResult> = Vec::with_capacity(N_CLIENTS);
    for task in tasks {
        attempts.push(task.await.expect("join swarm task"));
    }

    attempts.sort_by_key(|a| a.attempt_number);

    let unique_operation_ids: BTreeSet<String> =
        attempts.iter().map(|a| a.operation_id.clone()).collect();
    assert_eq!(
        unique_operation_ids.len(),
        1,
        "the same operation_id must be shared across identities"
    );

    let k_remainings: Vec<f64> = attempts.iter().map(|a| a.k_bits_remaining).collect();
    let k_decreased = k_remainings.windows(2).any(|w| w[1] < w[0]);
    assert!(
        k_decreased,
        "aggregate k_bits_remaining must decrease across attempts"
    );

    let escalation_count = attempts
        .iter()
        .filter(|a| a.lane != "base_assurance")
        .count();
    let reject_count = attempts.iter().filter(|a| a.state == "REJECT").count();
    assert!(
        escalation_count > 0 || reject_count > 0,
        "must escalate lane or reject once budget is exhausted"
    );

    let final_attempt = attempts.last().expect("swarm attempt list should be non-empty");
    assert!(
        final_attempt.state == "REJECT" || final_attempt.state == "FROZEN",
        "final state must be REJECT or FROZEN once threshold is crossed"
    );

    let first_escalation_at_attempt = attempts
        .iter()
        .find(|a| a.lane != "base_assurance")
        .map(|a| a.attempt_number);

    let artifact = SystemArtifact {
        n_clients: N_CLIENTS,
        n_total_attempts: attempts.len(),
        first_escalation_at_attempt,
        final_state: final_attempt.state.clone(),
        final_k_remaining: final_attempt.k_bits_remaining,
    };

    let encoded =
        serde_json::to_vec_pretty(&artifact).expect("operation swarm artifact should serialize");
    std::fs::write(ARTIFACT_PATH, encoded).expect("write operation swarm artifact json");

    let identities_in_operation: BTreeSet<String> = attempts
        .iter()
        .filter(|a| a.operation_id == final_attempt.operation_id)
        .map(|a| a.identity.clone())
        .collect();
    assert!(
        identities_in_operation.len() > 1,
        "operation should aggregate more than one identity"
    );
}

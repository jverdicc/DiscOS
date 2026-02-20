use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use sha2::Digest;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Code, Request, Response, Status};

use discos_client::pb;

#[derive(Clone, Debug)]
struct ClaimCtx {
    topic_key: Vec<u8>,
    tool_name: String,
    agent_id: String,
    objective: String,
}

#[derive(Default)]
struct TopicState {
    duplicate_count: usize,
    tool_names: BTreeSet<String>,
    agent_ids: BTreeSet<String>,
}

#[derive(Default)]
struct DaemonState {
    claims: HashMap<Vec<u8>, ClaimCtx>,
    topics: HashMap<Vec<u8>, TopicState>,
}

#[derive(Clone)]
struct ScenarioMockDaemon(Arc<Mutex<DaemonState>>);

#[tonic::async_trait]
impl pb::evidence_os_server::EvidenceOs for ScenarioMockDaemon {
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
        let tool_name = req
            .metadata()
            .get("x-tool-name")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        let agent_id = req
            .metadata()
            .get("x-agent-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let inner = req.into_inner();
        let signals = inner
            .signals
            .ok_or_else(|| Status::invalid_argument("signals missing"))?;

        let objective = inner
            .claim_name
            .splitn(4, '-')
            .last()
            .unwrap_or("objective")
            .to_string();

        let mut topic_key = signals.semantic_hash;
        topic_key.extend_from_slice(&signals.phys_hir_signature_hash);
        topic_key.extend_from_slice(&signals.dependency_merkle_root);

        let claim_id = sha2::Sha256::digest(
            format!("{}|{}|{}", inner.claim_name, tool_name, agent_id).as_bytes(),
        )
        .to_vec();

        let mut guard = self.0.lock().await;
        guard.claims.insert(
            claim_id.clone(),
            ClaimCtx {
                topic_key: topic_key.clone(),
                tool_name,
                agent_id,
                objective,
            },
        );

        Ok(Response::new(pb::CreateClaimV2Response {
            claim_id,
            topic_id: sha2::Sha256::digest(&topic_key).to_vec(),
        }))
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
        req: Request<pb::ExecuteClaimV2Request>,
    ) -> Result<Response<pb::ExecuteClaimV2Response>, Status> {
        let claim_id = req.into_inner().claim_id;
        let mut guard = self.0.lock().await;

        let ctx = guard
            .claims
            .get(&claim_id)
            .cloned()
            .ok_or_else(|| Status::not_found("unknown claim"))?;

        let topic = guard.topics.entry(ctx.topic_key.clone()).or_default();
        topic.duplicate_count += 1;
        topic.tool_names.insert(ctx.tool_name.clone());
        topic.agent_ids.insert(ctx.agent_id.clone());

        if topic.tool_names.len() > 1 {
            return Err(Status::new(
                Code::FailedPrecondition,
                "ESCALATE shared-topic cross-tool differential probing",
            ));
        }

        if topic.agent_ids.len() > 1 {
            return Err(Status::new(
                Code::FailedPrecondition,
                "ESCALATE identity rotation with stable topic",
            ));
        }

        if topic.duplicate_count >= 4 {
            return Err(Status::new(Code::FailedPrecondition, "ESCALATE duplicate probing"));
        }

        if topic.duplicate_count >= 3 {
            return Err(Status::new(Code::ResourceExhausted, "THROTTLE duplicate probing"));
        }

        Ok(Response::new(pb::ExecuteClaimV2Response {
            certified: true,
            e_value: 1.0,
            canonical_output: serde_json::to_vec(&serde_json::json!({
                "status": "ALLOW",
                "objective": ctx.objective,
            }))
            .map_err(|e| Status::internal(e.to_string()))?,
        }))
    }

    async fn fetch_capsule(
        &self,
        _: Request<pb::FetchCapsuleRequest>,
    ) -> Result<Response<pb::FetchCapsuleResponse>, Status> {
        Err(Status::unimplemented("not needed"))
    }

    async fn get_public_key(
        &self,
        _: Request<pb::GetPublicKeyRequest>,
    ) -> Result<Response<pb::GetPublicKeyResponse>, Status> {
        Err(Status::unimplemented("not needed"))
    }

    async fn get_signed_tree_head(
        &self,
        _: Request<pb::GetSignedTreeHeadRequest>,
    ) -> Result<Response<pb::GetSignedTreeHeadResponse>, Status> {
        Err(Status::unimplemented("not needed"))
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
        Err(Status::unimplemented("not needed"))
    }

    type WatchRevocationsStream = tokio_stream::Empty<Result<pb::RevocationEvent, Status>>;

    async fn watch_revocations(
        &self,
        _: Request<pb::WatchRevocationsRequest>,
    ) -> Result<Response<Self::WatchRevocationsStream>, Status> {
        Err(Status::unimplemented("not needed"))
    }

    async fn get_server_info(
        &self,
        _: Request<pb::GetServerInfoRequest>,
    ) -> Result<Response<pb::GetServerInfoResponse>, Status> {
        Err(Status::unimplemented("not needed"))
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scenarios_suite_writes_artifacts_and_is_deterministic() {
    async fn spawn_daemon() -> (tokio::task::JoinHandle<()>, String) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind listener");
        let addr = listener.local_addr().expect("listener local addr");

        let daemon = ScenarioMockDaemon(Arc::new(Mutex::new(DaemonState::default())));
        let server = tokio::spawn(async move {
            Server::builder()
                .add_service(pb::evidence_os_server::EvidenceOsServer::new(daemon))
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await
                .expect("mock daemon should serve");
        });
        (server, format!("http://{addr}"))
    }

    let (first_server, first_endpoint) = spawn_daemon().await;
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root parent")
        .parent()
        .expect("repo root")
        .to_path_buf();

    let cmd = [
        "run",
        "-p",
        "discos-cli",
        "--",
        "--endpoint",
        &first_endpoint,
        "scenario",
        "run-suite",
    ];

    let first = Command::new("cargo")
        .current_dir(&repo_root)
        .args(cmd)
        .output()
        .expect("first scenario suite run");
    assert!(
        first.status.success(),
        "first run must succeed: {}",
        String::from_utf8_lossy(&first.stderr)
    );

    let first_value: serde_json::Value =
        serde_json::from_slice(&first.stdout).expect("first run stdout JSON");

    first_server.abort();
    let (second_server, second_endpoint) = spawn_daemon().await;

    let second = Command::new("cargo")
        .current_dir(&repo_root)
        .args([
            "run",
            "-p",
            "discos-cli",
            "--",
            "--endpoint",
            &second_endpoint,
            "scenario",
            "run-suite",
        ])
        .output()
        .expect("second scenario suite run");
    assert!(
        second.status.success(),
        "second run must succeed: {}",
        String::from_utf8_lossy(&second.stderr)
    );

    let second_value: serde_json::Value =
        serde_json::from_slice(&second.stdout).expect("second run stdout JSON");

    let runs = first_value["runs"].as_array().expect("runs array");
    assert!(!runs.is_empty(), "suite should return scenario runs");

    let saw_escalation_or_deny = runs.iter().any(|run| {
        run["final_state"] == "REQUIRE_HUMAN" || run["final_state"] == "DENY"
    });
    assert!(
        saw_escalation_or_deny,
        "at least one scenario should escalate to REQUIRE_HUMAN or DENY"
    );

    let first_fingerprints: Vec<_> = runs
        .iter()
        .map(|run| run["deterministic_fingerprint"].clone())
        .collect();
    let second_fingerprints: Vec<_> = second_value["runs"]
        .as_array()
        .expect("second runs array")
        .iter()
        .map(|run| run["deterministic_fingerprint"].clone())
        .collect();

    assert_eq!(
        first_fingerprints, second_fingerprints,
        "fixed seeds and identical inputs should be deterministic"
    );

    for scenario in [
        "rapid-repeated-tool-calls",
        "cross-tool-differential-probing",
        "identity-rotation-stable-topic",
        "benign-mixed-traffic",
    ] {
        let artifact_dir = repo_root.join("artifacts/scenarios").join(scenario);
        assert!(artifact_dir.join("run.json").exists(), "run.json should exist");
        assert!(artifact_dir.join("run.md").exists(), "run.md should exist");
    }

    second_server.abort();
}

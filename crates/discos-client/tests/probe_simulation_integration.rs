use std::{path::PathBuf, process::Command, sync::Arc};

use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Code, Request, Response, Status};

use discos_client::pb;

#[derive(Default)]
struct State {
    total_executes: usize,
}

#[derive(Clone)]
struct ProbeMockDaemon(Arc<Mutex<State>>);

#[tonic::async_trait]
impl pb::evidence_os_server::EvidenceOs for ProbeMockDaemon {
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
        let body = req.into_inner();
        let mut claim_id = vec![0u8; 32];
        for (idx, b) in body.claim_name.as_bytes().iter().enumerate().take(32) {
            claim_id[idx] = *b;
        }
        let topic_id = body
            .signals
            .map(|s| s.semantic_hash)
            .filter(|h| !h.is_empty())
            .unwrap_or_else(|| vec![7u8; 32]);
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
        let _claim_id = req.into_inner().claim_id;
        let mut guard = self.0.lock().await;
        guard.total_executes += 1;

        if guard.total_executes >= 5 {
            return Err(Status::new(Code::FailedPrecondition, "FROZEN"));
        }
        if guard.total_executes >= 3 {
            return Err(Status::new(Code::ResourceExhausted, "THROTTLE"));
        }

        Ok(Response::new(pb::ExecuteClaimV2Response {
            certified: true,
            e_value: 1.0,
            canonical_output: b"{}".to_vec(),
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

    type WatchRevocationsStream =
        tokio_stream::wrappers::ReceiverStream<Result<pb::RevocationEvent, Status>>;

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
async fn probe_simulation_script_writes_artifacts_and_detects_controls() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("listener addr");

    let daemon = ProbeMockDaemon(Arc::new(Mutex::new(State::default())));

    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(pb::evidence_os_server::EvidenceOsServer::new(daemon))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("mock daemon should serve");
    });

    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root parent")
        .parent()
        .expect("repo root")
        .to_path_buf();
    let out_dir = repo_root.join("artifacts/system-test/probe-sim-integration");
    std::fs::create_dir_all(&out_dir).expect("create artifact dir");

    let status = Command::new("bash")
        .current_dir(&repo_root)
        .arg("scripts/probe_simulation.sh")
        .arg("--endpoint")
        .arg(format!("http://{addr}"))
        .arg("--claims")
        .arg("6")
        .arg("--unique-hashes")
        .arg("3")
        .arg("--topics")
        .arg("2")
        .arg("--artifact-dir")
        .arg(&out_dir)
        .arg("--require-controls")
        .status()
        .expect("run probe simulation script");

    server.abort();

    assert!(status.success(), "probe simulation script should succeed");

    let summary_path = out_dir.join("probe_simulation_summary.json");
    let requests_path = out_dir.join("probe_simulation_requests.jsonl");
    assert!(summary_path.exists(), "summary artifact should exist");
    assert!(requests_path.exists(), "requests artifact should exist");

    let summary: serde_json::Value = serde_json::from_slice(
        &std::fs::read(summary_path).expect("summary should be readable json"),
    )
    .expect("summary should parse");
    assert!(summary["throttle_started_at"].is_number() || summary["freeze_started_at"].is_number());
}

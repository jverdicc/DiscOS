use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use discos_client::{pb, ClientError, DiscosClient};
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Clone)]
struct OracleMockDaemon {
    create_calls: Arc<AtomicUsize>,
}

#[tonic::async_trait]
impl pb::evidence_os_server::EvidenceOs for OracleMockDaemon {
    async fn health(
        &self,
        _: Request<pb::HealthRequest>,
    ) -> Result<Response<pb::HealthResponse>, Status> {
        Ok(Response::new(pb::HealthResponse {
            status: "ok".to_string(),
        }))
    }

    async fn create_claim_v2(
        &self,
        req: Request<pb::CreateClaimV2Request>,
    ) -> Result<Response<pb::CreateClaimV2Response>, Status> {
        self.create_calls.fetch_add(1, Ordering::SeqCst);
        let req = req.into_inner();
        if req.oracle_id == "acme.safety.v1" {
            return Ok(Response::new(pb::CreateClaimV2Response {
                claim_id: vec![1; 32],
                topic_id: vec![2; 32],
            }));
        }

        Err(Status::invalid_argument(format!(
            "unknown oracle_id: {}",
            req.oracle_id
        )))
    }

    async fn commit_artifacts(
        &self,
        _: Request<pb::CommitArtifactsRequest>,
    ) -> Result<Response<pb::CommitArtifactsResponse>, Status> {
        Err(Status::unimplemented("not needed"))
    }

    async fn commit_wasm(
        &self,
        _: Request<pb::CommitWasmRequest>,
    ) -> Result<Response<pb::CommitWasmResponse>, Status> {
        Err(Status::unimplemented("not needed"))
    }

    async fn freeze(
        &self,
        _: Request<pb::FreezeRequest>,
    ) -> Result<Response<pb::FreezeResponse>, Status> {
        Err(Status::unimplemented("not needed"))
    }

    async fn execute_claim_v2(
        &self,
        _: Request<pb::ExecuteClaimV2Request>,
    ) -> Result<Response<pb::ExecuteClaimV2Response>, Status> {
        Err(Status::unimplemented("not needed"))
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

async fn spawn_server() -> (String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind listener");
    let addr = listener.local_addr().expect("local addr");
    let calls = Arc::new(AtomicUsize::new(0));

    let daemon = OracleMockDaemon {
        create_calls: Arc::clone(&calls),
    };

    let incoming = TcpListenerStream::new(listener);
    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(pb::evidence_os_server::EvidenceOsServer::new(daemon))
            .serve_with_incoming(incoming)
            .await
            .expect("serve mock daemon");
    });

    (format!("http://{}", addr), calls, handle)
}

fn request_with_oracle_id(oracle_id: &str) -> pb::CreateClaimV2Request {
    pb::CreateClaimV2Request {
        claim_name: "oracle-claim".to_string(),
        metadata: Some(pb::ClaimMetadataV2 {
            lane: "lane-a".to_string(),
            alpha_micros: 1,
            epoch_config_ref: "epoch.v1".to_string(),
            output_schema_id: "cbrn-sc.v1".to_string(),
        }),
        signals: Some(pb::TopicSignalsV2 {
            semantic_hash: vec![],
            phys_hir_signature_hash: vec![3; 32],
            dependency_merkle_root: vec![],
        }),
        holdout_ref: "holdout.v1".to_string(),
        epoch_size: 32,
        oracle_num_symbols: 8,
        access_credit: 1024,
        oracle_id: oracle_id.to_string(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unknown_oracle_id_returns_actionable_error_without_retries() {
    let (endpoint, calls, _jh) = spawn_server().await;
    let mut client = DiscosClient::connect(&endpoint)
        .await
        .expect("connect client");

    let err = client
        .create_claim_v2(request_with_oracle_id("does.not.exist.v1"))
        .await
        .expect_err("unknown oracle id should fail");

    match err {
        ClientError::Kernel(message) => {
            assert!(message.contains("InvalidArgument"));
            assert!(message.contains("unknown oracle_id: does.not.exist.v1"));
        }
        other => panic!("expected kernel error, got {other:?}"),
    }

    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn known_oracle_id_succeeds() {
    let (endpoint, calls, _jh) = spawn_server().await;
    let mut client = DiscosClient::connect(&endpoint)
        .await
        .expect("connect client");

    let resp = client
        .create_claim_v2(request_with_oracle_id("acme.safety.v1"))
        .await
        .expect("known oracle id should succeed");

    assert_eq!(resp.claim_id.len(), 32);
    assert_eq!(resp.topic_id.len(), 32);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

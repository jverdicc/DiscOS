use std::{collections::HashMap, sync::Arc, time::Duration};

use discos_client::pb;
use evidenceos_redteam::{run_redteam, Thresholds};
use sha2::Digest;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Code, Request, Response, Status};

#[derive(Default)]
struct MockState {
    claim_topics: HashMap<Vec<u8>, Vec<u8>>,
}

#[derive(Clone)]
struct MockDaemon(Arc<Mutex<MockState>>);

#[tonic::async_trait]
impl pb::evidence_os_server::EvidenceOs for MockDaemon {
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
        let metadata = r
            .metadata
            .ok_or_else(|| Status::new(Code::InvalidArgument, "INVALID_METADATA"))?;
        if r.claim_name.trim().is_empty() {
            return Err(Status::new(Code::InvalidArgument, "INVALID_CLAIM_NAME"));
        }
        if metadata.epoch_config_ref.contains("nullspec/unsigned") {
            return Err(Status::new(Code::FailedPrecondition, "UNSIGNED_NULLSPEC"));
        }

        let claim_id = sha2::Sha256::digest(
            format!("{}:{}", r.claim_name, metadata.epoch_config_ref).as_bytes(),
        )
        .to_vec();
        let topic_id = r
            .signals
            .map(|s| s.semantic_hash)
            .ok_or_else(|| Status::new(Code::InvalidArgument, "INVALID_SIGNALS"))?;
        self.0
            .lock()
            .await
            .claim_topics
            .insert(claim_id.clone(), topic_id.clone());
        Ok(Response::new(pb::CreateClaimV2Response {
            claim_id,
            topic_id,
            state: pb::ClaimState::Committed as i32,
        }))
    }

    async fn execute_claim_v2(
        &self,
        req: Request<pb::ExecuteClaimV2Request>,
    ) -> Result<Response<pb::ExecuteClaimV2Response>, Status> {
        let id = req.into_inner().claim_id;
        let state = self.0.lock().await;
        let _topic = state
            .claim_topics
            .get(&id)
            .ok_or_else(|| Status::new(Code::InvalidArgument, "UNKNOWN_CLAIM"))?;
        tokio::time::sleep(Duration::from_millis(1)).await;
        Ok(Response::new(pb::ExecuteClaimV2Response {
            state: pb::ClaimState::Certified as i32,
            decision: pb::Decision::Approve as i32,
            reason_codes: Vec::new(),
            canonical_output: br#"{"decision":"allow"}"#.to_vec(),
            e_value: 1.0,
            certified: true,
            capsule_hash: vec![0; 32],
            etl_index: 0,
        }))
    }

    async fn create_claim(
        &self,
        _: Request<pb::CreateClaimRequest>,
    ) -> Result<Response<pb::CreateClaimResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }

    async fn commit_artifacts(
        &self,
        _: Request<pb::CommitArtifactsRequest>,
    ) -> Result<Response<pb::CommitArtifactsResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn freeze(
        &self,
        _: Request<pb::FreezeRequest>,
    ) -> Result<Response<pb::FreezeResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn fetch_capsule(
        &self,
        _: Request<pb::FetchCapsuleRequest>,
    ) -> Result<Response<pb::FetchCapsuleResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn get_public_key(
        &self,
        _: Request<pb::GetPublicKeyRequest>,
    ) -> Result<Response<pb::GetPublicKeyResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn get_signed_tree_head(
        &self,
        _: Request<pb::GetSignedTreeHeadRequest>,
    ) -> Result<Response<pb::GetSignedTreeHeadResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn get_inclusion_proof(
        &self,
        _: Request<pb::GetInclusionProofRequest>,
    ) -> Result<Response<pb::GetInclusionProofResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn get_consistency_proof(
        &self,
        _: Request<pb::GetConsistencyProofRequest>,
    ) -> Result<Response<pb::GetConsistencyProofResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn revoke_claim(
        &self,
        _: Request<pb::RevokeClaimRequest>,
    ) -> Result<Response<pb::RevokeClaimResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    type WatchRevocationsStream =
        tokio_stream::wrappers::ReceiverStream<Result<pb::WatchRevocationsResponse, Status>>;
    async fn watch_revocations(
        &self,
        _: Request<pb::WatchRevocationsRequest>,
    ) -> Result<Response<Self::WatchRevocationsStream>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn freeze_gates(
        &self,
        _: Request<pb::FreezeGatesRequest>,
    ) -> Result<Response<pb::FreezeGatesResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn seal(
        &self,
        _: Request<pb::SealRequest>,
    ) -> Result<Response<pb::SealResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn seal_claim(
        &self,
        _: Request<pb::SealClaimRequest>,
    ) -> Result<Response<pb::SealClaimResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn execute_claim(
        &self,
        _: Request<pb::ExecuteClaimRequest>,
    ) -> Result<Response<pb::ExecuteClaimResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn get_capsule(
        &self,
        _: Request<pb::GetCapsuleRequest>,
    ) -> Result<Response<pb::GetCapsuleResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn get_revocation_feed(
        &self,
        _: Request<pb::GetRevocationFeedRequest>,
    ) -> Result<Response<pb::GetRevocationFeedResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn grant_credit(
        &self,
        _: Request<pb::GrantCreditRequest>,
    ) -> Result<Response<pb::GrantCreditResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn set_credit_limit(
        &self,
        _: Request<pb::SetCreditLimitRequest>,
    ) -> Result<Response<pb::SetCreditLimitResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn set_holdout_pool_budgets(
        &self,
        _: Request<pb::SetHoldoutPoolBudgetsRequest>,
    ) -> Result<Response<pb::SetHoldoutPoolBudgetsResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
    async fn get_holdout_pool_budgets(
        &self,
        _: Request<pb::GetHoldoutPoolBudgetsRequest>,
    ) -> Result<Response<pb::GetHoldoutPoolBudgetsResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }

    async fn get_server_info(
        &self,
        _: Request<pb::GetServerInfoRequest>,
    ) -> Result<Response<pb::GetServerInfoResponse>, Status> {
        Err(Status::unimplemented("n/a"))
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn redteam_harness_passes_against_hardened_mock() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");

    let server = tokio::spawn(async move {
        Server::builder()
            .add_service(pb::evidence_os_server::EvidenceOsServer::new(MockDaemon(
                Arc::new(Mutex::new(MockState::default())),
            )))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("serve");
    });

    let result = run_redteam(
        &format!("http://{addr}"),
        8,
        &Thresholds {
            max_arm_auc: 0.95,
            max_size_variance: 0.0,
            enforce_strict_pln: true,
            production_mode: true,
        },
    )
    .await;

    server.abort();
    assert!(
        result.is_ok(),
        "redteam should pass on hardened daemon: {result:?}"
    );
}

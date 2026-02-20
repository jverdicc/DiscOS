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

use std::{collections::HashMap, sync::Arc};

use discos_client::{
    pb, verify_consistency, verify_inclusion, verify_sth_signature, ConsistencyProof, DiscosClient,
    InclusionProof, SignedTreeHead,
};
use ed25519_dalek::{Signer, SigningKey};
use evidenceos_core::crypto_transcripts;
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
    key_id: String,
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
        let leaf = crypto_transcripts::etl_leaf_hash(&capsule);
        let mut st = self.0.lock().await;
        st.root = leaf;
        st.sig = st
            .signing_key
            .sign(&crypto_transcripts::sth_signature_digest(1, st.root))
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
            key_id: st.key_id.clone(),
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
            proto_hash: "test-proto-hash".into(),
            protocol_package: "evidenceos.v2".into(),
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

#[tokio::test]
#[ignore]
async fn e2e_v2_daemon_contract_verification() {
    let key = SigningKey::from_bytes(&[7u8; 32]);
    let daemon = TestDaemon(Arc::new(Mutex::new(State {
        claims: HashMap::new(),
        signing_key: key,
        root: [0; 32],
        sig: [0; 64],
        key_id: "k1".into(),
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

    let mut c = DiscosClient::connect(&format!("http://{addr}"))
        .await
        .expect("connect");
    c.health().await.expect("health");
    let created = c
        .create_claim_v2(pb::CreateClaimV2Request {
            claim_name: "c".into(),
            metadata: None,
            signals: None,
            holdout_ref: "h".into(),
            epoch_size: 1,
            oracle_num_symbols: 1,
            access_credit: 1,
            oracle_id: "default".into(),
        })
        .await
        .expect("create");
    c.commit_artifacts(pb::CommitArtifactsRequest {
        claim_id: created.claim_id.clone(),
        manifests: vec![],
    })
    .await
    .expect("commit artifacts");
    c.commit_wasm(pb::CommitWasmRequest {
        claim_id: created.claim_id.clone(),
        wasm_hash: vec![],
        wasm_module: vec![],
    })
    .await
    .expect("commit wasm");
    c.freeze(pb::FreezeRequest {
        claim_id: created.claim_id.clone(),
    })
    .await
    .expect("freeze");
    c.execute_claim_v2(pb::ExecuteClaimV2Request {
        claim_id: created.claim_id.clone(),
    })
    .await
    .expect("execute");
    let key = c.get_public_key().await.expect("pubkey");
    let f = c
        .fetch_capsule(pb::FetchCapsuleRequest {
            claim_id: created.claim_id.clone(),
        })
        .await
        .expect("fetch");
    let sth = SignedTreeHead {
        tree_size: f.etl_tree_size,
        root_hash: f.etl_root_hash.clone().try_into().expect("root len"),
        signature: f.sth_signature.clone().try_into().expect("sig len"),
    };
    verify_sth_signature(&sth, &key.pubkey).expect("sig verify");
    assert!(verify_inclusion(
        sth.root_hash,
        &InclusionProof {
            leaf_hash: [3u8; 32],
            leaf_index: 0,
            tree_size: 1,
            audit_path: vec![]
        }
    ));
    assert!(verify_consistency(
        sth.root_hash,
        sth.root_hash,
        &ConsistencyProof {
            old_tree_size: 1,
            new_tree_size: 1,
            path: vec![]
        }
    ));
}

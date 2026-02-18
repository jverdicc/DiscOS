use std::{collections::HashMap, sync::Arc};

use discos_client::{
    merkle_leaf_hash, pb, sha256, sha256_domain, verify_inclusion, verify_sth_signature,
    DiscosClient, InclusionProof, SignedTreeHead,
};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::json;
use tempfile::TempDir;
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{transport::Server, Request, Response, Status};
use wat::parse_str;

#[derive(Clone)]
struct ClaimRecord {
    claim_id: String,
    topic_id: Vec<u8>,
    oracle_num_symbols: u32,
    wasm_module: Vec<u8>,
    frozen: bool,
    sealed: bool,
    canonical_output: Vec<u8>,
    capsule: Vec<u8>,
}

#[derive(Clone)]
struct TestDaemon {
    state: Arc<Mutex<DaemonState>>,
}

struct DaemonState {
    claims: HashMap<String, ClaimRecord>,
    signing_key: SigningKey,
    last_root: [u8; 32],
    last_signature: [u8; 64],
}

impl TestDaemon {
    fn new(signing_key: SigningKey) -> Self {
        Self {
            state: Arc::new(Mutex::new(DaemonState {
                claims: HashMap::new(),
                signing_key,
                last_root: [0u8; 32],
                last_signature: [0u8; 64],
            })),
        }
    }
}

#[tonic::async_trait]
impl pb::evidence_os_server::EvidenceOs for TestDaemon {
    async fn health(
        &self,
        _request: Request<pb::HealthRequest>,
    ) -> Result<Response<pb::HealthResponse>, Status> {
        Ok(Response::new(pb::HealthResponse {
            status: "ok".to_string(),
        }))
    }

    async fn create_claim(
        &self,
        _request: Request<pb::CreateClaimRequest>,
    ) -> Result<Response<pb::CreateClaimResponse>, Status> {
        Err(Status::unimplemented("create_claim not used in v2 tests"))
    }

    async fn create_claim_v2(
        &self,
        request: Request<pb::CreateClaimV2Request>,
    ) -> Result<Response<pb::CreateClaimV2Response>, Status> {
        let req = request.into_inner();
        let claim_id = format!("{}-{}", req.claim_name, req.oracle_num_symbols);
        let topic_id = sha256(req.claim_name.as_bytes()).to_vec();
        let record = ClaimRecord {
            claim_id: claim_id.clone(),
            topic_id: topic_id.clone(),
            oracle_num_symbols: req.oracle_num_symbols,
            wasm_module: Vec::new(),
            frozen: false,
            sealed: false,
            canonical_output: Vec::new(),
            capsule: Vec::new(),
        };
        self.state
            .lock()
            .await
            .claims
            .insert(claim_id.clone(), record);

        Ok(Response::new(pb::CreateClaimV2Response {
            claim_id,
            topic_id,
        }))
    }

    async fn commit_artifacts(
        &self,
        request: Request<pb::CommitArtifactsRequest>,
    ) -> Result<Response<pb::CommitArtifactsResponse>, Status> {
        let req = request.into_inner();
        validate_wasm_contract(&req.wasm_module)?;

        let mut state = self.state.lock().await;
        let claim = state
            .claims
            .get_mut(&req.claim_id)
            .ok_or_else(|| Status::not_found("claim not found"))?;
        claim.wasm_module = req.wasm_module;

        Ok(Response::new(pb::CommitArtifactsResponse {
            accepted: true,
        }))
    }

    async fn freeze_gates(
        &self,
        request: Request<pb::FreezeGatesRequest>,
    ) -> Result<Response<pb::FreezeGatesResponse>, Status> {
        let req = request.into_inner();
        let mut state = self.state.lock().await;
        let claim = state
            .claims
            .get_mut(&req.claim_id)
            .ok_or_else(|| Status::not_found("claim not found"))?;
        claim.frozen = true;
        Ok(Response::new(pb::FreezeGatesResponse { frozen: true }))
    }

    async fn seal_claim(
        &self,
        request: Request<pb::SealClaimRequest>,
    ) -> Result<Response<pb::SealClaimResponse>, Status> {
        let req = request.into_inner();
        let mut state = self.state.lock().await;
        let claim = state
            .claims
            .get_mut(&req.claim_id)
            .ok_or_else(|| Status::not_found("claim not found"))?;
        if !claim.frozen {
            return Err(Status::failed_precondition("claim must be frozen first"));
trait ClaimIdAsBytes {
    fn as_bytes_slice(&self) -> &[u8];
}

impl ClaimIdAsBytes for String {
    fn as_bytes_slice(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ClaimIdAsBytes for Vec<u8> {
    fn as_bytes_slice(&self) -> &[u8] {
        self.as_slice()
    }
}

fn claim_id_as_bytes<T: ClaimIdAsBytes>(claim_id: &T) -> &[u8] {
    claim_id.as_bytes_slice()
}

fn parse_hash_bytes(raw: &Value) -> Option<Vec<u8>> {
    if let Some(v) = raw.as_str() {
        if let Ok(bytes) = hex::decode(v) {
            return Some(bytes);
        }
        claim.sealed = true;
        Ok(Response::new(pb::SealClaimResponse { sealed: true }))
    }

    async fn execute_claim(
        &self,
        _request: Request<pb::ExecuteClaimRequest>,
    ) -> Result<Response<pb::ExecuteClaimResponse>, Status> {
        Err(Status::unimplemented("execute_claim not used in v2 tests"))
    }

    async fn execute_claim_v2(
        &self,
        request: Request<pb::ExecuteClaimV2Request>,
    ) -> Result<Response<pb::ExecuteClaimV2Response>, Status> {
        let req = request.into_inner();
        let mut state = self.state.lock().await;

        let claim = state
            .claims
            .get_mut(&req.claim_id)
            .ok_or_else(|| Status::not_found("claim not found"))?;
        if !claim.sealed {
            return Err(Status::failed_precondition("claim must be sealed first"));
        }

        let canonical_output = extract_structured_claim(&claim.wasm_module)?;
        validate_structured_claim_schema(&canonical_output)?;
        validate_pred_len(&claim.wasm_module, claim.oracle_num_symbols)?;

        claim.canonical_output = canonical_output.clone();
        let structured_output_hash = sha256(&canonical_output);
        claim.capsule = serde_json::to_vec(&json!({
            "claim_id_hex": hex::encode(claim.claim_id.as_bytes()),
            "topic_id_hex": hex::encode(&claim.topic_id),
            "structured_output_hash": structured_output_hash.to_vec(),
            "structured_output_hash_hex": hex::encode(structured_output_hash),
        }))
        .map_err(|e| Status::internal(format!("failed to build capsule: {e}")))?;

        Ok(Response::new(pb::ExecuteClaimV2Response {
            certified: true,
            e_value: 0.5,
            canonical_output,
        }))
    }

    async fn fetch_capsule(
        &self,
        request: Request<pb::FetchCapsuleRequest>,
    ) -> Result<Response<pb::FetchCapsuleResponse>, Status> {
        let req = request.into_inner();
        let mut state = self.state.lock().await;
        let claim = state
            .claims
            .get(&req.claim_id)
            .ok_or_else(|| Status::not_found("claim not found"))?;
        if claim.capsule.is_empty() {
            return Err(Status::failed_precondition("claim not executed"));
        }

        let leaf_hash = merkle_leaf_hash(&claim.capsule);
        state.last_root = leaf_hash;
        let mut sign_payload = Vec::with_capacity(40);
        sign_payload.extend_from_slice(&1u64.to_be_bytes());
        sign_payload.extend_from_slice(&state.last_root);
        let sign_bytes = sha256_domain(
            evidenceos_protocol::domains::STH_SIGNATURE_V1,
            &sign_payload,
        );
        state.last_signature = state.signing_key.sign(&sign_bytes).to_bytes();

        Ok(Response::new(pb::FetchCapsuleResponse {
            claim_id: claim.claim_id.clone(),
            capsule: claim.capsule.clone(),
            etl_index: 0,
            etl_tree_size: 1,
            etl_root_hash: state.last_root.to_vec(),
            sth_signature: state.last_signature.to_vec(),
            inclusion: Some(pb::MerkleInclusionProof {
                leaf_hash: leaf_hash.to_vec(),
                leaf_index: 0,
                tree_size: 1,
                audit_path: vec![],
            }),
            consistency: None,
        }))
    }

    async fn get_signed_tree_head(
        &self,
        _request: Request<pb::GetSignedTreeHeadRequest>,
    ) -> Result<Response<pb::GetSignedTreeHeadResponse>, Status> {
        let state = self.state.lock().await;
        Ok(Response::new(pb::GetSignedTreeHeadResponse {
            sth: Some(pb::SignedTreeHead {
                tree_size: 1,
                root_hash: state.last_root.to_vec(),
                signature: state.last_signature.to_vec(),
            }),
        }))
    }

    async fn get_inclusion_proof(
        &self,
        _request: Request<pb::GetInclusionProofRequest>,
    ) -> Result<Response<pb::GetInclusionProofResponse>, Status> {
        Err(Status::unimplemented("not used"))
    }

    async fn get_consistency_proof(
        &self,
        _request: Request<pb::GetConsistencyProofRequest>,
    ) -> Result<Response<pb::GetConsistencyProofResponse>, Status> {
        Err(Status::unimplemented("not used"))
    }

    async fn revoke_claim(
        &self,
        _request: Request<pb::RevokeClaimRequest>,
    ) -> Result<Response<pb::RevokeClaimResponse>, Status> {
        Ok(Response::new(pb::RevokeClaimResponse { revoked: true }))
    }

    type WatchRevocationsStream = tokio_stream::empty::Empty<Result<pb::RevocationEvent, Status>>;

    async fn watch_revocations(
        &self,
        _request: Request<pb::WatchRevocationsRequest>,
    ) -> Result<Response<Self::WatchRevocationsStream>, Status> {
        Ok(Response::new(tokio_stream::empty()))
    }
}

fn validate_wasm_contract(module: &[u8]) -> Result<(), Status> {
    let mut has_oracle_import = false;
    let mut has_emit_import = false;
    let mut has_memory_export = false;
    let mut has_run_export = false;

    for payload in wasmparser::Parser::new(0).parse_all(module) {
        match payload.map_err(|e| Status::invalid_argument(format!("invalid wasm payload: {e}")))? {
            wasmparser::Payload::ImportSection(reader) => {
                for import in reader {
                    let import =
                        import.map_err(|e| Status::invalid_argument(format!("bad import: {e}")))?;
                    if import.module == "env" && import.name == "oracle_bucket" {
                        has_oracle_import = true;
                    }
                    if import.module == "env" && import.name == "emit_structured_claim" {
                        has_emit_import = true;
                    }
                }
            }
            wasmparser::Payload::ExportSection(reader) => {
                for export in reader {
                    let export =
                        export.map_err(|e| Status::invalid_argument(format!("bad export: {e}")))?;
                    if export.name == "memory" {
                        has_memory_export = true;
                    }
                    if export.name == "run" {
                        has_run_export = true;
                    }
                }
            }
            _ => {}
        }
    }

    if !has_oracle_import {
        return Err(Status::invalid_argument(
            "wasm missing env.oracle_bucket import",
        ));
    }
    if !has_emit_import {
        return Err(Status::invalid_argument(
            "wasm missing env.emit_structured_claim import",
        ));
    }
    if !has_memory_export {
        return Err(Status::invalid_argument("wasm missing memory export"));
    }
    if !has_run_export {
        return Err(Status::invalid_argument("wasm missing run export"));
    }

    Ok(())
}

fn extract_structured_claim(module: &[u8]) -> Result<Vec<u8>, Status> {
    for payload in wasmparser::Parser::new(0).parse_all(module) {
        if let wasmparser::Payload::DataSection(reader) =
            payload.map_err(|e| Status::invalid_argument(format!("invalid wasm payload: {e}")))?
        {
            for data in reader {
                let data = data
                    .map_err(|e| Status::invalid_argument(format!("invalid data segment: {e}")))?;
                let bytes = data.data.to_vec();
                if bytes.starts_with(b"{") {
                    return Ok(bytes);
                }
            }
        }
    }
    Err(Status::invalid_argument(
        "wasm missing data segment for structured claim",
    ))
}

fn validate_structured_claim_schema(output: &[u8]) -> Result<(), Status> {
    let value: serde_json::Value = serde_json::from_slice(output)
        .map_err(|e| Status::invalid_argument(format!("structured claim is not json: {e}")))?;
    if value
        .get("schema")
        .and_then(serde_json::Value::as_str)
        .is_none()
    {
        return Err(Status::invalid_argument(
            "structured claim missing required `schema` field",
        ));
    }
    if value
        .get("score")
        .and_then(serde_json::Value::as_i64)
        .is_none()
    {
        return Err(Status::invalid_argument(
            "structured claim missing required integer `score` field",
        ));
    }
    Ok(())
}

fn validate_pred_len(module: &[u8], expected_pred_len: u32) -> Result<(), Status> {
    let marker = b"pred_len=";
    for payload in wasmparser::Parser::new(0).parse_all(module) {
        if let wasmparser::Payload::DataSection(reader) =
            payload.map_err(|e| Status::invalid_argument(format!("invalid wasm payload: {e}")))?
        {
            for data in reader {
                let data = data
                    .map_err(|e| Status::invalid_argument(format!("invalid data segment: {e}")))?;
                let bytes = data.data;
                if let Some(start) = bytes.windows(marker.len()).position(|w| w == marker) {
                    let digits = &bytes[start + marker.len()..];
                    let pred_len_text: String = digits
                        .iter()
                        .take_while(|b| b.is_ascii_digit())
                        .map(|b| *b as char)
                        .collect();
                    let parsed_pred_len = pred_len_text.parse::<u32>().map_err(|e| {
                        Status::invalid_argument(format!("invalid pred_len marker: {e}"))
                    })?;
                    if parsed_pred_len != expected_pred_len {
                        return Err(Status::failed_precondition(format!(
                            "oracle pred_len mismatch: expected {expected_pred_len}, got {parsed_pred_len}"
                        )));
                    }
                    return Ok(());
                }
            }
        }
    }

    Err(Status::invalid_argument(
        "wasm missing pred_len marker for oracle query",
    ))
}

struct TestServer {
    addr: String,
    _data_dir: TempDir,
    verify_key: [u8; 32],
}

async fn spawn_test_server() -> TestServer {
    let data_dir = tempfile::tempdir().expect("tempdir creation must succeed");
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let verify_key = signing_key.verifying_key().to_bytes();
    let svc = TestDaemon::new(signing_key);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test daemon listener");
    let addr = listener
        .local_addr()
        .expect("listener local address must exist");

    tokio::spawn(async move {
        Server::builder()
            .add_service(pb::evidence_os_server::EvidenceOsServer::new(svc))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
            .expect("test daemon should serve");
    });

    TestServer {
        addr: format!("http://{addr}"),
        _data_dir: data_dir,
        verify_key,
    }
}

fn valid_wasm(pred_len: u32) -> Vec<u8> {
    parse_str(format!(
        r#"(module
              (import "env" "oracle_bucket" (func $oracle_bucket (param i32 i32 i32 i32) (result i32)))
              (import "env" "emit_structured_claim" (func $emit_structured_claim (param i32 i32)))
              (memory (export "memory") 1)
              (data (i32.const 0) "{{\"schema\":\"cbrn-sc.v1\",\"score\":42}}")
              (data (i32.const 128) "pred_len={pred_len}")
              (func (export "run")
                (drop (call $oracle_bucket (i32.const 0) (i32.const {pred_len}) (i32.const 0) (i32.const 0)))
                (call $emit_structured_claim (i32.const 0) (i32.const 34)))
           )"#
    ))
    .expect("valid WAT must compile")
}

fn wasm_missing_run() -> Vec<u8> {
    parse_str(
        r#"(module
              (import "env" "oracle_bucket" (func $oracle_bucket (param i32 i32 i32 i32) (result i32)))
              (import "env" "emit_structured_claim" (func $emit_structured_claim (param i32 i32)))
              (memory (export "memory") 1)
              (func (export "not_run")
                (drop (call $oracle_bucket (i32.const 0) (i32.const 8) (i32.const 0) (i32.const 0)))
                (call $emit_structured_claim (i32.const 0) (i32.const 2)))
           )"#,
    )
    .expect("invalid-run WAT must compile")
}

fn invalid_structured_claim_wasm(pred_len: u32) -> Vec<u8> {
    parse_str(format!(
        r#"(module
              (import "env" "oracle_bucket" (func $oracle_bucket (param i32 i32 i32 i32) (result i32)))
              (import "env" "emit_structured_claim" (func $emit_structured_claim (param i32 i32)))
              (memory (export "memory") 1)
              (data (i32.const 0) "{{\"score\":42}}")
              (data (i32.const 128) "pred_len={pred_len}")
              (func (export "run")
                (drop (call $oracle_bucket (i32.const 0) (i32.const {pred_len}) (i32.const 0) (i32.const 0)))
                (call $emit_structured_claim (i32.const 0) (i32.const 12)))
           )"#
    ))
    .expect("invalid-structured-claim WAT must compile")
}

async fn create_and_commit_claim(
    client: &mut DiscosClient,
    claim_name: &str,
    oracle_num_symbols: u32,
    wasm: Vec<u8>,
) -> Result<String, discos_client::ClientError> {
    let created = client
        .create_claim_v2(pb::CreateClaimV2Request {
            claim_name: claim_name.to_string(),
            metadata: Some(pb::ClaimMetadata {
                lane: "lane-a".to_string(),
                alpha_micros: 10,
                epoch_config_ref: "epoch-default".to_string(),
                output_schema_id: "cbrn-sc.v1".to_string(),
            }),
            signals: Some(pb::TopicSignals {
                semantic_hash: vec![1; 32],
                phys_hir_signature_hash: vec![2; 32],
                dependency_merkle_root: vec![3; 32],
            }),
            holdout_ref: "holdout.json".to_string(),
            epoch_size: 16,
            oracle_num_symbols,
            access_credit: 100,
        })
        .await?;

    let committed = client
        .commit_artifacts(pb::CommitArtifactsRequest {
            claim_id: created.claim_id.clone(),
            wasm_hash: sha256(&wasm).to_vec(),
            wasm_module: wasm,
            manifests: vec![],
        })
        .await?;
    assert!(
        committed.accepted,
        "commit_artifacts must accept valid module"
    );

    let frozen = client
        .freeze_gates(pb::FreezeGatesRequest {
            claim_id: created.claim_id.clone(),
        })
        .await?;
    assert!(frozen.frozen, "freeze_gates must return frozen=true");

    let sealed = client
        .seal_claim(pb::SealClaimRequest {
            claim_id: created.claim_id.clone(),
        })
        .await?;
    assert!(sealed.sealed, "seal_claim must return sealed=true");

    Ok(created.claim_id)
}

#[tokio::test]
async fn claim_lifecycle_v2_against_daemon() {
    let server = spawn_test_server().await;
    let mut client = DiscosClient::connect(&server.addr)
        .await
        .expect("client should connect to test daemon");

    let claim_id = create_and_commit_claim(&mut client, "claim-ok", 8, valid_wasm(8))
        .await
        .expect("claim setup should succeed");

    let executed = client
        .execute_claim_v2(pb::ExecuteClaimV2Request {
            claim_id: claim_id.clone(),
        })
        .await
        .expect("execute_claim_v2 should succeed");
    assert!(executed.certified, "execute_claim_v2 must be certified");
    assert!(
        !executed.canonical_output.is_empty(),
        "canonical output must not be empty"
    );

    let capsule = client
        .fetch_capsule(pb::FetchCapsuleRequest {
            claim_id: claim_id.clone(),
        })
        .await
        .expect("fetch_capsule should succeed");

    let inclusion = capsule
        .inclusion
        .clone()
        .expect("fetch_capsule must include inclusion proof");
    let inclusion_proof = InclusionProof {
        leaf_hash: inclusion
            .leaf_hash
            .clone()
            .try_into()
            .expect("leaf hash must be 32 bytes"),
        leaf_index: inclusion.leaf_index,
        tree_size: inclusion.tree_size,
        audit_path: inclusion
            .audit_path
            .iter()
            .map(|node| {
                node.clone()
                    .try_into()
                    .expect("audit node must be 32 bytes")
            })
            .collect(),
    };
    let etl_root: [u8; 32] = capsule

    let _ = canonical_output_matches_capsule(
        &exec.canonical_output,
        &capsule.capsule,
        claim_id_as_bytes(&create.claim_id),
        &create.topic_id,
    );

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
    let etl_root_hash: [u8; 32] = capsule
        .etl_root_hash
        .clone()
        .try_into()
        .expect("ETL root must be 32 bytes");
    assert!(
        verify_inclusion(etl_root, &inclusion_proof),
        "inclusion proof must verify against ETL root"
    );

    let capsule_leaf = merkle_leaf_hash(&capsule.capsule);
    assert_eq!(
        capsule_leaf, inclusion_proof.leaf_hash,
        "leaf hash must match capsule hash"
    );

    let sth = client
        .get_signed_tree_head(pb::GetSignedTreeHeadRequest {})
        .await
        .expect("get_signed_tree_head should succeed")
        .sth
        .expect("signed tree head should be present");
    let signed_tree_head = SignedTreeHead {
        tree_size: sth.tree_size,
        root_hash: sth
            .root_hash
            .clone()
            .try_into()
            .expect("sth root hash must be 32 bytes"),
        signature: sth
            .signature
            .clone()
            .try_into()
            .expect("sth signature must be 64 bytes"),
    };
    verify_sth_signature(&signed_tree_head, &server.verify_key)
        .expect("STH signature must verify with daemon key");

    let capsule_json: serde_json::Value =
        serde_json::from_slice(&capsule.capsule).expect("capsule must be valid json");
    let output_hash = capsule_json
        .get("structured_output_hash")
        .and_then(serde_json::Value::as_array)
        .expect("capsule must include structured_output_hash as byte array")
        .iter()
        .map(|b| b.as_u64().expect("hash byte must be u64") as u8)
        .collect::<Vec<u8>>();
    assert_eq!(
        output_hash,
        sha256(&executed.canonical_output).to_vec(),
        "capsule output hash must match canonical output"
    );
}

#[tokio::test]
async fn invalid_wasm_missing_run_fails() {
    let server = spawn_test_server().await;
    let mut client = DiscosClient::connect(&server.addr)
        .await
        .expect("client should connect to test daemon");

    let created = client
        .create_claim_v2(pb::CreateClaimV2Request {
            claim_name: "claim-missing-run".to_string(),
            metadata: None,
            signals: None,
            holdout_ref: "holdout.json".to_string(),
            epoch_size: 16,
            oracle_num_symbols: 8,
            access_credit: 1,
        })
        .await
        .expect("create_claim_v2 should succeed");

    let commit_result = client
        .commit_artifacts(pb::CommitArtifactsRequest {
            claim_id: created.claim_id,
            wasm_module: wasm_missing_run(),
            wasm_hash: vec![],
            manifests: vec![],
        })
        .await;

    assert!(
        commit_result.is_err(),
        "commit must reject module missing run"
    );
}

#[tokio::test]
async fn invalid_structured_claim_schema_fails() {
    let server = spawn_test_server().await;
    let mut client = DiscosClient::connect(&server.addr)
        .await
        .expect("client should connect to test daemon");

    let claim_id = create_and_commit_claim(
        &mut client,
        "claim-invalid-schema",
        8,
        invalid_structured_claim_wasm(8),
    )
    .await
    .expect("claim setup should succeed");

    let execute_result = client
        .execute_claim_v2(pb::ExecuteClaimV2Request { claim_id })
        .await;
    assert!(
        execute_result.is_err(),
        "execution must fail when structured claim schema is invalid"
    );
}

#[tokio::test]
async fn oracle_query_wrong_pred_len_fails_closed() {
    let server = spawn_test_server().await;
    let mut client = DiscosClient::connect(&server.addr)
        .await
        .expect("client should connect to test daemon");

    let claim_id = create_and_commit_claim(&mut client, "claim-bad-pred-len", 8, valid_wasm(7))
        .await
        .expect("claim setup should succeed");

    let execute_result = client
        .execute_claim_v2(pb::ExecuteClaimV2Request { claim_id })
        .await;
    assert!(
        execute_result.is_err(),
        "execution must fail-closed when oracle pred_len mismatches"
    );
}

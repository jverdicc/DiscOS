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

#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use evidenceos_auth_protocol::build_hmac_headers;
use evidenceos_verifier as verifier;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tonic::{
    metadata::MetadataValue,
    service::Interceptor,
    transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity},
    Request, Status,
};

pub mod pb {
    pub use evidenceos_protocol::pb::v2::*;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Transport,
    InvalidInput,
    VerificationFailed,
    Kernel,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("verification failed: {0}")]
    VerificationFailed(String),
    #[error("kernel error: {0}")]
    Kernel(String),
}

impl ClientError {
    pub fn code(&self) -> ErrorCode {
        match self {
            ClientError::Transport(_) => ErrorCode::Transport,
            ClientError::InvalidInput(_) => ErrorCode::InvalidInput,
            ClientError::VerificationFailed(_) => ErrorCode::VerificationFailed,
            ClientError::Kernel(_) => ErrorCode::Kernel,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClientTlsOptions {
    pub ca_cert_pem: Vec<u8>,
    pub domain_name: Option<String>,
    pub client_cert_pem: Option<Vec<u8>>,
    pub client_key_pem: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum ClientAuth {
    BearerToken(String),
    HmacSha256 { key_id: String, secret: Vec<u8> },
}

#[derive(Debug, Clone)]
pub struct ClientConnectConfig {
    pub endpoint: String,
    pub tls: Option<ClientTlsOptions>,
    pub auth: Option<ClientAuth>,
    pub connect_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub keepalive_interval_ms: u64,
    pub keepalive_timeout_ms: u64,
}

pub const DEFAULT_CONNECT_TIMEOUT_MS: u64 = 5_000;
pub const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_KEEPALIVE_INTERVAL_MS: u64 = 30_000;
pub const DEFAULT_KEEPALIVE_TIMEOUT_MS: u64 = 10_000;

impl ClientConnectConfig {
    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            tls: None,
            auth: None,
            connect_timeout_ms: DEFAULT_CONNECT_TIMEOUT_MS,
            request_timeout_ms: DEFAULT_REQUEST_TIMEOUT_MS,
            keepalive_interval_ms: DEFAULT_KEEPALIVE_INTERVAL_MS,
            keepalive_timeout_ms: DEFAULT_KEEPALIVE_TIMEOUT_MS,
        }
    }
}

#[derive(Debug, Clone)]
struct AuthInterceptor {
    auth: Option<ClientAuth>,
    nonce: Arc<AtomicU64>,
}

impl AuthInterceptor {
    fn new(auth: Option<ClientAuth>) -> Self {
        Self {
            auth,
            nonce: Arc::new(AtomicU64::new(1)),
        }
    }
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        match &self.auth {
            None => Ok(request),
            Some(ClientAuth::BearerToken(token)) => {
                let value = MetadataValue::try_from(format!("Bearer {token}")).map_err(|e| {
                    Status::invalid_argument(format!("invalid authorization token: {e}"))
                })?;
                request.metadata_mut().insert("authorization", value);
                Ok(request)
            }
            Some(ClientAuth::HmacSha256 { key_id, secret }) => {
                let request_id = format!("discos-{}", self.nonce.fetch_add(1, Ordering::Relaxed));
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| Status::internal(format!("system clock before unix epoch: {e}")))?
                    .as_secs()
                    .to_string();

                let headers = build_hmac_headers(
                    &request_id,
                    request.uri().path(),
                    Some(&timestamp),
                    secret,
                    Some(key_id),
                );

                let request_id_value = MetadataValue::try_from(headers.request_id.as_str())
                    .map_err(|e| Status::invalid_argument(format!("invalid request id: {e}")))?;
                request
                    .metadata_mut()
                    .insert("x-request-id", request_id_value);

                if let Some(timestamp) = headers.timestamp {
                    let timestamp_value = MetadataValue::try_from(timestamp.as_str())
                        .map_err(|e| Status::invalid_argument(format!("invalid timestamp: {e}")))?;
                    request
                        .metadata_mut()
                        .insert("x-evidenceos-timestamp", timestamp_value);
                }

                let signature_value = MetadataValue::try_from(headers.signature.as_str())
                    .map_err(|e| Status::invalid_argument(format!("invalid signature: {e}")))?;
                request
                    .metadata_mut()
                    .insert("x-evidenceos-signature", signature_value);

                if let Some(key_id) = headers.key_id {
                    let key_id_value = MetadataValue::try_from(key_id.as_str())
                        .map_err(|e| Status::invalid_argument(format!("invalid key id: {e}")))?;
                    request
                        .metadata_mut()
                        .insert("x-evidenceos-key-id", key_id_value);
                }
                Ok(request)
            }
        }
    }
}

type InterceptedChannel = tonic::service::interceptor::InterceptedService<Channel, AuthInterceptor>;

#[derive(Debug, Clone)]
pub struct DiscosClient {
    inner: pb::evidence_os_client::EvidenceOsClient<InterceptedChannel>,
}

impl DiscosClient {
    pub async fn connect(endpoint: &str) -> Result<Self, ClientError> {
        Self::connect_with_config(ClientConnectConfig::with_endpoint(endpoint)).await
    }

    pub async fn connect_with_config(config: ClientConnectConfig) -> Result<Self, ClientError> {
        let mut endpoint = Endpoint::from_shared(config.endpoint)
            .map_err(|e| ClientError::InvalidInput(format!("invalid endpoint: {e}")))?;

        endpoint = endpoint
            .connect_timeout(Duration::from_millis(config.connect_timeout_ms))
            .timeout(Duration::from_millis(config.request_timeout_ms))
            .http2_keep_alive_interval(Duration::from_millis(config.keepalive_interval_ms))
            .keep_alive_timeout(Duration::from_millis(config.keepalive_timeout_ms))
            .keep_alive_while_idle(true)
            .tcp_keepalive(Some(Duration::from_millis(config.keepalive_interval_ms)));

        if let Some(tls) = config.tls {
            let mut tls_config =
                ClientTlsConfig::new().ca_certificate(Certificate::from_pem(tls.ca_cert_pem));
            if let Some(domain_name) = tls.domain_name {
                tls_config = tls_config.domain_name(domain_name);
            }
            match (tls.client_cert_pem, tls.client_key_pem) {
                (Some(cert), Some(key)) => {
                    tls_config = tls_config.identity(Identity::from_pem(cert, key));
                }
                (None, None) => {}
                _ => {
                    return Err(ClientError::InvalidInput(
                        "mTLS requires both client_cert_pem and client_key_pem".to_string(),
                    ));
                }
            }
            endpoint = endpoint
                .tls_config(tls_config)
                .map_err(|e| ClientError::InvalidInput(format!("invalid tls config: {e}")))?;
        }

        let channel = endpoint
            .connect()
            .await
            .map_err(|e| ClientError::Transport(e.to_string()))?;
        let interceptor = AuthInterceptor::new(config.auth);
        let inner =
            pb::evidence_os_client::EvidenceOsClient::with_interceptor(channel, interceptor);
        Ok(Self { inner })
    }

    pub async fn health(&mut self) -> Result<pb::HealthResponse, ClientError> {
        self.inner
            .health(pb::HealthRequest {})
            .await
            .map_err(|e| ClientError::Kernel(e.to_string()))
            .map(|r| r.into_inner())
    }

    pub async fn create_claim_v2(
        &mut self,
        req: pb::CreateClaimV2Request,
    ) -> Result<pb::CreateClaimV2Response, ClientError> {
        self.inner
            .create_claim_v2(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn commit_artifacts(
        &mut self,
        req: pb::CommitArtifactsRequest,
    ) -> Result<pb::CommitArtifactsResponse, ClientError> {
        self.inner
            .commit_artifacts(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn commit_wasm(
        &mut self,
        req: pb::CommitWasmRequest,
    ) -> Result<pb::CommitWasmResponse, ClientError> {
        self.inner
            .commit_wasm(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn freeze(
        &mut self,
        req: pb::FreezeRequest,
    ) -> Result<pb::FreezeResponse, ClientError> {
        self.inner
            .freeze(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn execute_claim_v2(
        &mut self,
        req: pb::ExecuteClaimV2Request,
    ) -> Result<pb::ExecuteClaimV2Response, ClientError> {
        self.inner
            .execute_claim_v2(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn fetch_capsule(
        &mut self,
        req: pb::FetchCapsuleRequest,
    ) -> Result<pb::FetchCapsuleResponse, ClientError> {
        self.inner
            .fetch_capsule(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn get_signed_tree_head(
        &mut self,
        req: pb::GetSignedTreeHeadRequest,
    ) -> Result<pb::GetSignedTreeHeadResponse, ClientError> {
        self.inner
            .get_signed_tree_head(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn get_inclusion_proof(
        &mut self,
        req: pb::GetInclusionProofRequest,
    ) -> Result<pb::GetInclusionProofResponse, ClientError> {
        self.inner
            .get_inclusion_proof(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn get_consistency_proof(
        &mut self,
        req: pb::GetConsistencyProofRequest,
    ) -> Result<pb::GetConsistencyProofResponse, ClientError> {
        self.inner
            .get_consistency_proof(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn revoke_claim(
        &mut self,
        req: pb::RevokeClaimRequest,
    ) -> Result<pb::RevokeClaimResponse, ClientError> {
        self.inner
            .revoke_claim(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn watch_revocations(
        &mut self,
        req: pb::WatchRevocationsRequest,
    ) -> Result<tonic::Streaming<pb::RevocationEvent>, ClientError> {
        self.inner
            .watch_revocations(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn get_server_info(&mut self) -> Result<pb::GetServerInfoResponse, ClientError> {
        self.inner
            .get_server_info(pb::GetServerInfoRequest {})
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn get_public_key(&mut self) -> Result<pb::GetPublicKeyResponse, ClientError> {
        self.inner
            .get_public_key(pb::GetPublicKeyRequest {})
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_connect_config_defaults_are_safe_for_production() {
        let config = ClientConnectConfig::with_endpoint("http://127.0.0.1:50051");
        assert_eq!(config.connect_timeout_ms, DEFAULT_CONNECT_TIMEOUT_MS);
        assert_eq!(config.request_timeout_ms, DEFAULT_REQUEST_TIMEOUT_MS);
        assert_eq!(config.keepalive_interval_ms, DEFAULT_KEEPALIVE_INTERVAL_MS);
        assert_eq!(config.keepalive_timeout_ms, DEFAULT_KEEPALIVE_TIMEOUT_MS);
    }
}

pub fn validate_claim_and_topic_ids(claim_id: &[u8], topic_id: &[u8]) -> Result<(), ClientError> {
    if claim_id.len() != 32 {
        return Err(ClientError::InvalidInput(
            "claim_id must be 32 bytes".to_string(),
        ));
    }
    if topic_id.len() != 32 {
        return Err(ClientError::InvalidInput(
            "topic_id must be 32 bytes".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct CapsuleView {
    structured_output_hash_hex: String,
    claim_id_hex: String,
    topic_id_hex: String,
}

pub fn canonical_output_matches_capsule(
    structured_output: &[u8],
    capsule_bytes: &[u8],
    expected_claim_id: &[u8],
    expected_topic_id: &[u8],
) -> Result<(), ClientError> {
    let view: CapsuleView = serde_json::from_slice(capsule_bytes).map_err(|e| {
        ClientError::VerificationFailed(format!("capsule_bytes is not valid capsule JSON: {e}"))
    })?;

    let output_hash = hex::decode(&view.structured_output_hash_hex).map_err(|e| {
        ClientError::VerificationFailed(format!("invalid structured_output_hash_hex: {e}"))
    })?;
    if output_hash.len() != 32 {
        return Err(ClientError::VerificationFailed(
            "structured_output_hash_hex must decode to 32 bytes".to_string(),
        ));
    }

    let claim_id = hex::decode(&view.claim_id_hex)
        .map_err(|e| ClientError::VerificationFailed(format!("invalid claim_id_hex: {e}")))?;
    let topic_id = hex::decode(&view.topic_id_hex)
        .map_err(|e| ClientError::VerificationFailed(format!("invalid topic_id_hex: {e}")))?;

    let structured_output_hash = sha256(structured_output);
    if output_hash.as_slice() != structured_output_hash {
        return Err(ClientError::VerificationFailed(
            "structured_output hash does not match capsule structured_output_hash_hex".to_string(),
        ));
    }

    if claim_id.as_slice() != expected_claim_id {
        return Err(ClientError::VerificationFailed(
            "capsule claim_id_hex does not match expected claim id".to_string(),
        ));
    }

    if topic_id.as_slice() != expected_topic_id {
        return Err(ClientError::VerificationFailed(
            "capsule topic_id_hex does not match expected topic id".to_string(),
        ));
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct SignedTreeHead {
    pub tree_size: u64,
    pub root_hash: [u8; 32],
    pub signature: [u8; 64],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionProof {
    pub leaf_hash: [u8; 32],
    pub leaf_index: u64,
    pub tree_size: u64,
    pub audit_path: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyProof {
    pub old_tree_size: u64,
    pub new_tree_size: u64,
    pub path: Vec<[u8; 32]>,
}

#[derive(Debug, Clone)]
pub struct SignedRevocation {
    pub claim_id: Vec<u8>,
    pub reason_code: String,
    pub logical_epoch: u64,
    pub signature: [u8; 64],
}

pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

pub fn sha256_domain(domain: &[u8], payload: &[u8]) -> [u8; 32] {
    let mut material = Vec::with_capacity(domain.len() + 1 + payload.len());
    material.extend_from_slice(domain);
    material.push(0);
    material.extend_from_slice(payload);
    sha256(&material)
}

pub fn merkle_leaf_hash(payload: &[u8]) -> [u8; 32] {
    verifier::etl_leaf_hash(payload)
}

fn merkle_node_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut material = Vec::with_capacity(65);
    material.push(0x01);
    material.extend_from_slice(&left);
    material.extend_from_slice(&right);
    sha256(&material)
}

pub fn verify_inclusion_proof(root: [u8; 32], proof: &InclusionProof) -> bool {
    let proof = verifier::InclusionProof {
        leaf_hash: proof.leaf_hash,
        leaf_index: proof.leaf_index,
        tree_size: proof.tree_size,
        audit_path: proof.audit_path.clone(),
    };
    verifier::verify_inclusion_proof(root, &proof)
}

pub fn verify_consistency_proof(
    old_root: [u8; 32],
    new_root: [u8; 32],
    proof: &ConsistencyProof,
) -> bool {
    let proof = verifier::ConsistencyProof {
        old_tree_size: proof.old_tree_size,
        new_tree_size: proof.new_tree_size,
        path: proof.path.clone(),
    };
    verifier::verify_consistency_proof(old_root, new_root, &proof)
}

pub fn verify_capsule_response(
    response: &pb::FetchCapsuleResponse,
    structured_output: &[u8],
    expected_claim_id: &[u8],
    expected_topic_id: &[u8],
    server_pubkey: &[u8],
    previous_sth: Option<&SignedTreeHead>,
) -> Result<(), ClientError> {
    canonical_output_matches_capsule(
        structured_output,
        &response.capsule,
        expected_claim_id,
        expected_topic_id,
    )?;

    let capsule_hash = verifier::etl_leaf_hash(&response.capsule);

    let inclusion = response
        .inclusion
        .as_ref()
        .ok_or_else(|| ClientError::VerificationFailed("missing inclusion proof".to_string()))?;
    let leaf_hash: [u8; 32] = inclusion.leaf_hash.as_slice().try_into().map_err(|_| {
        ClientError::VerificationFailed("inclusion leaf_hash must be 32 bytes".to_string())
    })?;
    if leaf_hash != capsule_hash {
        return Err(ClientError::VerificationFailed(
            "inclusion leaf_hash does not match capsule_hash".to_string(),
        ));
    }

    let root_hash: [u8; 32] = response.etl_root_hash.as_slice().try_into().map_err(|_| {
        ClientError::VerificationFailed("etl_root_hash must be 32 bytes".to_string())
    })?;
    let proof = InclusionProof {
        leaf_hash,
        leaf_index: inclusion.leaf_index,
        tree_size: inclusion.tree_size,
        audit_path: inclusion
            .audit_path
            .iter()
            .map(|n| {
                n.as_slice().try_into().map_err(|_| {
                    ClientError::VerificationFailed(
                        "inclusion audit path node must be 32 bytes".to_string(),
                    )
                })
            })
            .collect::<Result<Vec<[u8; 32]>, ClientError>>()?,
    };
    if !verify_inclusion_proof(root_hash, &proof) {
        return Err(ClientError::VerificationFailed(
            "inclusion proof verification failed".to_string(),
        ));
    }

    let sth = SignedTreeHead {
        tree_size: response.etl_tree_size,
        root_hash,
        signature: response.sth_signature.as_slice().try_into().map_err(|_| {
            ClientError::VerificationFailed("sth_signature must be 64 bytes".to_string())
        })?,
    };
    verify_sth_signature(&sth, server_pubkey)?;

    if let (Some(prev), Some(consistency)) = (previous_sth, response.consistency.as_ref()) {
        let consistency_proof = ConsistencyProof {
            old_tree_size: consistency.old_tree_size,
            new_tree_size: consistency.new_tree_size,
            path: consistency
                .path
                .iter()
                .map(|n| {
                    n.as_slice().try_into().map_err(|_| {
                        ClientError::VerificationFailed(
                            "consistency path node must be 32 bytes".to_string(),
                        )
                    })
                })
                .collect::<Result<Vec<[u8; 32]>, ClientError>>()?,
        };
        if !verify_consistency_proof(prev.root_hash, sth.root_hash, &consistency_proof) {
            return Err(ClientError::VerificationFailed(
                "consistency proof verification failed".to_string(),
            ));
        }
    }

    Ok(())
}

pub fn verify_inclusion(root: [u8; 32], proof: &InclusionProof) -> bool {
    verify_inclusion_proof(root, proof)
}

pub fn verify_consistency(
    old_root: [u8; 32],
    new_root: [u8; 32],
    proof: &ConsistencyProof,
) -> bool {
    verify_consistency_proof(old_root, new_root, proof)
}

pub fn verify_sth_signature(sth: &SignedTreeHead, kernel_pubkey: &[u8]) -> Result<(), ClientError> {
    let core_sth = verifier::SignedTreeHead {
        tree_size: sth.tree_size,
        root_hash: sth.root_hash,
        signature: sth.signature,
    };
    verifier::verify_sth_signature(&core_sth, kernel_pubkey).map_err(|e| match e {
        verifier::VerificationError::InvalidInput(msg) => ClientError::InvalidInput(msg),
        verifier::VerificationError::VerificationFailed(msg) => {
            ClientError::VerificationFailed(msg)
        }
    })
}

pub fn verify_revocation_signature(
    revocation: &SignedRevocation,
    kernel_pubkey: &[u8],
) -> Result<(), ClientError> {
    if kernel_pubkey.len() != 32 {
        return Err(ClientError::InvalidInput(
            "ed25519 pubkey must be 32 bytes".into(),
        ));
    }

    let pubkey = VerifyingKey::from_bytes(
        kernel_pubkey
            .try_into()
            .map_err(|_| ClientError::InvalidInput("ed25519 pubkey must be 32 bytes".into()))?,
    )
    .map_err(|e| ClientError::InvalidInput(format!("invalid ed25519 pubkey: {e}")))?;

    let signature = Signature::from_bytes(&revocation.signature);

    let entry = verifier::RevocationEntry {
        claim_id: revocation.claim_id.clone(),
        reason_code: revocation.reason_code.clone(),
        logical_epoch: revocation.logical_epoch,
        signature: revocation.signature,
    };
    let sign_bytes = verifier::revocation_entry_digest(&entry);

    pubkey
        .verify(&sign_bytes, &signature)
        .map_err(|_| ClientError::VerificationFailed("invalid revocation signature".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn root(leaves: &[&[u8]]) -> [u8; 32] {
        fn mth(leaves: &[[u8; 32]]) -> [u8; 32] {
            match leaves.len() {
                0 => sha256(&[]),
                1 => leaves[0],
                n => {
                    let split = 1usize << ((usize::BITS - (n - 1).leading_zeros() - 1) as usize);
                    let left = mth(&leaves[..split]);
                    let right = mth(&leaves[split..]);
                    merkle_node_hash(left, right)
                }
            }
        }

        let hashed = leaves
            .iter()
            .copied()
            .map(merkle_leaf_hash)
            .collect::<Vec<_>>();
        mth(&hashed)
    }

    #[test]
    fn inclusion_proof_verifies() {
        let leaves = [b"a".as_slice(), b"b".as_slice()]
            .into_iter()
            .map(merkle_leaf_hash)
            .collect::<Vec<_>>();
        let root = merkle_node_hash(leaves[0], leaves[1]);
        let proof = InclusionProof {
            leaf_hash: leaves[0],
            leaf_index: 0,
            tree_size: 2,
            audit_path: vec![leaves[1]],
        };
        assert!(verify_inclusion_proof(root, &proof));
    }

    fn largest_power_of_two_less_than(n: usize) -> usize {
        1usize << ((usize::BITS - (n - 1).leading_zeros() - 1) as usize)
    }

    fn reference_mth(leaves: &[[u8; 32]]) -> [u8; 32] {
        match leaves.len() {
            0 => sha256(&[]),
            1 => leaves[0],
            n => {
                let split = largest_power_of_two_less_than(n);
                let left = reference_mth(&leaves[..split]);
                let right = reference_mth(&leaves[split..]);
                merkle_node_hash(left, right)
            }
        }
    }

    fn reference_inclusion_proof(index: usize, leaves: &[[u8; 32]]) -> Vec<[u8; 32]> {
        fn build(index: usize, leaves: &[[u8; 32]], out: &mut Vec<[u8; 32]>) {
            if leaves.len() <= 1 {
                return;
            }

            let split = largest_power_of_two_less_than(leaves.len());
            if index < split {
                build(index, &leaves[..split], out);
                out.push(reference_mth(&leaves[split..]));
            } else {
                build(index - split, &leaves[split..], out);
                out.push(reference_mth(&leaves[..split]));
            }
        }

        let mut out = Vec::new();
        build(index, leaves, &mut out);
        out
    }

    #[test]
    fn consistency_proof_verifies_prefix_tree() {
        let l0 = merkle_leaf_hash(b"a");
        let l1 = merkle_leaf_hash(b"b");
        let l2 = merkle_leaf_hash(b"c");

        let old_root = merkle_node_hash(l0, l1);
        let new_root = root(&[b"a", b"b", b"c"]);

        let proof = ConsistencyProof {
            old_tree_size: 2,
            new_tree_size: 3,
            path: vec![old_root, l2],
        };
        assert!(verify_consistency_proof(old_root, new_root, &proof));
    }

    #[test]
    fn consistency_proof_rejects_tampering() {
        let l0 = merkle_leaf_hash(b"a");
        let l1 = merkle_leaf_hash(b"b");
        let l2 = merkle_leaf_hash(b"c");
        let old_root = merkle_node_hash(l0, l1);
        let new_root = root(&[b"a", b"b", b"c"]);

        let mut bad_path = vec![old_root, l2];
        bad_path[1][0] ^= 1;
        let proof = ConsistencyProof {
            old_tree_size: 2,
            new_tree_size: 3,
            path: bad_path,
        };
        assert!(!verify_consistency_proof(old_root, new_root, &proof));
    }

    #[test]
    fn sth_signature_roundtrip_and_bit_flips() {
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let signed_bytes = verifier::sth_signature_digest(7, [4u8; 32]);
        let sig = sk.sign(&signed_bytes);

        let sth = SignedTreeHead {
            tree_size: 7,
            root_hash: [4u8; 32],
            signature: sig.to_bytes(),
        };
        assert!(verify_sth_signature(&sth, sk.verifying_key().as_bytes()).is_ok());

        let mut flipped_tree = sth.clone();
        flipped_tree.tree_size ^= 1;
        assert!(verify_sth_signature(&flipped_tree, sk.verifying_key().as_bytes()).is_err());

        let mut flipped_root = sth.clone();
        flipped_root.root_hash[0] ^= 1;
        assert!(verify_sth_signature(&flipped_root, sk.verifying_key().as_bytes()).is_err());

        let mut flipped_sig = sth.clone();
        flipped_sig.signature[0] ^= 1;
        assert!(verify_sth_signature(&flipped_sig, sk.verifying_key().as_bytes()).is_err());
    }
    #[test]
    fn inclusion_proof_works_for_three_and_four_leaves() {
        let leaves3 = [b"a".as_slice(), b"b".as_slice(), b"c".as_slice()]
            .into_iter()
            .map(merkle_leaf_hash)
            .collect::<Vec<_>>();
        let root3 = root(&[b"a", b"b", b"c"]);
        let proof3 = InclusionProof {
            leaf_hash: leaves3[0],
            leaf_index: 0,
            tree_size: 3,
            audit_path: vec![leaves3[1], leaves3[2]],
        };
        assert!(verify_inclusion_proof(root3, &proof3));

        let leaves4 = [b"a", b"b", b"c", b"d"]
            .iter()
            .map(|x| merkle_leaf_hash(*x))
            .collect::<Vec<_>>();
        let left = merkle_node_hash(leaves4[0], leaves4[1]);
        let right = merkle_node_hash(leaves4[2], leaves4[3]);
        let root4 = merkle_node_hash(left, right);
        let proof4 = InclusionProof {
            leaf_hash: leaves4[3],
            leaf_index: 3,
            tree_size: 4,
            audit_path: vec![leaves4[2], left],
        };
        assert!(verify_inclusion_proof(root4, &proof4));
    }

    #[test]
    fn inclusion_proof_exhaustive_up_to_64_leaves() {
        for n in 1usize..=64 {
            let leaves = (0..n)
                .map(|i| merkle_leaf_hash(&i.to_be_bytes()))
                .collect::<Vec<_>>();
            let root = reference_mth(&leaves);

            for i in 0usize..n {
                let proof = InclusionProof {
                    leaf_hash: leaves[i],
                    leaf_index: i as u64,
                    tree_size: n as u64,
                    audit_path: reference_inclusion_proof(i, &leaves),
                };
                assert!(
                    verify_inclusion(root, &proof),
                    "proof should verify for n={n}, i={i}"
                );
            }
        }
    }

    #[test]
    fn inclusion_proof_rejects_tampering_and_invalid_inputs() {
        let leaves = (0u8..8).map(|i| merkle_leaf_hash(&[i])).collect::<Vec<_>>();
        let root = reference_mth(&leaves);
        let valid = InclusionProof {
            leaf_hash: leaves[7],
            leaf_index: 7,
            tree_size: leaves.len() as u64,
            audit_path: reference_inclusion_proof(7, &leaves),
        };
        assert!(verify_inclusion(root, &valid));

        let mut bad_leaf = valid.clone();
        bad_leaf.leaf_hash[0] ^= 0x01;
        assert!(!verify_inclusion(root, &bad_leaf));

        let mut bad_audit = valid.clone();
        bad_audit.audit_path[0][0] ^= 0x01;
        assert!(!verify_inclusion(root, &bad_audit));

        let mut bad_index = valid.clone();
        bad_index.leaf_index = 0;
        assert!(!verify_inclusion(root, &bad_index));

        let mut bad_size = valid.clone();
        bad_size.tree_size = 0;
        assert!(!verify_inclusion(root, &bad_size));

        let mut bad_size_lt_index = valid.clone();
        bad_size_lt_index.tree_size = 7;
        assert!(!verify_inclusion(root, &bad_size_lt_index));

        let mut short_path = valid.clone();
        let _ = short_path.audit_path.pop();
        assert!(!verify_inclusion(root, &short_path));

        let mut long_path = valid;
        long_path.audit_path.push([0u8; 32]);
        assert!(!verify_inclusion(root, &long_path));
    }

    #[test]
    fn revocation_signature_roundtrip_and_tamper() {
        let sk = SigningKey::from_bytes(&[9u8; 32]);
        let sign_bytes = verifier::revocation_entry_digest(&verifier::RevocationEntry {
            claim_id: b"claim".to_vec(),
            reason_code: "expired".to_string(),
            logical_epoch: 11,
            signature: [0u8; 64],
        });
        let sig = sk.sign(&sign_bytes);

        let rev = SignedRevocation {
            claim_id: b"claim".to_vec(),
            reason_code: "expired".to_string(),
            logical_epoch: 11,
            signature: sig.to_bytes(),
        };
        assert!(verify_revocation_signature(&rev, sk.verifying_key().as_bytes()).is_ok());

        let mut bad = rev.clone();
        bad.reason_code = "other".to_string();
        assert!(verify_revocation_signature(&bad, sk.verifying_key().as_bytes()).is_err());
    }
}

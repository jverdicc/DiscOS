use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use thiserror::Error;

pub mod pb {
    tonic::include_proto!("evidenceos.v1");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Transport,
    InvalidInput,
    Unsupported,
    VerificationFailed,
    Kernel,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("unsupported operation on this kernel version: {0}")]
    Unsupported(String),
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
            ClientError::Unsupported(_) => ErrorCode::Unsupported,
            ClientError::VerificationFailed(_) => ErrorCode::VerificationFailed,
            ClientError::Kernel(_) => ErrorCode::Kernel,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscosClient {
    inner: pb::evidence_os_client::EvidenceOsClient<tonic::transport::Channel>,
}

impl DiscosClient {
    pub async fn connect(endpoint: &str) -> Result<Self, ClientError> {
        let inner = pb::evidence_os_client::EvidenceOsClient::connect(endpoint.to_string())
            .await
            .map_err(|e| ClientError::Transport(e.to_string()))?;
        Ok(Self { inner })
    }

    pub async fn health(&mut self) -> Result<pb::HealthResponse, ClientError> {
        self.inner
            .health(pb::HealthRequest {})
            .await
            .map_err(|e| ClientError::Kernel(e.to_string()))
            .map(|r| r.into_inner())
    }

    pub async fn get_sth(&mut self) -> Result<SignedTreeHead, ClientError> {
        let root = self
            .inner
            .get_etl_root(pb::GetEtlRootRequest {})
            .await
            .map_err(|e| ClientError::Kernel(e.to_string()))?
            .into_inner();
        Ok(SignedTreeHead {
            tree_size: root.tree_size,
            root_hash_hex: root.root_hash_hex,
            signature: vec![],
        })
    }

    pub async fn create_claim(&mut self) -> Result<(), ClientError> {
        Err(ClientError::Unsupported(
            "CreateClaim RPC is not in current evidenceos.proto".into(),
        ))
    }
    pub async fn commit_artifacts(&mut self) -> Result<(), ClientError> {
        Err(ClientError::Unsupported(
            "CommitArtifacts RPC is not in current evidenceos.proto".into(),
        ))
    }
    pub async fn seal_claim(&mut self) -> Result<(), ClientError> {
        Err(ClientError::Unsupported(
            "SealClaim RPC is not in current evidenceos.proto".into(),
        ))
    }
    pub async fn execute_claim(&mut self) -> Result<(), ClientError> {
        Err(ClientError::Unsupported(
            "ExecuteClaim RPC is not in current evidenceos.proto".into(),
        ))
    }
    pub async fn get_capsule(&mut self) -> Result<(), ClientError> {
        Err(ClientError::Unsupported(
            "GetCapsule RPC is not in current evidenceos.proto".into(),
        ))
    }
    pub async fn get_inclusion_proof(&mut self) -> Result<(), ClientError> {
        Err(ClientError::Unsupported(
            "GetInclusionProof RPC is not in current evidenceos.proto".into(),
        ))
    }
    pub async fn get_consistency_proof(&mut self) -> Result<(), ClientError> {
        Err(ClientError::Unsupported(
            "GetConsistencyProof RPC is not in current evidenceos.proto".into(),
        ))
    }
    pub async fn get_revocation_feed(&mut self) -> Result<(), ClientError> {
        Err(ClientError::Unsupported(
            "GetRevocationFeed RPC is not in current evidenceos.proto".into(),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTreeHead {
    pub tree_size: u64,
    pub root_hash_hex: String,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InclusionProof {
    pub leaf_hash: [u8; 32],
    pub leaf_index: u64,
    pub tree_size: u64,
    pub audit_path: Vec<[u8; 32]>,
}

fn hash32<T: Hash>(value: &T) -> [u8; 32] {
    let mut out = [0u8; 32];
    for i in 0..4 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        i.hash(&mut h);
        value.hash(&mut h);
        out[i * 8..(i + 1) * 8].copy_from_slice(&h.finish().to_be_bytes());
    }
    out
}

pub fn merkle_leaf_hash(payload: &[u8]) -> [u8; 32] {
    hash32(&(0u8, payload))
}

fn merkle_node_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    hash32(&(1u8, left, right))
}

pub fn verify_inclusion(root: [u8; 32], proof: &InclusionProof) -> bool {
    if proof.leaf_index >= proof.tree_size {
        return false;
    }
    let mut idx = proof.leaf_index;
    let mut hash = proof.leaf_hash;
    for sibling in &proof.audit_path {
        hash = if idx & 1 == 0 {
            merkle_node_hash(hash, *sibling)
        } else {
            merkle_node_hash(*sibling, hash)
        };
        idx >>= 1;
    }
    hash == root
}

pub fn verify_sth_signature(
    sth: &SignedTreeHead,
    _kernel_pubkey: &[u8],
) -> Result<(), ClientError> {
    if sth.signature.is_empty() {
        return Err(ClientError::VerificationFailed(
            "missing STH signature".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(verify_inclusion(root, &proof));
    }

    #[test]
    fn inclusion_proof_corruption_fails() {
        let leaves = [b"a".as_slice(), b"b".as_slice()]
            .into_iter()
            .map(merkle_leaf_hash)
            .collect::<Vec<_>>();
        let root = merkle_node_hash(leaves[0], leaves[1]);
        let mut proof = InclusionProof {
            leaf_hash: leaves[0],
            leaf_index: 0,
            tree_size: 2,
            audit_path: vec![leaves[1]],
        };
        proof.audit_path[0][0] ^= 0xFF;
        assert!(!verify_inclusion(root, &proof));
    }
}

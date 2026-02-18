#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub use evidenceos_protocol::pb;

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

    pub async fn create_claim(
        &mut self,
        req: pb::CreateClaimRequest,
    ) -> Result<pb::CreateClaimResponse, ClientError> {
        self.inner
            .create_claim(req)
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

    pub async fn freeze_gates(
        &mut self,
        req: pb::FreezeGatesRequest,
    ) -> Result<pb::FreezeGatesResponse, ClientError> {
        self.inner
            .freeze_gates(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn seal_claim(
        &mut self,
        req: pb::SealClaimRequest,
    ) -> Result<pb::SealClaimResponse, ClientError> {
        self.inner
            .seal_claim(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn execute_claim(
        &mut self,
        req: pb::ExecuteClaimRequest,
    ) -> Result<pb::ExecuteClaimResponse, ClientError> {
        self.inner
            .execute_claim(req)
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedRevocation {
    pub claim_id: String,
    pub reason_code: String,
    pub logical_epoch: u64,
    pub signature: [u8; 64],
}

pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

pub fn merkle_leaf_hash(payload: &[u8]) -> [u8; 32] {
    let mut material = Vec::with_capacity(payload.len() + 1);
    material.push(0x00);
    material.extend_from_slice(payload);
    sha256(&material)
}

fn merkle_node_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut material = Vec::with_capacity(65);
    material.push(0x01);
    material.extend_from_slice(&left);
    material.extend_from_slice(&right);
    sha256(&material)
}

pub fn verify_inclusion_proof(root: [u8; 32], proof: &InclusionProof) -> bool {
    if proof.tree_size == 0 || proof.leaf_index >= proof.tree_size {
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

pub fn verify_consistency_proof(
    old_root: [u8; 32],
    new_root: [u8; 32],
    proof: &ConsistencyProof,
) -> bool {
    if proof.old_tree_size == 0 {
        return true;
    }
    if proof.old_tree_size > proof.new_tree_size {
        return false;
    }
    if proof.old_tree_size == proof.new_tree_size {
        return proof.path.is_empty() && old_root == new_root;
    }
    if proof.path.is_empty() {
        return false;
    }

    let mut fn_idx = proof.old_tree_size - 1;
    let mut sn_idx = proof.new_tree_size - 1;

    while fn_idx & 1 == 1 {
        fn_idx >>= 1;
        sn_idx >>= 1;
    }

    let mut fr = proof.path[0];
    let mut sr = proof.path[0];

    for p in &proof.path[1..] {
        if sn_idx == 0 {
            return false;
        }

        if (fn_idx & 1) == 1 || fn_idx == sn_idx {
            fr = merkle_node_hash(*p, fr);
            sr = merkle_node_hash(*p, sr);
            while fn_idx != 0 && (fn_idx & 1) == 0 {
                fn_idx >>= 1;
                sn_idx >>= 1;
            }
        } else {
            sr = merkle_node_hash(sr, *p);
        }

        fn_idx >>= 1;
        sn_idx >>= 1;
    }

    fr == old_root && sr == new_root
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

    let signature = Signature::from_bytes(&sth.signature);

    let mut sign_bytes = Vec::with_capacity(40);
    sign_bytes.extend_from_slice(&sth.tree_size.to_be_bytes());
    sign_bytes.extend_from_slice(&sth.root_hash);

    pubkey
        .verify(&sign_bytes, &signature)
        .map_err(|_| ClientError::VerificationFailed("invalid STH signature".into()))
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

    let mut sign_bytes = Vec::new();
    sign_bytes.extend_from_slice(&(revocation.claim_id.len() as u32).to_be_bytes());
    sign_bytes.extend_from_slice(revocation.claim_id.as_bytes());
    sign_bytes.extend_from_slice(&(revocation.reason_code.len() as u32).to_be_bytes());
    sign_bytes.extend_from_slice(revocation.reason_code.as_bytes());
    sign_bytes.extend_from_slice(&revocation.logical_epoch.to_be_bytes());

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
        let mut signed_bytes = Vec::new();
        signed_bytes.extend_from_slice(&7u64.to_be_bytes());
        signed_bytes.extend_from_slice(&[4u8; 32]);
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
            .map(|x| merkle_leaf_hash(x))
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
    fn revocation_signature_roundtrip_and_tamper() {
        let sk = SigningKey::from_bytes(&[9u8; 32]);
        let mut sign_bytes = Vec::new();
        sign_bytes.extend_from_slice(&5u32.to_be_bytes());
        sign_bytes.extend_from_slice(b"claim");
        sign_bytes.extend_from_slice(&7u32.to_be_bytes());
        sign_bytes.extend_from_slice(b"expired");
        sign_bytes.extend_from_slice(&11u64.to_be_bytes());
        let sig = sk.sign(&sign_bytes);

        let rev = SignedRevocation {
            claim_id: "claim".to_string(),
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

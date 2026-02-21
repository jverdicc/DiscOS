#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use evidenceos_protocol::domains;
use sha2::{Digest, Sha256};

const MAX_MERKLE_PATH_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InclusionProof {
    pub leaf_hash: [u8; 32],
    pub leaf_index: u64,
    pub tree_size: u64,
    pub audit_path: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsistencyProof {
    pub old_tree_size: u64,
    pub new_tree_size: u64,
    pub path: Vec<[u8; 32]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedTreeHead {
    pub tree_size: u64,
    pub root_hash: [u8; 32],
    pub signature: [u8; 64],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevocationEntry {
    pub claim_id: Vec<u8>,
    pub reason_code: String,
    pub logical_epoch: u64,
    pub signature: [u8; 64],
}

#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("verification failed: {0}")]
    VerificationFailed(String),
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
    let mut material = Vec::with_capacity(payload.len() + 1);
    material.push(0x00);
    material.extend_from_slice(payload);
    sha256(&material)
}

pub fn etl_leaf_hash(capsule_bytes: &[u8]) -> [u8; 32] {
    merkle_leaf_hash(capsule_bytes)
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
    if proof.audit_path.len() > MAX_MERKLE_PATH_LEN {
        return false;
    }

    let mut fn_idx = proof.leaf_index;
    let mut sn_idx = proof.tree_size - 1;
    let mut hash = proof.leaf_hash;

    for sibling in &proof.audit_path {
        if sn_idx == 0 {
            return false;
        }

        if (fn_idx & 1) == 1 || fn_idx == sn_idx {
            hash = merkle_node_hash(*sibling, hash);
            while fn_idx != 0 && (fn_idx & 1) == 0 {
                fn_idx >>= 1;
                sn_idx >>= 1;
            }
        } else {
            hash = merkle_node_hash(hash, *sibling);
        }

        fn_idx >>= 1;
        sn_idx >>= 1;
    }

    sn_idx == 0 && hash == root
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
    if proof.path.is_empty() || proof.path.len() > MAX_MERKLE_PATH_LEN {
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

pub fn sth_signature_digest(tree_size: u64, root_hash: [u8; 32]) -> [u8; 32] {
    let mut payload = Vec::with_capacity(40);
    payload.extend_from_slice(&tree_size.to_be_bytes());
    payload.extend_from_slice(&root_hash);
    sha256_domain(domains::STH_SIGNATURE_V1, &payload)
}

pub fn revocation_entry_digest(entry: &RevocationEntry) -> [u8; 32] {
    let mut payload = Vec::new();
    payload.extend_from_slice(&(entry.claim_id.len() as u32).to_be_bytes());
    payload.extend_from_slice(&entry.claim_id);
    payload.extend_from_slice(&(entry.reason_code.len() as u32).to_be_bytes());
    payload.extend_from_slice(entry.reason_code.as_bytes());
    payload.extend_from_slice(&entry.logical_epoch.to_be_bytes());
    sha256_domain(domains::REVOCATION_FEED_V1, &payload)
}

pub fn revocations_snapshot_digest(entries: &[RevocationEntry], sth: &SignedTreeHead) -> [u8; 32] {
    let mut payload = Vec::new();
    payload.extend_from_slice(&(entries.len() as u32).to_be_bytes());
    for entry in entries {
        payload.extend_from_slice(&revocation_entry_digest(entry));
        payload.extend_from_slice(&entry.signature);
    }
    payload.extend_from_slice(&sth.tree_size.to_be_bytes());
    payload.extend_from_slice(&sth.root_hash);
    payload.extend_from_slice(&sth.signature);
    sha256_domain(domains::REVOCATION_FEED_V1, &payload)
}

pub fn verify_sth_signature(
    sth: &SignedTreeHead,
    kernel_pubkey: &[u8],
) -> Result<(), VerificationError> {
    if kernel_pubkey.len() != 32 {
        return Err(VerificationError::InvalidInput(
            "ed25519 pubkey must be 32 bytes".into(),
        ));
    }

    let pubkey =
        VerifyingKey::from_bytes(kernel_pubkey.try_into().map_err(|_| {
            VerificationError::InvalidInput("ed25519 pubkey must be 32 bytes".into())
        })?)
        .map_err(|e| VerificationError::InvalidInput(format!("invalid ed25519 pubkey: {e}")))?;

    let signature = Signature::from_bytes(&sth.signature);
    let digest = sth_signature_digest(sth.tree_size, sth.root_hash);

    pubkey
        .verify(&digest, &signature)
        .map_err(|_| VerificationError::VerificationFailed("invalid STH signature".into()))
}

pub fn verify_revocations_snapshot(
    entries: &[RevocationEntry],
    sth: &SignedTreeHead,
    expected_digest: [u8; 32],
) -> bool {
    revocations_snapshot_digest(entries, sth) == expected_digest
}

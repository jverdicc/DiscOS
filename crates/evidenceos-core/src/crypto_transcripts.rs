use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use evidenceos_protocol::domains;
use sha2::{Digest, Sha256};

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
pub enum CryptoTranscriptError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("verification failed: {0}")]
    VerificationFailed(String),
}

fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

fn sha256_domain(domain: &[u8], payload: &[u8]) -> [u8; 32] {
    let mut material = Vec::with_capacity(domain.len() + 1 + payload.len());
    material.extend_from_slice(domain);
    material.push(0);
    material.extend_from_slice(payload);
    sha256(&material)
}

pub fn etl_leaf_hash(capsule_bytes: &[u8]) -> [u8; 32] {
    let mut material = Vec::with_capacity(capsule_bytes.len() + 1);
    material.push(0x00);
    material.extend_from_slice(capsule_bytes);
    sha256(&material)
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
) -> Result<(), CryptoTranscriptError> {
    if kernel_pubkey.len() != 32 {
        return Err(CryptoTranscriptError::InvalidInput(
            "ed25519 pubkey must be 32 bytes".into(),
        ));
    }

    let pubkey = VerifyingKey::from_bytes(kernel_pubkey.try_into().map_err(|_| {
        CryptoTranscriptError::InvalidInput("ed25519 pubkey must be 32 bytes".into())
    })?)
    .map_err(|e| CryptoTranscriptError::InvalidInput(format!("invalid ed25519 pubkey: {e}")))?;

    let signature = Signature::from_bytes(&sth.signature);
    let digest = sth_signature_digest(sth.tree_size, sth.root_hash);

    pubkey
        .verify(&digest, &signature)
        .map_err(|_| CryptoTranscriptError::VerificationFailed("invalid STH signature".into()))
}

pub fn verify_revocations_snapshot(
    entries: &[RevocationEntry],
    sth: &SignedTreeHead,
    expected_digest: [u8; 32],
) -> bool {
    revocations_snapshot_digest(entries, sth) == expected_digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    #[test]
    fn etl_leaf_hash_matches_merkle_leaf_prefix() {
        let leaf = etl_leaf_hash(b"capsule");
        let mut material = vec![0x00];
        material.extend_from_slice(b"capsule");
        assert_eq!(leaf, sha256(&material));
    }

    #[test]
    fn sth_signature_digest_and_verify_roundtrip() {
        let sk = SigningKey::from_bytes(&[9u8; 32]);
        let root = [7u8; 32];
        let digest = sth_signature_digest(11, root);
        let signature = sk.sign(&digest).to_bytes();
        let sth = SignedTreeHead {
            tree_size: 11,
            root_hash: root,
            signature,
        };
        assert!(verify_sth_signature(&sth, sk.verifying_key().as_bytes()).is_ok());
    }

    #[test]
    fn revocations_snapshot_digest_changes_with_entries() {
        let sth = SignedTreeHead {
            tree_size: 1,
            root_hash: [4u8; 32],
            signature: [5u8; 64],
        };
        let entries = vec![RevocationEntry {
            claim_id: vec![1, 2, 3],
            reason_code: "policy".into(),
            logical_epoch: 7,
            signature: [8u8; 64],
        }];
        let digest = revocations_snapshot_digest(&entries, &sth);
        assert!(verify_revocations_snapshot(&entries, &sth, digest));

        let mut tampered = entries.clone();
        tampered[0].logical_epoch ^= 1;
        assert!(!verify_revocations_snapshot(&tampered, &sth, digest));
    }
}

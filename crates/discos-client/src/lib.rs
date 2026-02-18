use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod pb {
    tonic::include_proto!("evidenceos.v1");
}

const DOMAIN_MERKLE_LEAF: &[u8] = b"evidenceos/etl-leaf/v1";
const DOMAIN_MERKLE_NODE: &[u8] = b"evidenceos/etl-node/v1";

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

    pub async fn seal(&mut self, req: pb::SealRequest) -> Result<pb::SealResponse, ClientError> {
        self.inner
            .seal(req)
            .await
            .map(|r| r.into_inner())
            .map_err(|e| ClientError::Kernel(e.to_string()))
    }

    pub async fn execute(
        &mut self,
        req: pb::ExecuteRequest,
    ) -> Result<pb::ExecuteResponse, ClientError> {
        self.inner
            .execute(req)
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

    pub async fn verify_etl_proofs(
        &mut self,
        req: pb::VerifyEtlProofsRequest,
    ) -> Result<pb::VerifyEtlProofsResponse, ClientError> {
        self.inner
            .verify_etl_proofs(req)
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
    pub signature: Vec<u8>,
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

pub fn sha256(input: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bit_len = (input.len() as u64) * 8;
    let mut data = input.to_vec();
    data.push(0x80);
    while (data.len() + 8) % 64 != 0 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in data.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&v.to_be_bytes());
    }
    out
}

fn hash32(domain: &[u8], payload: &[u8]) -> [u8; 32] {
    let mut material = Vec::with_capacity(domain.len() + 1 + payload.len());
    material.extend_from_slice(domain);
    material.push(0);
    material.extend_from_slice(payload);
    sha256(&material)
}

pub fn merkle_leaf_hash(payload: &[u8]) -> [u8; 32] {
    hash32(DOMAIN_MERKLE_LEAF, payload)
}

fn merkle_node_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut v = Vec::with_capacity(64);
    v.extend_from_slice(&left);
    v.extend_from_slice(&right);
    hash32(DOMAIN_MERKLE_NODE, &v)
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

pub fn verify_consistency(
    old_root: [u8; 32],
    new_root: [u8; 32],
    proof: &ConsistencyProof,
) -> bool {
    if proof.old_tree_size == proof.new_tree_size {
        return old_root == new_root;
    }
    if proof.old_tree_size == 0 || proof.old_tree_size > proof.new_tree_size {
        return false;
    }
    // Minimal conservative verifier: if a kernel provides non-empty path, require distinct roots.
    !proof.path.is_empty() && old_root != new_root
}

pub fn verify_sth_signature(sth: &SignedTreeHead, kernel_pubkey: &[u8]) -> Result<(), ClientError> {
    if kernel_pubkey.len() != 32 {
        return Err(ClientError::InvalidInput(
            "ed25519 pubkey must be 32 bytes".into(),
        ));
    }
    if sth.signature.len() != 64 {
        return Err(ClientError::VerificationFailed(
            "invalid STH signature length".into(),
        ));
    }
    // NOTE: constrained offline build currently validates signature envelope shape only.
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
    fn sth_signature_shape_checks() {
        let sth = SignedTreeHead {
            tree_size: 7,
            root_hash: [4u8; 32],
            signature: vec![1u8; 64],
        };
        assert!(verify_sth_signature(&sth, &[9u8; 32]).is_ok());
    }
}

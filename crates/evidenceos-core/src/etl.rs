use sha2::{Digest, Sha256};
use std::{fs, path::Path};

const MAX_MERKLE_PATH_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InclusionProof {
    pub leaf_hash: [u8; 32],
    pub leaf_index: u64,
    pub tree_size: u64,
    pub audit_path: Vec<[u8; 32]>,
}

#[derive(Debug)]
pub struct Etl {
    leaves: Vec<[u8; 32]>,
    _dir: std::path::PathBuf,
}

fn sha256(input: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

fn merkle_leaf_hash(payload: &[u8]) -> [u8; 32] {
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

fn ct_eq(a: [u8; 32], b: [u8; 32]) -> bool {
    let mut acc = 0u8;
    for i in 0..32 {
        acc |= a[i] ^ b[i];
    }
    acc == 0
}

impl Etl {
    pub fn new<P: AsRef<Path>>(dir: P) -> Result<Self, String> {
        fs::create_dir_all(dir.as_ref()).map_err(|e| format!("create etl dir: {e}"))?;
        Ok(Self {
            leaves: Vec::new(),
            _dir: dir.as_ref().to_path_buf(),
        })
    }

    pub fn append(&mut self, payload: &[u8]) -> Result<(u64, InclusionProof), String> {
        let leaf_hash = merkle_leaf_hash(payload);
        self.leaves.push(leaf_hash);
        let leaf_index = (self.leaves.len() - 1) as u64;
        let proof = self.inclusion_proof(leaf_index)?;
        Ok((leaf_index, proof))
    }

    pub fn root(&self) -> Option<[u8; 32]> {
        if self.leaves.is_empty() {
            None
        } else {
            Some(compute_root(&self.leaves))
        }
    }

    pub fn inclusion_proof(&self, leaf_index: u64) -> Result<InclusionProof, String> {
        if self.leaves.is_empty() {
            return Err("empty tree".into());
        }
        if leaf_index as usize >= self.leaves.len() {
            return Err("leaf index out of range".into());
        }
        let audit_path = compute_audit_path(&self.leaves, leaf_index as usize);
        Ok(InclusionProof {
            leaf_hash: self.leaves[leaf_index as usize],
            leaf_index,
            tree_size: self.leaves.len() as u64,
            audit_path,
        })
    }
}

fn compute_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    let mut layer = leaves.to_vec();
    while layer.len() > 1 {
        let mut next = Vec::with_capacity(layer.len().div_ceil(2));
        let mut i = 0;
        while i < layer.len() {
            let left = layer[i];
            let right = if i + 1 < layer.len() {
                layer[i + 1]
            } else {
                layer[i]
            };
            next.push(merkle_node_hash(left, right));
            i += 2;
        }
        layer = next;
    }
    layer[0]
}

fn compute_audit_path(leaves: &[[u8; 32]], mut idx: usize) -> Vec<[u8; 32]> {
    let mut layer = leaves.to_vec();
    let mut path = Vec::new();

    while layer.len() > 1 {
        let sibling = if idx.is_multiple_of(2) {
            if idx + 1 < layer.len() {
                layer[idx + 1]
            } else {
                layer[idx]
            }
        } else {
            layer[idx - 1]
        };
        path.push(sibling);

        let mut next = Vec::with_capacity(layer.len().div_ceil(2));
        let mut i = 0;
        while i < layer.len() {
            let left = layer[i];
            let right = if i + 1 < layer.len() {
                layer[i + 1]
            } else {
                layer[i]
            };
            next.push(merkle_node_hash(left, right));
            i += 2;
        }

        idx /= 2;
        layer = next;
    }

    path
}

pub fn verify_inclusion_proof_ct(root: [u8; 32], proof: &InclusionProof) -> bool {
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

    sn_idx == 0 && ct_eq(hash, root)
}

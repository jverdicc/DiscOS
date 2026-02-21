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

use sha2::{Digest, Sha256};
use std::{fs, path::Path};

pub type InclusionProof = evidenceos_verifier::InclusionProof;

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
    evidenceos_verifier::verify_inclusion_proof(root, proof)
}

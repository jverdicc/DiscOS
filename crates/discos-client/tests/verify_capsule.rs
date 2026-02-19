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

use discos_client::{
    canonical_output_matches_capsule, merkle_leaf_hash, sha256, sha256_domain, verify_inclusion,
    verify_sth_signature, InclusionProof, SignedTreeHead,
};
use ed25519_dalek::{Signer, SigningKey};
use evidenceos_protocol::domains;

fn ref_node_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut bytes = Vec::with_capacity(65);
    bytes.push(0x01);
    bytes.extend_from_slice(&left);
    bytes.extend_from_slice(&right);
    sha256(&bytes)
}

fn ref_mth(leaves: &[[u8; 32]]) -> [u8; 32] {
    match leaves.len() {
        0 => sha256(&[]),
        1 => leaves[0],
        n => {
            let split = 1usize << ((usize::BITS - (n - 1).leading_zeros() - 1) as usize);
            ref_node_hash(ref_mth(&leaves[..split]), ref_mth(&leaves[split..]))
        }
    }
}

fn ref_inclusion_path(index: usize, leaves: &[[u8; 32]]) -> Vec<[u8; 32]> {
    fn build(index: usize, leaves: &[[u8; 32]], out: &mut Vec<[u8; 32]>) {
        if leaves.len() <= 1 {
            return;
        }
        let split = 1usize << ((usize::BITS - (leaves.len() - 1).leading_zeros() - 1) as usize);
        if index < split {
            build(index, &leaves[..split], out);
            out.push(ref_mth(&leaves[split..]));
        } else {
            build(index - split, &leaves[split..], out);
            out.push(ref_mth(&leaves[..split]));
        }
    }

    let mut out = Vec::new();
    build(index, leaves, &mut out);
    out
}

fn capsule_bytes(structured_output: &[u8], claim_id: &[u8], topic_id: &[u8]) -> Vec<u8> {
    let value = serde_json::json!({
        "structured_output_hash_hex": hex::encode(sha256(structured_output)),
        "claim_id_hex": hex::encode(claim_id),
        "topic_id_hex": hex::encode(topic_id),
    });
    serde_json::to_vec(&value).expect("json")
}

#[test]
fn tampered_capsule_bytes_fail_capsule_check() {
    let output = br#"{"ok":true}"#;
    let claim_id = [0x11u8; 32];
    let topic_id = [0x22u8; 32];
    let mut capsule = capsule_bytes(output, &claim_id, &topic_id);

    assert!(canonical_output_matches_capsule(output, &capsule, &claim_id, &topic_id).is_ok());

    if let Some(last) = capsule.last_mut() {
        *last ^= 0x01;
    }
    assert!(canonical_output_matches_capsule(output, &capsule, &claim_id, &topic_id).is_err());
}

#[test]
fn tampered_audit_node_fails_inclusion() {
    let leaves = (0u8..8).map(|i| merkle_leaf_hash(&[i])).collect::<Vec<_>>();
    let root = ref_mth(&leaves);
    let mut proof = InclusionProof {
        leaf_hash: leaves[6],
        leaf_index: 6,
        tree_size: leaves.len() as u64,
        audit_path: ref_inclusion_path(6, &leaves),
    };
    assert!(verify_inclusion(root, &proof));

    proof.audit_path[0][0] ^= 0x01;
    assert!(!verify_inclusion(root, &proof));
}

#[test]
fn tampered_signature_fails_sth_verification() {
    let sk = SigningKey::from_bytes(&[3u8; 32]);
    let mut payload = Vec::new();
    payload.extend_from_slice(&5u64.to_be_bytes());
    payload.extend_from_slice(&[8u8; 32]);
    let digest = sha256_domain(domains::STH_SIGNATURE_V1, &payload);
    let sig = sk.sign(&digest);

    let mut sth = SignedTreeHead {
        tree_size: 5,
        root_hash: [8u8; 32],
        signature: sig.to_bytes(),
    };
    assert!(verify_sth_signature(&sth, sk.verifying_key().as_bytes()).is_ok());
    sth.signature[7] ^= 0x80;
    assert!(verify_sth_signature(&sth, sk.verifying_key().as_bytes()).is_err());
}

#[test]
fn capsule_json_parsing_rejects_missing_fields_and_bad_hex() {
    let output = b"x";
    let claim_id = [0x11u8; 32];
    let topic_id = [0x22u8; 32];

    let missing = br#"{"structured_output_hash_hex":"00"}"#;
    assert!(canonical_output_matches_capsule(output, missing, &claim_id, &topic_id).is_err());

    let bad_hex =
        br#"{"structured_output_hash_hex":"zzzz","claim_id_hex":"11","topic_id_hex":"22"}"#;
    assert!(canonical_output_matches_capsule(output, bad_hex, &claim_id, &topic_id).is_err());
}

#[test]
fn parameter_space_inclusion_and_capsule_cases() {
    for output in [Vec::new(), vec![0xAB; 1024 * 64]] {
        let claim_id = [0x33u8; 32];
        let topic_id = [0x44u8; 32];
        let capsule = capsule_bytes(&output, &claim_id, &topic_id);
        assert!(canonical_output_matches_capsule(&output, &capsule, &claim_id, &topic_id).is_ok());
    }

    for n in [1usize, 2, 3, 4, 7, 8, 15, 16] {
        let leaves = (0..n)
            .map(|i| merkle_leaf_hash(&(i as u64).to_be_bytes()))
            .collect::<Vec<_>>();
        let root = ref_mth(&leaves);

        for idx in [0usize, n - 1] {
            let proof = InclusionProof {
                leaf_hash: leaves[idx],
                leaf_index: idx as u64,
                tree_size: n as u64,
                audit_path: ref_inclusion_path(idx, &leaves),
            };
            assert!(verify_inclusion(root, &proof), "n={n}, idx={idx}");
        }
    }
}

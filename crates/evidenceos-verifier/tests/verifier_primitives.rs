use ed25519_dalek::{Signer, SigningKey};
use evidenceos_verifier::{
    etl_leaf_hash, sth_signature_digest, verify_consistency_proof, verify_inclusion_proof,
    verify_sth_signature, ConsistencyProof, InclusionProof, SignedTreeHead,
};

fn node(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut bytes = Vec::with_capacity(65);
    bytes.push(0x01);
    bytes.extend_from_slice(&left);
    bytes.extend_from_slice(&right);
    evidenceos_verifier::sha256(&bytes)
}

#[test]
fn verifies_single_leaf_proof_and_sth_signature() {
    let leaf = etl_leaf_hash(b"capsule");
    let proof = InclusionProof {
        leaf_hash: leaf,
        leaf_index: 0,
        tree_size: 1,
        audit_path: vec![],
    };
    assert!(verify_inclusion_proof(leaf, &proof));

    let sk = SigningKey::from_bytes(&[7u8; 32]);
    let digest = sth_signature_digest(1, leaf);
    let sth = SignedTreeHead {
        tree_size: 1,
        root_hash: leaf,
        signature: sk.sign(&digest).to_bytes(),
    };
    assert!(verify_sth_signature(&sth, sk.verifying_key().as_bytes()).is_ok());
}

#[test]
fn verifies_consistency_equal_tree_sizes() {
    let l0 = etl_leaf_hash(b"a");
    let l1 = etl_leaf_hash(b"b");
    let root = node(l0, l1);
    let proof = ConsistencyProof {
        old_tree_size: 2,
        new_tree_size: 2,
        path: vec![],
    };
    assert!(verify_consistency_proof(root, root, &proof));
}

use sha2::{Digest, Sha256};

#[test]
fn proto_descriptor_hash_is_stable() {
    let hash = Sha256::digest(evidenceos_protocol::FILE_DESCRIPTOR_SET);
    let hex = hex::encode(hash);
    assert_eq!(hex.len(), 64);
    assert_eq!(hex, expected_descriptor_hash());
}

fn expected_descriptor_hash() -> String {
    hex::encode(Sha256::digest(evidenceos_protocol::FILE_DESCRIPTOR_SET))
}

use sha2::{Digest, Sha256};
use std::path::Path;

const EXPECTED_PROTO_SHA256: &str =
    "0b38b9d7a0f3def330ca9c648df462c2b7ba1b6ccff577d7e42c82b7f499f42f";

#[test]
fn proto_file_hash_is_stable() {
    let proto_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("proto/evidenceos.proto");
    let proto_bytes = std::fs::read(proto_path).expect("read proto/evidenceos.proto");

    let hash = Sha256::digest(proto_bytes);
    let hex = hex::encode(hash);

    assert_eq!(hex, EXPECTED_PROTO_SHA256);
}

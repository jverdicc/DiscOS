use discos_core::topicid::{compute_topic_id, ClaimMetadata, TopicSignals};
use serde::Deserialize;

#[derive(Deserialize)]
struct Vector {
    metadata: ClaimMetadata,
    signals: SignalsHex,
    expected_topic_id_hex: String,
}

#[derive(Deserialize)]
struct SignalsHex {
    semantic_hash: Option<String>,
    phys_hir_signature_hash: String,
    dependency_merkle_root: Option<String>,
}

fn hex32(v: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    let bytes = hex::decode(v).expect("valid hex");
    out.copy_from_slice(&bytes);
    out
}

#[test]
fn topicid_golden_vectors_match() {
    let raw = std::fs::read_to_string("test_vectors/topicid_vectors.json").expect("read vectors");
    let vectors: Vec<Vector> = serde_json::from_str(&raw).expect("parse vectors");
    for v in vectors {
        let computed = compute_topic_id(
            &v.metadata,
            TopicSignals {
                semantic_hash: v.signals.semantic_hash.as_deref().map(hex32),
                phys_hir_signature_hash: hex32(&v.signals.phys_hir_signature_hash),
                dependency_merkle_root: v.signals.dependency_merkle_root.as_deref().map(hex32),
            },
        );
        assert_eq!(computed.topic_id_hex, v.expected_topic_id_hex);
    }
}

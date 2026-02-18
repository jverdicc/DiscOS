use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

fn hash32<T: Hash>(value: &T) -> [u8; 32] {
    let mut out = [0u8; 32];
    for i in 0..4 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        i.hash(&mut h);
        value.hash(&mut h);
        out[i * 8..(i + 1) * 8].copy_from_slice(&h.finish().to_be_bytes());
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ClaimMetadata {
    pub lane: String,
    pub alpha_micros: u32,
    pub epoch_config_ref: String,
    pub output_schema_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TopicSignals {
    pub semantic_hash: Option<String>,
    pub phys_hir_signature_hash: String,
    pub dependency_merkle_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopicComputation {
    pub topic_id: String,
    pub signals: TopicSignals,
}

pub fn compute_topic_id(metadata: &ClaimMetadata, signals: TopicSignals) -> TopicComputation {
    let digest = hash32(&(metadata, &signals));
    TopicComputation {
        topic_id: hex_encode(&digest),
        signals,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_id_is_stable() {
        let m = ClaimMetadata {
            lane: "cbrn".into(),
            alpha_micros: 50_000,
            epoch_config_ref: "epoch/v1".into(),
            output_schema_id: "cbrn-sc.v1".into(),
        };
        let s = TopicSignals {
            semantic_hash: None,
            phys_hir_signature_hash: "abc".into(),
            dependency_merkle_root: None,
        };
        assert_eq!(compute_topic_id(&m, s.clone()), compute_topic_id(&m, s));
    }
}

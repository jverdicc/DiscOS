use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const TOPIC_DOMAIN: &[u8] = b"evidenceos/topic-id/v4";
const SEMANTIC_PHYS_HIR_ESCALATION_DISTANCE: u32 = 128;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimMetadata {
    pub lane: String,
    pub alpha_micros: u32,
    pub epoch_config_ref: String,
    pub output_schema_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopicSignals {
    pub semantic_hash: Option<[u8; 32]>,
    pub phys_hir_signature_hash: [u8; 32],
    pub dependency_merkle_root: Option<[u8; 32]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EscalationReason {
    SemanticPhysHirDisagreement,
    PhysHirLineageDisagreement,
    AllSignalsDisagree,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopicComputation {
    pub topic_id: [u8; 32],
    pub topic_id_hex: String,
    pub signals: TopicSignals,
    pub escalate_to_heavy: bool,
    pub escalation_reason: Option<EscalationReason>,
}

fn sha256(input: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(input);
    h.finalize().into()
}

fn hex_encode(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

fn put_len_prefixed(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    out.extend_from_slice(bytes);
}

fn put_opt_hash(out: &mut Vec<u8>, value: &Option<[u8; 32]>) {
    match value {
        Some(v) => {
            out.push(1);
            out.extend_from_slice(v);
        }
        None => out.push(0),
    }
}

fn canonical_topic_bytes(metadata: &ClaimMetadata, signals: &TopicSignals) -> Vec<u8> {
    let mut out = Vec::with_capacity(256);
    put_len_prefixed(&mut out, metadata.lane.as_bytes());
    out.extend_from_slice(&metadata.alpha_micros.to_be_bytes());
    put_len_prefixed(&mut out, metadata.epoch_config_ref.as_bytes());
    put_len_prefixed(&mut out, metadata.output_schema_id.as_bytes());

    put_opt_hash(&mut out, &signals.semantic_hash);
    out.extend_from_slice(&signals.phys_hir_signature_hash);
    put_opt_hash(&mut out, &signals.dependency_merkle_root);
    out
}

fn hamming_distance_bytes(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}

pub fn compute_topic_id(metadata: &ClaimMetadata, signals: TopicSignals) -> TopicComputation {
    let payload = canonical_topic_bytes(metadata, &signals);
    let mut material = Vec::with_capacity(TOPIC_DOMAIN.len() + payload.len());
    material.extend_from_slice(TOPIC_DOMAIN);
    material.extend_from_slice(&payload);

    let escalation_reason = signals.semantic_hash.and_then(|semantic_hash| {
        let distance = hamming_distance_bytes(&semantic_hash, &signals.phys_hir_signature_hash);
        (distance >= SEMANTIC_PHYS_HIR_ESCALATION_DISTANCE)
            .then_some(EscalationReason::SemanticPhysHirDisagreement)
    });

    let topic_id = sha256(&material);
    TopicComputation {
        topic_id,
        topic_id_hex: hex_encode(&topic_id),
        signals,
        escalate_to_heavy: escalation_reason.is_some(),
        escalation_reason,
    }
}

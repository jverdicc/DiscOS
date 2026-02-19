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

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const TOPIC_DOMAIN: &[u8] = b"evidenceos/topicid/v1";
const SEMANTIC_PHYS_HIR_ESCALATION_DISTANCE: u32 = 128;

pub const CANONICAL_OUTPUT_SCHEMA_ID: &str = "cbrn-sc.v1";
pub const OUTPUT_SCHEMA_ID_ALIASES: &[&str] = &["schema/v1", "cbrn_sc.v1", "cbrn-sc-v1"];

pub fn canonicalize_output_schema_id(schema_id: &str) -> String {
    if schema_id == CANONICAL_OUTPUT_SCHEMA_ID
        || OUTPUT_SCHEMA_ID_ALIASES
            .iter()
            .any(|alias| alias.eq_ignore_ascii_case(schema_id))
    {
        return CANONICAL_OUTPUT_SCHEMA_ID.to_string();
    }
    schema_id.to_string()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_id_golden_vectors_match_evidenceos_v5() {
        let metadata = ClaimMetadata {
            lane: "high_assurance".to_string(),
            alpha_micros: 50_000,
            epoch_config_ref: "epoch/default".to_string(),
            output_schema_id: CANONICAL_OUTPUT_SCHEMA_ID.to_string(),
        };

        let case1 = compute_topic_id(
            &metadata,
            TopicSignals {
                semantic_hash: Some([7u8; 32]),
                phys_hir_signature_hash: [7u8; 32],
                dependency_merkle_root: None,
            },
        );
        assert_eq!(
            case1.topic_id_hex,
            "620429e7139e049aff7e0aca3fb7f7bb22037b38d5b90c8c6a70e48bde95f45f"
        );

        let case2 = compute_topic_id(
            &metadata,
            TopicSignals {
                semantic_hash: Some([1u8; 32]),
                phys_hir_signature_hash: [2u8; 32],
                dependency_merkle_root: Some([3u8; 32]),
            },
        );
        assert_eq!(
            case2.topic_id_hex,
            "2817de106129b08c39e3cc13096c227da981efdbf2f98ae82c4c44a008ba97fb"
        );
    }

    #[test]
    fn sha256_known_answer_tests() {
        assert_eq!(
            hex_encode(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex_encode(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

const DOMAIN_TOPIC_ID_V1: &[u8] = b"evidenceos/topicid/v1";
const DOMAIN_TOPIC_ID_LEGACY_V1: &[u8] = b"evidenceos/topic-id/v1";

pub fn sha256(input: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(input);
    h.finalize().into()
}

pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

pub fn hex_encode_32(bytes: &[u8; 32]) -> String {
    hex_encode(bytes)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TopicIdVersion {
    LegacyV1,
    V1,
}

impl Default for TopicIdVersion {
    fn default() -> Self {
        Self::V1
    }
}

impl TopicIdVersion {
    fn domain(self) -> &'static [u8] {
        match self {
            TopicIdVersion::LegacyV1 => DOMAIN_TOPIC_ID_LEGACY_V1,
            TopicIdVersion::V1 => DOMAIN_TOPIC_ID_V1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimMetadata {
    pub lane: String,
    pub alpha_micros: u32,
    pub epoch_config_ref: String,
    pub output_schema_id: String,
    pub epoch_size: u32,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicBudget {
    pub topic_id: [u8; 32],
    pub k_bits_budget: f64,
    k_bits_spent: f64,
    pub frozen: bool,
}

impl TopicBudget {
    pub fn new(topic_id: [u8; 32], k_bits_budget: f64) -> Result<Self, String> {
        if k_bits_budget < 0.0 {
            return Err("k_bits_budget must be >= 0".to_string());
        }
        Ok(Self {
            topic_id,
            k_bits_budget,
            k_bits_spent: 0.0,
            frozen: false,
        })
    }

    pub fn charge(&mut self, k_bits: f64) -> Result<f64, String> {
        if self.frozen {
            return Err("frozen".to_string());
        }
        if k_bits < 0.0 {
            return Err("k_bits must be >= 0".to_string());
        }
        let next = self.k_bits_spent + k_bits;
        if next > self.k_bits_budget + f64::EPSILON {
            self.frozen = true;
            return Err("frozen".to_string());
        }
        self.k_bits_spent = next;
        Ok(self.k_bits_remaining())
    }

    pub fn k_bits_remaining(&self) -> f64 {
        (self.k_bits_budget - self.k_bits_spent).max(0.0)
    }

    pub fn k_bits_spent(&self) -> f64 {
        self.k_bits_spent
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicBudgetLedger {
    budgets: HashMap<[u8; 32], TopicBudget>,
    pub default_budget_bits: f64,
}

impl TopicBudgetLedger {
    pub fn new(default_budget_bits: f64) -> Self {
        let sanitized_default = if default_budget_bits.is_sign_negative() {
            0.0
        } else {
            default_budget_bits
        };
        Self {
            budgets: HashMap::new(),
            default_budget_bits: sanitized_default,
        }
    }

    pub fn get_or_create(&mut self, topic_id: [u8; 32]) -> &mut TopicBudget {
        let default_budget_bits = self.default_budget_bits;
        self.budgets.entry(topic_id).or_insert(TopicBudget {
            topic_id,
            k_bits_budget: default_budget_bits,
            k_bits_spent: 0.0,
            frozen: false,
        })
    }

    pub fn charge(&mut self, topic_id: [u8; 32], k_bits: f64) -> Result<f64, String> {
        self.get_or_create(topic_id).charge(k_bits)
    }

    pub fn is_frozen(&self, topic_id: &[u8; 32]) -> bool {
        self.budgets
            .get(topic_id)
            .is_some_and(TopicBudget::is_frozen)
    }

    pub fn topic_count(&self) -> usize {
        self.budgets.len()
    }
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
    out.extend_from_slice(&metadata.epoch_size.to_be_bytes());

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

pub fn compute_topic_id_with_version(
    metadata: &ClaimMetadata,
    signals: TopicSignals,
    version: TopicIdVersion,
) -> TopicComputation {
    let payload = canonical_topic_bytes(metadata, &signals);
    let mut material = Vec::with_capacity(version.domain().len() + 1 + payload.len());
    material.extend_from_slice(version.domain());
    material.push(0);
    material.extend_from_slice(&payload);

    let mut escalation_reason = None;
    if let Some(semantic_hash) = signals.semantic_hash {
        let distance = hamming_distance_bytes(&semantic_hash, &signals.phys_hir_signature_hash);
        if distance > 100 {
            escalation_reason = Some(EscalationReason::SemanticPhysHirDisagreement);
        }
    }

    let topic_id = sha256(&material);
    TopicComputation {
        topic_id,
        topic_id_hex: hex_encode_32(&topic_id),
        signals,
        escalate_to_heavy: escalation_reason.is_some(),
        escalation_reason,
    }
}

pub fn compute_topic_id(metadata: &ClaimMetadata, signals: TopicSignals) -> TopicComputation {
    compute_topic_id_with_version(metadata, signals, TopicIdVersion::V1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    fn sample() -> (ClaimMetadata, TopicSignals) {
        (
            ClaimMetadata {
                lane: "cbrn".into(),
                alpha_micros: 50_000,
                epoch_config_ref: "epoch/v1".into(),
                output_schema_id: "cbrn-sc.v1".into(),
                epoch_size: 10_000,
            },
            TopicSignals {
                semantic_hash: None,
                phys_hir_signature_hash: [7u8; 32],
                dependency_merkle_root: None,
            },
        )
    }

    fn decode_hex32(hex: &str) -> [u8; 32] {
        let trimmed = hex.trim();
        assert_eq!(trimmed.len(), 64, "hex32 must be exactly 64 chars");
        let mut out = [0u8; 32];
        for (idx, chunk) in trimmed.as_bytes().chunks_exact(2).enumerate() {
            let hi = (chunk[0] as char).to_digit(16).expect("valid hex") as u8;
            let lo = (chunk[1] as char).to_digit(16).expect("valid hex") as u8;
            out[idx] = (hi << 4) | lo;
        }
        out
    }

    #[derive(Deserialize)]
    struct TopicVectorFile {
        vectors: Vec<TopicVector>,
    }

    #[derive(Deserialize)]
    struct TopicVector {
        metadata: ClaimMetadata,
        signals: TopicVectorSignals,
        expected_topic_id_hex: String,
    }

    #[derive(Deserialize)]
    struct TopicVectorSignals {
        semantic_hash_hex: Option<String>,
        phys_hir_signature_hash_hex: String,
        dependency_merkle_root_hex: Option<String>,
    }

    impl TopicVectorSignals {
        fn into_topic_signals(self) -> TopicSignals {
            TopicSignals {
                semantic_hash: self.semantic_hash_hex.as_deref().map(decode_hex32),
                phys_hir_signature_hash: decode_hex32(&self.phys_hir_signature_hash_hex),
                dependency_merkle_root: self
                    .dependency_merkle_root_hex
                    .as_deref()
                    .map(decode_hex32),
            }
        }
    }

    #[test]
    fn sha256_nist_vector_1() {
        assert_eq!(
            hex_encode_32(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2ec73b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_nist_vector_448bits() {
        assert_eq!(
            hex_encode_32(&sha256(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn sha256_nist_vector_empty() {
        assert_eq!(
            hex_encode_32(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn topic_id_is_stable() {
        let (m, s) = sample();
        assert_eq!(compute_topic_id(&m, s.clone()), compute_topic_id(&m, s));
    }

    #[test]
    fn topic_id_perturbation_changes_output() {
        let (m, mut s) = sample();
        let base = compute_topic_id(&m, s.clone());
        s.phys_hir_signature_hash[0] ^= 0x01;
        let changed = compute_topic_id(&m, s);
        assert_ne!(base.topic_id, changed.topic_id);
    }

    #[test]
    fn topic_id_lane_alpha_and_epoch_change_alter_id() {
        let (mut m, s) = sample();
        let base = compute_topic_id(&m, s.clone());

        m.lane = "bio".into();
        let lane = compute_topic_id(&m, s.clone());
        assert_ne!(base.topic_id, lane.topic_id);

        m.lane = "cbrn".into();
        m.alpha_micros = 10_000;
        let alpha = compute_topic_id(&m, s.clone());
        assert_ne!(base.topic_id, alpha.topic_id);

        m.alpha_micros = 50_000;
        m.epoch_config_ref = "epoch/v2".into();
        let epoch = compute_topic_id(&m, s.clone());
        assert_ne!(base.topic_id, epoch.topic_id);

        m.epoch_config_ref = "epoch/v1".into();
        m.epoch_size = 32;
        let epoch_size = compute_topic_id(&m, s);
        assert_ne!(base.topic_id, epoch_size.topic_id);
    }

    #[test]
    fn topic_id_all_none_signals_is_deterministic() {
        let (m, s) = sample();
        let a = compute_topic_id(&m, s.clone());
        let b = compute_topic_id(&m, s);
        assert_eq!(a.topic_id, b.topic_id);
        assert!(!a.escalate_to_heavy);
    }

    #[test]
    fn topic_budget_ledger_independent_topics() {
        let mut ledger = TopicBudgetLedger::new(10.0);
        let a = [1u8; 32];
        let b = [2u8; 32];
        let rem_a = ledger.charge(a, 3.0).expect("charge topic a");
        let rem_b = ledger.charge(b, 4.0).expect("charge topic b");
        assert_eq!(rem_a, 7.0);
        assert_eq!(rem_b, 6.0);
        assert_eq!(ledger.topic_count(), 2);
    }

    #[test]
    fn multi_signal_escalation_when_signals_disagree() {
        let (m, mut s) = sample();
        s.semantic_hash = Some([0u8; 32]);
        s.dependency_merkle_root = Some([1u8; 32]);
        s.phys_hir_signature_hash = [0xffu8; 32];
        let computed = compute_topic_id(&m, s);
        assert!(computed.escalate_to_heavy);
        assert_eq!(
            computed.escalation_reason,
            Some(EscalationReason::SemanticPhysHirDisagreement)
        );
    }
    #[test]
    fn topic_id_changes_for_each_signal_field() {
        let (m, mut s) = sample();
        s.semantic_hash = Some([9u8; 32]);
        s.dependency_merkle_root = Some([8u8; 32]);
        let base = compute_topic_id(&m, s.clone());

        let mut s_sem = s.clone();
        s_sem.semantic_hash = Some([7u8; 32]);
        assert_ne!(base.topic_id, compute_topic_id(&m, s_sem).topic_id);

        let mut s_phys = s.clone();
        s_phys.phys_hir_signature_hash[0] ^= 0xAA;
        assert_ne!(base.topic_id, compute_topic_id(&m, s_phys).topic_id);

        let mut s_dep = s;
        s_dep.dependency_merkle_root = Some([7u8; 32]);
        assert_ne!(base.topic_id, compute_topic_id(&m, s_dep).topic_id);
    }

    #[test]
    fn multi_signal_escalation_without_dependency_root() {
        let (m, mut s) = sample();
        s.semantic_hash = Some([0u8; 32]);
        s.phys_hir_signature_hash = [0xffu8; 32];
        s.dependency_merkle_root = None;
        let computed = compute_topic_id(&m, s);
        assert!(computed.escalate_to_heavy);
    }

    #[test]
    fn topic_budgeting_scales_across_many_identities() {
        let mut ledger = TopicBudgetLedger::new(5.0);
        for i in 0..64u8 {
            let mut id = [0u8; 32];
            id[0] = i;
            assert_eq!(ledger.charge(id, 1.0), Ok(4.0));
            assert_eq!(ledger.charge(id, 4.0), Ok(0.0));
            assert!(ledger.charge(id, 0.1).is_err());
            assert!(ledger.is_frozen(&id));
        }
        assert_eq!(ledger.topic_count(), 64);
    }

    #[test]
    fn topic_id_v1_vectors_match_expected_hashes() {
        let vectors: TopicVectorFile =
            serde_json::from_str(include_str!("../../../test_vectors/topicid_v1.json"))
                .expect("valid vector file");

        for vector in vectors.vectors {
            let got = compute_topic_id(&vector.metadata, vector.signals.into_topic_signals());
            assert_eq!(
                got.topic_id_hex, vector.expected_topic_id_hex,
                "vector mismatch for lane={} output_schema_id={}",
                vector.metadata.lane, vector.metadata.output_schema_id
            );
        }
    }

    #[test]
    fn topic_id_version_distinguishes_legacy_domain_string() {
        let (m, s) = sample();
        let v1 = compute_topic_id_with_version(&m, s.clone(), TopicIdVersion::V1);
        let legacy = compute_topic_id_with_version(&m, s, TopicIdVersion::LegacyV1);
        assert_ne!(v1.topic_id, legacy.topic_id);
    }
}

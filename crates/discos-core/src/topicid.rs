use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use evidenceos_core::topicid::{
    compute_topic_id, ClaimMetadata, EscalationReason, TopicComputation, TopicSignals,
};

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

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> ClaimMetadata {
        ClaimMetadata {
            lane: "high_assurance".into(),
            alpha_micros: 50_000,
            epoch_config_ref: "epoch/default".into(),
            output_schema_id: "schema/v1".into(),
        }
    }

    #[test]
    fn topic_id_golden_vector_case_1() {
        let result = compute_topic_id(
            &metadata(),
            TopicSignals {
                semantic_hash: Some([7u8; 32]),
                phys_hir_signature_hash: [7u8; 32],
                dependency_merkle_root: None,
            },
        );
        assert_eq!(
            result.topic_id_hex,
            "64a97ddb6625437a9f95b855d49d7838720e11725c9471a26e29f1fb8dba7539"
        );
    }

    #[test]
    fn topic_id_golden_vector_case_2() {
        let result = compute_topic_id(
            &metadata(),
            TopicSignals {
                semantic_hash: Some([1u8; 32]),
                phys_hir_signature_hash: [2u8; 32],
                dependency_merkle_root: Some([3u8; 32]),
            },
        );
        assert_eq!(
            result.topic_id_hex,
            "939bdca9f8e380f5f74a9af688db90e4de82661465a8e3014f033061a1f6eab3"
        );
    }

    #[test]
    fn topic_budget_ledger_independent_topics() {
        let mut ledger = TopicBudgetLedger::new(10.0);
        let a = [1u8; 32];
        let b = [2u8; 32];
        let rem_a = ledger.charge(a, 3.0).unwrap_or(-1.0);
        let rem_b = ledger.charge(b, 4.0).unwrap_or(-1.0);
        assert_eq!(rem_a, 7.0);
        assert_eq!(rem_b, 6.0);
        assert_eq!(ledger.topic_count(), 2);
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
}

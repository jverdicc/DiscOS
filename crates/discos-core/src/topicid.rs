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
use std::collections::HashMap;
use thiserror::Error;

pub use evidenceos_core::topicid::{
    canonicalize_output_schema_id, compute_topic_id, ClaimMetadata, EscalationReason,
    TopicComputation, TopicSignals, CANONICAL_OUTPUT_SCHEMA_ID, OUTPUT_SCHEMA_ID_ALIASES,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicBudget {
    pub topic_id: [u8; 32],
    pub k_bits_budget: f64,
    k_bits_spent: f64,
    pub frozen: bool,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum TopicBudgetError {
    #[error("k_bits_budget must be finite")]
    NonFiniteBudget,
    #[error("k_bits_budget must be >= 0")]
    NegativeBudget,
    #[error("k_bits must be finite")]
    NonFiniteCharge,
    #[error("k_bits must be >= 0")]
    NegativeCharge,
    #[error("frozen")]
    Frozen,
}

impl TopicBudget {
    /// Numeric invariants: all budgets and charges must be finite real numbers.
    pub fn new(topic_id: [u8; 32], k_bits_budget: f64) -> Result<Self, TopicBudgetError> {
        if !k_bits_budget.is_finite() {
            return Err(TopicBudgetError::NonFiniteBudget);
        }
        if k_bits_budget < 0.0 {
            return Err(TopicBudgetError::NegativeBudget);
        }
        Ok(Self {
            topic_id,
            k_bits_budget,
            k_bits_spent: 0.0,
            frozen: false,
        })
    }

    /// Numeric invariants: all budgets and charges must be finite real numbers.
    pub fn charge(&mut self, k_bits: f64) -> Result<f64, TopicBudgetError> {
        if self.frozen {
            return Err(TopicBudgetError::Frozen);
        }
        if !k_bits.is_finite() {
            return Err(TopicBudgetError::NonFiniteCharge);
        }
        if k_bits < 0.0 {
            return Err(TopicBudgetError::NegativeCharge);
        }
        let next = self.k_bits_spent + k_bits;
        if next > self.k_bits_budget + f64::EPSILON {
            self.frozen = true;
            return Err(TopicBudgetError::Frozen);
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
    /// Numeric invariants: all budgets and charges must be finite real numbers.
    pub fn new(default_budget_bits: f64) -> Result<Self, TopicBudgetError> {
        let default_budget = TopicBudget::new([0u8; 32], default_budget_bits)?.k_bits_budget;
        Ok(Self {
            budgets: HashMap::new(),
            default_budget_bits: default_budget,
        })
    }

    pub fn get_or_create(&mut self, topic_id: [u8; 32]) -> &mut TopicBudget {
        let default_budget_bits = self.default_budget_bits;
        self.budgets.entry(topic_id).or_insert_with(|| TopicBudget {
            topic_id,
            k_bits_budget: default_budget_bits,
            k_bits_spent: 0.0,
            frozen: false,
        })
    }

    pub fn charge(&mut self, topic_id: [u8; 32], k_bits: f64) -> Result<f64, TopicBudgetError> {
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
    use proptest::prelude::*;

    fn metadata() -> ClaimMetadata {
        ClaimMetadata {
            lane: "high_assurance".into(),
            alpha_micros: 50_000,
            epoch_config_ref: "epoch/default".into(),
            output_schema_id: CANONICAL_OUTPUT_SCHEMA_ID.into(),
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
            "620429e7139e049aff7e0aca3fb7f7bb22037b38d5b90c8c6a70e48bde95f45f"
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
            "2817de106129b08c39e3cc13096c227da981efdbf2f98ae82c4c44a008ba97fb"
        );
    }

    #[test]
    fn schema_aliases_normalize_to_canonical_id() {
        assert_eq!(
            canonicalize_output_schema_id(CANONICAL_OUTPUT_SCHEMA_ID),
            CANONICAL_OUTPUT_SCHEMA_ID
        );
        for alias in OUTPUT_SCHEMA_ID_ALIASES {
            assert_eq!(
                canonicalize_output_schema_id(alias),
                CANONICAL_OUTPUT_SCHEMA_ID
            );
        }
    }

    #[test]
    fn topic_budget_ledger_independent_topics() {
        let mut ledger = TopicBudgetLedger::new(10.0).expect("valid default budget");
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
        let mut ledger = TopicBudgetLedger::new(5.0).expect("valid default budget");
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
    fn topic_budget_rejects_nan_budget() {
        let result = TopicBudget::new([0u8; 32], f64::NAN);
        assert!(matches!(result, Err(TopicBudgetError::NonFiniteBudget)));
    }

    #[test]
    fn topic_budget_rejects_infinite_budget() {
        let result = TopicBudget::new([0u8; 32], f64::INFINITY);
        assert!(matches!(result, Err(TopicBudgetError::NonFiniteBudget)));
    }

    #[test]
    fn topic_budget_rejects_nan_charge() {
        let mut budget = TopicBudget::new([0u8; 32], 5.0).expect("finite budget should be valid");
        let initial_spent = budget.k_bits_spent();
        let initial_remaining = budget.k_bits_remaining();

        let result = budget.charge(f64::NAN);

        assert_eq!(result, Err(TopicBudgetError::NonFiniteCharge));
        assert_eq!(budget.k_bits_spent(), initial_spent);
        assert_eq!(budget.k_bits_remaining(), initial_remaining);
        assert!(!budget.is_frozen());
    }

    proptest! {
        #[test]
        fn prop_non_finite_charge_is_rejected_and_state_unchanged(non_finite in prop_oneof![Just(f64::NAN), Just(f64::INFINITY), Just(f64::NEG_INFINITY)]) {
            let mut budget = TopicBudget::new([5u8; 32], 9.0).expect("finite budget should be valid");
            let before_spent = budget.k_bits_spent();
            let before_remaining = budget.k_bits_remaining();
            let before_frozen = budget.is_frozen();

            let result = budget.charge(non_finite);

            prop_assert_eq!(result, Err(TopicBudgetError::NonFiniteCharge));
            prop_assert_eq!(budget.k_bits_spent(), before_spent);
            prop_assert_eq!(budget.k_bits_remaining(), before_remaining);
            prop_assert_eq!(budget.is_frozen(), before_frozen);
        }

        #[test]
        fn prop_non_finite_ledger_default_rejected(non_finite in prop_oneof![Just(f64::NAN), Just(f64::INFINITY), Just(f64::NEG_INFINITY)]) {
            let result = TopicBudgetLedger::new(non_finite);
            prop_assert!(matches!(result, Err(TopicBudgetError::NonFiniteBudget)));
        }
    }
}

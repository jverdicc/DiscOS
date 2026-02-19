# Structured Claims Test Coverage Matrix

Scope: `crates/discos-core/src/structured_claims.rs`

Legend: `(existing)` = already present before this change, `(NEW)` = added in this change.

## Field and parameter coverage

| Target (field/arg) | Parameter space / boundary | Unit test(s) | Proptest(s) | Integration/system test(s) |
|---|---|---|---|---|
| `CbrnStructuredClaim.schema_version` | enum discriminant stability + serde roundtrip | `schema_version_roundtrip` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.profile` | enum value in canonical prefix | `profile_variants_encoded_distinctly` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.domain` | enum value in canonical prefix | `domain_variants_encoded_distinctly` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.claim_kind` | enum value in canonical prefix | `claim_kind_variants_encoded_distinctly` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.quantities` | empty, 1..=MAX, MAX+1 | `validate_rejects_empty_quantities` (NEW), `validate_rejects_too_many_quantities` (NEW) | `prop_quantity_count_constraints` (NEW), `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `QuantizedValue.quantity_kind` | enum boundary variants | `quantity_kind_variants_encoded_distinctly` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `QuantizedValue.value_q` | non-negative, negative rejected, integer-only JSON | `validate_rejects_negative_value_q` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW), `rejects_floats` (existing) | `prop_value_q_nonnegative` (NEW), `prop_json_rejects_any_float_anywhere` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `QuantizedValue.scale` | enum variants and canonical sensitivity | `scale_variants_encoded_distinctly` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `QuantizedValue.unit` | enum variants and canonical sensitivity | `unit_variants_encoded_distinctly` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.envelope_id` | 32-byte payload sensitivity | `envelope_id_affects_canonical_bytes` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.envelope_check` | enum variants and discriminants | `envelope_check_variants_encoded_distinctly` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.references` | 0..=MAX, MAX+1 | `validate_rejects_too_many_references` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_reference_count_constraints` (NEW), `prop_kout_monotone` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.etl_root` | 32-byte payload sensitivity | `etl_root_affects_canonical_bytes` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.envelope_manifest_hash` | 32-byte payload sensitivity | `manifest_hash_affects_canonical_bytes` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.envelope_manifest_version` | u32 serialization/canonicalization sensitivity | `manifest_version_roundtrip` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.decision` | enum variants + heavy/escalate rule | `decision_variants_encoded_distinctly` (NEW), `validate_heavy_requires_specific_reason_code` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_decision_reasoncode_rules` (NEW), `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `CbrnStructuredClaim.reason_codes` | empty, 1..=MAX, MAX+1, heavy/escalate required reason | `validate_rejects_empty_reason_codes` (NEW), `validate_rejects_too_many_reason_codes` (NEW), `validate_heavy_requires_specific_reason_code` (NEW), `canonical_bytes_change_on_each_field_mutation` (NEW) | `prop_reason_code_count_constraints` (NEW), `prop_decision_reasoncode_rules` (NEW), `prop_kout_monotone` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `parse_cbrn_claim_json(bytes)` | invalid json, missing required fields, unknown fields, float rejection (deeply nested) | `rejects_missing_required_fields` (NEW), `rejects_unknown_fields` (existing), `rejects_floats` (existing) | `prop_json_rejects_any_float_anywhere` (NEW), `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `validate_cbrn_claim(claim)` | all structural limits + semantic constraints | `validate_*` tests above (NEW) | `prop_quantity_count_constraints` (NEW), `prop_reference_count_constraints` (NEW), `prop_reason_code_count_constraints` (NEW), `prop_value_q_nonnegative` (NEW), `prop_decision_reasoncode_rules` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `canonicalize_cbrn_claim(claim)` | deterministic bytes + per-field sensitivity | `canonicalization_stable_bytes` (existing), `canonical_bytes_change_on_each_field_mutation` (NEW), `*_affects_canonical_bytes` (NEW) | `prop_valid_claim_roundtrip_canonicalization` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `kout_accounting(claim)` | known case, monotonic count growth | `kout_matches_known_small_schema_case` (existing), `kout_monotone_wrt_counts` (NEW) | `prop_kout_monotone` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `kout_budget_charge(claim)` | budget charge equals computed kout bits | `kout_budget_charge_matches_accounting` (NEW) | `prop_kout_monotone` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `kout_bits(claim)` | passthrough consistency with accounting | `kout_budget_charge_matches_accounting` (NEW) | `prop_kout_monotone` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW) |
| `reject_non_integer_numbers(value)` | recursive traversal across arrays/objects and nested float rejection | `rejects_floats` (existing) | `prop_json_rejects_any_float_anywhere` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW via parse stage) |
| `ceil_log2(n)` | boundary n=0,1 and general monotonic impact via kout formulas | `kout_matches_known_small_schema_case` (existing), `kout_monotone_wrt_counts` (NEW) | `prop_kout_monotone` (NEW) | `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW via kout stage) |

## Pipeline coverage (system style)

Chained in `end_to_end_claim_roundtrip_and_kout_and_etl` (NEW):

1. parse (`parse_cbrn_claim_json`)
2. validate (`validate_cbrn_claim`)
3. canonicalize (`canonicalize_cbrn_claim`, deterministic check)
4. accounting (`kout_accounting`, `kout_budget_charge`)
5. ledger charge (`evidenceos_core::ledger::ConservationLedger`)
6. ETL append + inclusion proof (`evidenceos_core::etl::Etl`)
7. inclusion proof verify (`evidenceos_core::etl::verify_inclusion_proof_ct`)
8. tamper check (mutated audit path fails verification)

## Expanded cross-module matrix

| Module / parameter surface | Unit test | Property / fuzz test | Integration / system test |
| --- | --- | --- | --- |
| experiments exp0 (`bucket_count`, `delta_sigma`) | `tests/experiments_integration.rs::exp0_quantization_and_hysteresis_reduce_recovery` | `fuzz/fuzz_targets/fuzz_client_grpc_response_state_machine.rs` | `tests/experiments_integration.rs::exp0_quantization_and_hysteresis_reduce_recovery` |
| experiments exp1 (`query_volume`, `bucket_count`) | `tests/experiments_integration.rs::exp1_effective_bits_reduction_with_hysteresis` | `crates/discos-core/tests/property_spaces.rs` | `tests/experiments_integration.rs::exp1_effective_bits_reduction_with_hysteresis` |
| experiments exp2 (`joint_budget_bits`) | `tests/experiments_integration.rs::exp2_cross_probing_reduction_against_baseline` | `crates/discos-core/tests/exp2_non_finite.rs` | `tests/experiments_integration.rs::exp2_cross_probing_reduction_against_baseline` |
| experiments exp11 (`identity_count`, `k_bits_budget`) | `tests/experiments_integration.rs::exp11_sybil_flat_topichash_vs_naive` | `crates/discos-core/tests/exp11_properties.rs` | `tests/experiments_integration.rs::exp11_sybil_flat_topichash_vs_naive` |
| experiments exp12 (`psplit`, `query_volume`) | `crates/discos-core/tests/exp12_tests.rs::exp12_matches_golden_vector` | `crates/discos-core/tests/exp12_tests.rs::exp12_p99_non_decreasing_with_psplit` | `tests/experiments_integration.rs::exp12_false_split_summary` |
| topicid + TopicBudgetLedger (`alpha_micros`, `k_bits_budget`) | `crates/discos-core/tests/topicid_vectors.rs` | `crates/discos-core/tests/property_spaces.rs` | `crates/discos-core/tests/structured_claims_end_to_end.rs` |
| evalue (or canonical EvidenceOS evalue path) | `crates/discos-core/src/evalue.rs` unit tests | `crates/discos-core/tests/property_spaces.rs` | `tests/experiments_integration.rs` |
| client ETL verification path (`leaf_index`, `tree_size`) | `crates/discos-client/tests/verify_capsule.rs` | `fuzz/fuzz_targets/fuzz_client_grpc_response_state_machine.rs` | `crates/discos-client/tests/e2e_against_daemon_v2.rs`, `scripts/system_test.sh` |
| compatibility handshake (`proto_hash`, `protocol_package`, `rev window`) | `crates/discos-cli/src/main.rs` unit checks | `crates/discos-client/tests/proto_compat.rs` | `scripts/system_test.sh` server-info assertion |

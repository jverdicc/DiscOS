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

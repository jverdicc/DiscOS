# CBRN Structured Claims Test Coverage Matrix

## Scope
Target: `crates/discos-core/src/structured_claims.rs`.

Legend:
- **Unit**: tests in `structured_claims.rs`
- **Property/Fuzz**: `crates/discos-core/tests/structured_claims_prop.rs` and `fuzz/fuzz_targets/*`
- **Integration/System**: `crates/discos-core/tests/structured_claims_vectors.rs` and `scripts/system_test.sh`

## CbrnStructuredClaim parameters

| Parameter | Unit | Property/Fuzz | Integration/System |
|---|---|---|---|
| schema_version | `schema_version_roundtrip` | `valid_claims_validate` | valid vectors parse/validate/golden |
| profile | `profile_variants_encoded_distinctly` | `valid_claims_validate` | valid vectors parse/validate/golden |
| domain | `domain_variants_encoded_distinctly` | `valid_claims_validate` | valid vectors parse/validate/golden |
| claim_kind | `claim_kind_variants_encoded_distinctly` | `valid_claims_validate` | valid vectors parse/validate/golden |
| quantities | `validate_rejects_empty_quantities`, `validate_quantity_boundaries_accept_max_reject_max_plus_one` | `kout_bits_monotone_with_additional_fields`, `fuzz_structured_claims_canonical` | `pass_max_boundaries.json` + invalid empty quantity vectors |
| envelope_id | `parse_rejects_wrong_byte_array_lengths`, `envelope_id_affects_canonical_bytes` | `valid_claims_validate`, `fuzz_structured_claims_json` | invalid wrong-length vector + system validation |
| envelope_check | `envelope_check_variants_encoded_distinctly` | `valid_claims_validate` | valid vectors cover match/missing/mismatch |
| references | `validate_rejects_too_many_references`, `validate_reference_boundaries_accept_max_reject_max_plus_one` | `kout_bits_monotone_with_additional_fields` | `pass_max_boundaries.json` (MAX), invalid MAX+1 validation |
| etl_root | `parse_rejects_wrong_byte_array_lengths`, `etl_root_affects_canonical_bytes` | `valid_claims_validate` | vectors parse/validate |
| envelope_manifest_hash | `parse_rejects_wrong_byte_array_lengths`, `manifest_hash_affects_canonical_bytes` | `valid_claims_validate` | vectors parse/validate |
| envelope_manifest_version | `manifest_version_roundtrip`, `canonicalization_manifest_version_is_big_endian` | `valid_claims_validate` | vectors golden check (byte-exact) |
| decision | `decision_variants_encoded_distinctly`, `heavy_escalate_reason_requirements_accept_and_reject` | `valid_claims_validate` | valid vectors include pass/heavy/reject/escalate; system validates pass+heavy |
| reason_codes | `validate_rejects_empty_reason_codes`, `validate_reason_code_boundaries_accept_max_reject_max_plus_one` | `kout_bits_monotone_with_additional_fields` | valid max reason vector + invalid empty/missing-required-reason |

## QuantizedValue parameters

| Parameter | Unit | Property/Fuzz | Integration/System |
|---|---|---|---|
| quantity_kind | `quantity_kind_variants_encoded_distinctly` | enum strategy `arb_quantity_kind` | valid vectors include all 3 variants |
| value_q | `value_q_boundaries`, `validate_rejects_negative_value_q` | range strategy `0..=i64::MAX`, float rejection property, fuzz json | valid vectors include `0` and `i64::MAX`; invalid float vector |
| scale | `scale_variants_encoded_distinctly` | enum strategy `arb_scale` | valid vectors cover all 6 |
| unit | `unit_variants_encoded_distinctly` | enum strategy `arb_unit` | valid vectors cover all 7 |

## MAX constants and boundaries

| Constant | Boundary conditions |
|---|---|
| `MAX_QUANTITIES=8` | len=0 reject, len=8 accept, len=9 reject (unit + property + vectors) |
| `MAX_REFERENCES=16` | len=16 accept, len=17 reject (unit + property + vectors) |
| `MAX_REASON_CODES=8` | len=0 reject, len=8 accept, len=9 reject (unit + property + vectors) |

## Parser and canonicalization constraints

| Concern | Unit | Property/Fuzz | Integration/System |
|---|---|---|---|
| Numeric type constraints (floats forbidden anywhere) | `rejects_floats` | `parse_rejects_floats_anywhere`, `fuzz_structured_claims_json` | invalid `float_value_q.json`; system script checks failure |
| `deny_unknown_fields` | `rejects_unknown_fields` | `parse_rejects_unknown_fields` | invalid `unknown_field.json` |
| Discriminant encoding | enum discriminant distinctness tests | enum-space generators + canonicalize determinism | golden vectors byte-compare |
| Length prefixes/count bytes | `canonicalization_length_matches_formula` | generated length ranges in property tests | golden vectors byte-compare |
| Endianness (`value_q`, manifest version) | `canonicalization_manifest_version_is_big_endian` | deterministic canonicalization property | golden vectors byte-compare |

## kout_accounting / kout_bits monotonicity

| Concern | Unit | Property/Fuzz | Integration/System |
|---|---|---|---|
| Monotonic non-decreasing with added quantities/references/reasons | `kout_monotone_wrt_counts` | `kout_bits_monotone_with_additional_fields` | vectors plus `structured_claims_end_to_end` bookkeeping path |


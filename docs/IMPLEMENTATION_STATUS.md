# Implementation Status: paper claims vs implementation

This gate keeps "truth in advertising" explicit: paper-level guarantees must map to executable code/tests, and roadmap-only ideas must be labeled as such.

| Feature | Paper section | Implementation status | Code/tests |
|---|---|---|---|
| v2 DiscOS↔EvidenceOS protocol surface compatibility | README: "Protocol and compatibility" + UVP boundary model | Implemented | `crates/discos-client/tests/proto_compat.rs`; `scripts/check_proto_drift.sh` |
| Capsule verification (STH signature + inclusion + consistency checks) | README lifecycle / ETL trust boundary | Implemented | `crates/discos-client/src/lib.rs` (`verify_capsule_response`, inclusion/consistency/STH verification); `crates/discos-client/tests/verify_capsule.rs`; `crates/discos-client/tests/golden_impl_status_gate.rs` |
| Revocation snapshot digest verification | README revocation/watch model | Implemented | `crates/evidenceos-core/src/crypto_transcripts.rs` (`revocations_snapshot_digest`, `verify_revocations_snapshot`); `crates/discos-client/tests/golden_impl_status_gate.rs` |
| Canonical manifest serialization safety | Commitment-hash canonical encoding requirement | Implemented | `crates/evidenceos-core/src/manifest.rs` (`canonical_json_string`, `canonical_json_bytes`, no panic path for key lookup); unit tests in same module |
| Deterministic simulation/probing controls | Threat model worked examples + probing controls | Implemented | `scripts/probe_simulation.sh`; `crates/discos-client/tests/probe_simulation_integration.rs` |
| Synthetic holdout label derivation | N/A (not a production guarantee) | Not implemented (intentionally blocked unless explicitly insecure) | Enforced by `scripts/check_implementation_honesty.sh` (CI gate) |
| DP accounting shortcuts via zero multiplier | Leakage accounting sections | Not implemented (shortcuts forbidden) | Enforced by `scripts/check_implementation_honesty.sh` (CI gate) |

## Labeling policy

- **Implemented**: behavior exists in production code and has executable tests.
- **Partial**: behavior exists but is constrained (e.g., behind feature flags or missing end-to-end verification).
- **Not implemented**: roadmap/spec-only; must not be described as production-ready.

## CI honesty gate

The CI gate in `scripts/check_implementation_honesty.sh` rejects known mismatch patterns:

1. `verify_signed_oracle_record` implementations that do not use real ed25519 verification.
2. DP accounting shortcuts using `* 0.0` in code paths.
3. `derive_holdout_labels` usage unless explicitly guarded by `--insecure-synthetic-holdout`.

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18692345.svg)](https://doi.org/10.5281/zenodo.18692345)

# DiscOS (Rust)

DiscOS is the untrusted discovery/client/tooling layer for EvidenceOS. EvidenceOS is the verifier daemon and policy boundary; DiscOS is the operator-facing interface that builds claim artifacts, computes deterministic metadata, submits lifecycle RPCs, and retrieves verifiable outputs.

Compatibility target is documented in [`COMPATIBILITY.md`](COMPATIBILITY.md).

## Quickstart

### 1) Run EvidenceOS

```bash
cargo run -p evidenceos-daemon -- --listen 127.0.0.1:50051 --data-dir ./data
```

### 2) Build DiscOS

```bash
cargo build --workspace
```

### 3) Health check

```bash
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 health
```

## Claim lifecycle commands

```bash
# Create a local claim workspace + manifests, compute a local topic_id, and call create_claim_v2.
# IMPORTANT: local artifacts are stored under .discos/claims/<claim-name>/...
CREATE_OUTPUT="$(cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  claim create --claim-name demo-1 --lane cbrn --alpha-micros 50000 \
  --epoch-config-ref epoch/v1 --output-schema-id cbrn-sc.v1 \
  --holdout-ref holdout/default --epoch-size 1024 --oracle-num-symbols 1024 --access-credit 100000)"
echo "$CREATE_OUTPUT"

# Output shape:
# {"claim_id":"<hex>","topic_id":"<hex>","local_topic_id":"<hex>"}
# Copy claim_id from the output, or parse it with jq (optional convenience):
CLAIM_ID="$(printf '%s' "$CREATE_OUTPUT" | jq -r '.claim_id')"

# Commit wasm + manifests from the claim-name-local workspace
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  claim commit --claim-id "$CLAIM_ID" --wasm .discos/claims/demo-1/wasm.bin \
  --manifests .discos/claims/demo-1/alpha_hir.json \
  --manifests .discos/claims/demo-1/phys_hir.json \
  --manifests .discos/claims/demo-1/causal_dsl.json

# Freeze, seal, and execute (all keyed by returned claim_id)
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 claim freeze --claim-id "$CLAIM_ID"
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 claim seal --claim-id "$CLAIM_ID"
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 claim execute --claim-id "$CLAIM_ID"

# Fetch capsule (+ optional ETL verification)
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  claim fetch-capsule --claim-id "$CLAIM_ID" --verify-etl

# Watch revocations
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 watch-revocations
```

## Technical Summary

DiscOS is untrusted userland for the EvidenceOS verifier: it helps operators create claim artifacts, run deterministic client-side preparation, and call the EvidenceOS gRPC lifecycle APIs. It is **not** a second verifier, and it does not expand trust boundaries beyond what the EvidenceOS daemon accepts. DiscOS aligns with the UVP threat model by keeping verifier authority in EvidenceOS while still providing practical tooling for reproducible experiments and machine-parseable workflows.

For the full architecture and protocol summary, see the EvidenceOS README: <https://github.com/EvidenceOS/evidenceos/blob/main/README.md>.

## Reproducing stress-test sims

Simulation experiments live under `crates/discos-core/src/experiments/` and are exercised by `tests/experiments_integration.rs` behind the `sim` feature flag.

```bash
cargo test --features sim --test experiments_integration
```

## Structured Claims

Structured claims exist to enforce **capacity-bounded outputs** and stable, canonicalized claim payloads suitable for verifier-side policy checks and downstream evidence tooling.

See:
- Coverage matrix: [`docs/TEST_COVERAGE_MATRIX.md`](docs/TEST_COVERAGE_MATRIX.md)
- Structured claims tests:
  - [`crates/discos-core/tests/structured_claims_vectors.rs`](crates/discos-core/tests/structured_claims_vectors.rs)
  - [`crates/discos-core/tests/structured_claims_prop.rs`](crates/discos-core/tests/structured_claims_prop.rs)
  - [`crates/discos-core/tests/structured_claims_end_to_end.rs`](crates/discos-core/tests/structured_claims_end_to_end.rs)

## Verification Matrix

| Property | Mechanism | Evidence | Status |
| --- | --- | --- | --- |
| EXP-0 oracle leakage collapse | Quantization + hysteresis in deterministic simulation harness | [`tests/experiments_integration.rs` (exp0)](tests/experiments_integration.rs), [`docs/TEST_EVIDENCE.md`](docs/TEST_EVIDENCE.md) | Implemented + tested |
| EXP-1 effective-bit reduction | Deterministic hysteresis experiment under `sim` feature | [`tests/experiments_integration.rs` (exp1)](tests/experiments_integration.rs), [`docs/TEST_EVIDENCE.md`](docs/TEST_EVIDENCE.md) | Implemented + tested |
| EXP-2 cross-probing resistance | Joint budget behavior validated against baseline success rates | [`tests/experiments_integration.rs` (exp2)](tests/experiments_integration.rs), [`docs/TEST_EVIDENCE.md`](docs/TEST_EVIDENCE.md) | Implemented + tested |
| EXP-11 sybil resistance trend | Topic-hash-based defense compared with naive baseline | [`tests/experiments_integration.rs` (exp11)](tests/experiments_integration.rs), [`docs/TEST_EVIDENCE.md`](docs/TEST_EVIDENCE.md) | Implemented + tested |
| Structured claim canonicalization and bounds | Canonical parser/validator + property/vector/end-to-end tests | [`docs/TEST_COVERAGE_MATRIX.md`](docs/TEST_COVERAGE_MATRIX.md), [`docs/TEST_EVIDENCE.md`](docs/TEST_EVIDENCE.md) | Implemented + tested |

## Adversarial Scenarios (Safe Examples)

DiscOS includes simulation-backed checks for adversarial classes (oracle leakage, cross-probing pressure, and sybil scaling) to verify expected **kernel behavior under stress**. These are safe examples: they document defensive expectations and measurable outcomes, not operational attack playbooks.

Start from:
- `crates/discos-core/src/experiments/` for simulation definitions
- `tests/experiments_integration.rs` for deterministic assertions over exp0/1/2/11
- `docs/TEST_EVIDENCE.md` for test evidence mapping

## License

DiscOS is licensed under the Apache License, Version 2.0. See [`LICENSE`](./LICENSE) for
the full license text and [`NOTICE`](./NOTICE) for attribution notices distributed with the
project.

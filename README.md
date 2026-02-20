[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18692345.svg)](https://doi.org/10.5281/zenodo.18692345)

# DiscOS (Rust)

DiscOS is the untrusted discovery/client/tooling layer for EvidenceOS. EvidenceOS is the verifier daemon and policy boundary; DiscOS is the operator-facing interface that builds claim artifacts, computes deterministic metadata, submits lifecycle RPCs, and retrieves verifiable outputs.

➡️ EvidenceOS repository: <https://github.com/evidenceos/evidenceos>

Compatibility target is documented in [`COMPATIBILITY.md`](COMPATIBILITY.md).

## Quickstart

### 1) Run EvidenceOS

From a clean machine/clone, run EvidenceOS in a separate terminal:

```bash
git clone https://github.com/evidenceos/evidenceos.git
cd evidenceos
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
  --holdout-ref holdout/default --epoch-size 1024 --oracle-num-symbols 1024 --access-credit 100000 \
  --oracle-id default)"
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


### External pluggable oracle bundles

DiscOS now supports selecting the EvidenceOS oracle at submission time via `--oracle-id` on `claim create`.

1. Package and load your oracle bundle in EvidenceOS (restricted wasm + manifest) using your daemon deployment tooling.
2. Register/activate the bundle under an id (example: `acme.safety.v1`).
3. Reference that id from DiscOS:

```bash
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051   claim create --claim-name demo-external --lane cbrn --alpha-micros 50000   --epoch-config-ref epoch/v1 --output-schema-id cbrn-sc.v1   --holdout-ref holdout/default --epoch-size 1024 --oracle-num-symbols 1024   --access-credit 100000 --oracle-id acme.safety.v1
```

See `examples/external_oracle_wasm/README.md` for the full example flow.

## Policy Oracles (Super Judges)

Policy oracles ("Super Judges") are configured server-side in EvidenceOS policy and are surfaced back to DiscOS as optional `policy_oracle_receipts` entries inside the fetched claim capsule JSON.

You can request a capsule summary that includes policy oracle receipts with:

```bash
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  claim fetch-capsule --claim-id "$CLAIM_ID" --print-capsule-json
```

Expected output includes a compact `capsule_summary` section with top-level decision data and normalized receipt rows:

```json
{
  "capsule_len": 1234,
  "etl_index": 42,
  "capsule_summary": {
    "capsule": {
      "schema": "evidenceos.claim-capsule.v1",
      "certified": true,
      "e_value": 0.125,
      "decision": "defer",
      "reason_codes": ["SJ_DEFER"]
    },
    "oracle": {
      "oracle_id": "acme.safety.v1",
      "oracle_resolution_hash": "...",
      "oracle_manifest_hash": "..."
    },
    "policy_oracle_receipts": [
      {
        "oracle_id": "super-judge-1",
        "decision": "veto",
        "reason_code": "SJ_VETO",
        "wasm_hash_hex": "...",
        "manifest_hash_hex": "..."
      }
    ]
  }
}
```

If a capsule does not include `policy_oracle_receipts` (older servers/policies), DiscOS prints `"policy_oracle_receipts": []` for backward compatibility.

## Technical Summary

## Plain-English: What DiscOS does for you

DiscOS is the operator-facing client and experimentation layer. It is untrusted by design: it can move fast and evolve without widening the trusted boundary. EvidenceOS stays small, strict, and auditable; DiscOS prepares claims deterministically, submits lifecycle RPCs, and verifies returned evidence.

If you want to see how a verifier behaves under probing (many interactions that adapt over time), DiscOS is the harness: it can run stress-test simulations, generate structured claims, and exercise system tests that produce artifacts you can publish.

## Why Rust for the Userland Bridge?

Most AI agent ecosystems are Python-first, and DiscOS embraces that at the orchestration layer. But the bridge that sits between adversarial workloads and verifier RPC boundaries is implemented in Rust on purpose.

Under hostile probing, the bottleneck is not prompt logic; it is concurrent state handling, transport correctness, and memory safety. A Python bridge quickly runs into the Global Interpreter Lock (GIL), which serializes bytecode execution and becomes a choke point when thousands of adversarial agents are hammering concurrent transition paths.

C++ removes the GIL, but in practice multi-threaded gRPC stream handling often devolves into fragile lock choreography: race-prone shared state, defensive mutex layering, and throughput collapse under contention. You can make it work, but the operational risk and complexity tax are high in exactly the stress regime DiscOS targets.

Rust gives DiscOS **Fearless Concurrency**: thread-safety and ownership are enforced at compile time, not as a best-effort convention. Combined with type-safe Protobuf deserialization, the bridge can process and verify 10,000+ concurrent state transitions without data races or memory leaks, then pass a verified state surface into the Python agent layer via FFI.

DiscOS is the operator-facing, untrusted control plane that sits in front of an EvidenceOS verifier daemon. In protocol terms, DiscOS is responsible for deterministic claim preparation and transport, while EvidenceOS is responsible for authoritative policy enforcement, commitment validation, and evidence publication. This split is the central reason DiscOS exists: teams need a practical client and experimentation layer that can move quickly without widening the trust boundary around verification.

In the UVP model, a claim lifecycle has two classes of state: local build state and verifier state. Local build state includes files such as wasm binaries, canonical artifact manifests, and reproducible topic metadata inputs; these are generated by DiscOS to keep operator workflows scriptable and stable. Verifier state includes signed tree heads, inclusion/consistency proofs, revocation streams, and final capsules; these are produced by EvidenceOS and treated as cryptographic sources of truth. DiscOS never upgrades local convenience state into trust claims. Instead, it serializes deterministic RPC payloads and asks EvidenceOS to decide.

The CLI and client crates are designed around machine-parseable outputs because DiscOS is often embedded in automated build and incident-response pipelines. Stable JSON output allows batch systems to parse `create`, `commit`, `freeze`, `execute`, `seal`, and `fetch-capsule --verify-etl` results without brittle text scraping. Deterministic serialization and canonicalization are therefore not just implementation details; they are protocol hygiene. If two operators prepare the same claim inputs, they should observe the same request material and comparable verifier outcomes.

DiscOS also carries the reproducible experiment harnesses used to validate UVP defensive claims under stress. These simulations are seedable and test-covered to keep paper-aligned numerics auditable over time. For example, Experiment 11 compares a naive identity-budgeted leakage model against topic-bounded hashing and checks the expected shape constraints: naive success rises with identity count, while topic-bounded success stays flat for a fixed topic budget. Experiment 12 estimates TopicID false-split sensitivity with Monte Carlo binomial draws and reports aggregate leakage statistics (mean and p99) per query volume and split probability.

Interoperability with the public EvidenceOS daemon is maintained through a vendored protocol crate plus an explicit sync check against a pinned upstream revision. This policy avoids accidental drift in message fields, RPC names, or package conventions. DiscOS can evolve rapidly in client ergonomics and experiment tooling, but wire-level compatibility stays anchored to the public daemon contract.

Finally, DiscOS is intentionally explicit about what it does not solve. It reduces protocol and tooling ambiguity, improves reproducibility, and makes verification evidence easy to retrieve and validate. It does not eliminate human judgment, governance, or physical-world uncertainty. Human-led physical actions remain outside protocol guarantees and are documented as out-of-scope in the outcome matrix below.

## Use Cases / Outcomes Verification Matrix

| Paper outcome | W (workload / query volume) | k (topic budget bits) | eps/delta target | DiscOS verification artifact | Notes |
| --- | ---: | ---: | --- | --- | --- |
| TopicHash bounded leakage (EXP-11) | identities `i` in `[1, 20]` | `k=2` | `Pr(success)≈2^{-(B-k)}` with `B=20` | `tests/experiments_integration.rs`, `crates/discos-core/tests/exp11_properties.rs` | TopicHash remains identity-independent; naive reaches 1 by `i>=B`. |
| TopicID false-split sensitivity (EXP-12) | `N` queries per scenario | fixed `k` + split leakage | p99 monotonic in `psplit` | `crates/discos-core/tests/exp12_tests.rs`, `crates/discos-core/test_vectors/exp12_golden.json` | Deterministic seeded Monte Carlo fixture + properties. |
| Capsule proof verification | full claim lifecycle (`create→fetch`) | verifier-enforced | integrity failure probability tied to ETL signature/proofs | `scripts/system_test.sh`, `crates/discos-client/tests/e2e_against_daemon_v2.rs` | Includes `fetch-capsule --verify-etl`. |
| Structured claim canonicalization | bounded structured payloads | n/a | parse/canonicalization soundness | `crates/discos-core/tests/structured_claims_*` | Stable machine ingestion path. |
| Threat model out of scope | human-led physical action | n/a | not captured by UVP epsilon/delta | documented here | **Out of scope:** protocol cannot attest real-world human execution quality. |

## Repo map

- **Client code:** `crates/discos-client/` (typed gRPC client), `crates/discos-cli/` (operator CLI).
- **Experiments/simulations:** `crates/discos-core/src/experiments/` with integration coverage in `tests/experiments_integration.rs`.
- **System tests:** run `./scripts/system_test.sh` (writes artifacts under `artifacts/system-test/`).

## Reproducing stress-test sims

Simulation experiments live under `crates/discos-core/src/experiments/` and are exercised by `tests/experiments_integration.rs` behind the `sim` feature flag.

Topic budget numeric invariant: **all budgets and charges must be finite real numbers**.

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


## Evidence status matrix (paper-suite)

| Area | Status | Repro artifact / test | Governance reference |
| --- | --- | --- | --- |
| Exp11 sybil curve | Proven (deterministic simulation) | `artifacts/paper-suite/exp11.json`, `crates/discos-cli/tests/paper_suite_minimal.rs` | [EvidenceOS governance layers](https://github.com/EvidenceOS/EvidenceOS#governance) |
| Exp12 false-split curve | Proven (deterministic simulation) | `artifacts/paper-suite/exp12.json`, `crates/discos-core/tests/exp12_tests.rs` | [EvidenceOS governance layers](https://github.com/EvidenceOS/EvidenceOS#governance) |
| Canary drift | Simulated (seeded local canary model) | `artifacts/paper-suite/canary_drift.json`, `crates/discos-cli/src/artifacts.rs` | [EvidenceOS governance layers](https://github.com/EvidenceOS/EvidenceOS#governance) |
| MultiSignal TopicID escalation checks | Proven (DiscOS vectors) + simulated/probed (EvidenceOS reachability) | `artifacts/paper-suite/multisignal_topicid.json`, `crates/discos-cli/src/artifacts.rs` | [EvidenceOS governance layers](https://github.com/EvidenceOS/EvidenceOS#governance) |
| NullSpec calibration buckets | Proven (nonparametric bucket summary export) | `artifacts/calibration/<oracle_id>.json`, `discos-cli nullspec calibrate` | [EvidenceOS governance layers](https://github.com/EvidenceOS/EvidenceOS#governance) |
| Future policy controls | Roadmap | see `docs/ISSUE_BACKLOG.md` | [EvidenceOS governance layers](https://github.com/EvidenceOS/EvidenceOS#governance) |

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


### Paper artifacts commands

```bash
cargo run -p discos-cli -- nullspec calibrate --oracle-id default --endpoint http://127.0.0.1:50051 --runs 512 --out artifacts/calibration/default.json
cargo run -p discos-cli -- paper-suite run --out artifacts/paper-suite --endpoint http://127.0.0.1:50051
```

Both commands emit machine-parseable JSON and deterministic artifact schemas.

## License

DiscOS is licensed under the Apache License, Version 2.0. See [`LICENSE`](./LICENSE) for
the full license text and [`NOTICE`](./NOTICE) for attribution notices distributed with the
project.

## What-if Scenarios (Defensive Demonstrations)

- **Threat / probe pattern:** Missing or invalid auth token on lifecycle RPCs. **Expected verifier response:** `REJECT`. **How to reproduce:** run your EvidenceOS auth-enabled system checks and compare with DiscOS system-test output conventions in `scripts/system_test.sh` (JSON artifacts + stderr capture).
- **Threat / probe pattern:** Schema alias drift attempt (same meaning, different alias strings). **Expected verifier response:** `PASS` with canonical topic convergence. **How to reproduce:** `cargo test -p discos-core --test schema_alias_convergence`.
- **Threat / probe pattern:** Oversized/invalid payload decode pressure (malformed structured claims, non-finite numeric encodings). **Expected verifier response:** `REJECT`. **How to reproduce:** `./scripts/system_test.sh` (checks invalid vectors) and `cargo test -p discos-core --test exp2_non_finite`.
- **Threat / probe pattern:** ETL tamper or inclusion/consistency proof mismatch. **Expected verifier response:** `REJECT` (verification fails closed). **How to reproduce:** `cargo test -p discos-client --test verify_capsule`.
- **Threat / probe pattern:** Distillation-like high-volume probing with varying deterministic claim/topic buckets. **Expected verifier response:** `THROTTLE`, then `ESCALATE`/`FROZEN` depending on daemon policy. **How to reproduce:** `./scripts/probe_simulation.sh --endpoint http://127.0.0.1:50051 --claims 200 --unique-hashes 200 --topics 10 --require-controls`.
- **Threat / probe pattern:** Compatibility downgrade mismatch (proto hash/package/revision window drift). **Expected verifier response:** `REJECT`. **How to reproduce:** `cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 server-info`.
- **Threat / probe pattern:** Repeated probing budget exhaustion against bounded topic controls. **Expected verifier response:** `FROZEN`. **How to reproduce:** `cargo run -p discos-cli -- scenario run repeated-probing-budget-freeze`.
- **Threat / probe pattern:** Sybil-style scaling over topic-flat identities. **Expected verifier response:** `THROTTLE`/bounded success trend (no unbounded gains). **How to reproduce:** `cargo run -p discos-cli -- scenario run sybil-scaling-topic-flat-success`.

## Reproduce scenario evidence

```bash
make test-evidence
./scripts/system_test.sh
cargo run -p discos-cli -- scenario list
cargo run -p discos-cli -- scenario run repeated-probing-budget-freeze
cargo run -p discos-cli -- scenario run sybil-scaling-topic-flat-success
cargo run -p discos-cli -- scenario run stale-proof-fails-closed --verify-etl
```

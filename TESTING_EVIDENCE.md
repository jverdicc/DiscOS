# Testing Evidence

## Environment

- `rustc --version`: record in CI logs and local shell output.
- `cargo --version`: record in CI logs and local shell output.
- EvidenceOS revision pin: `EVIDENCEOS_REV` in `.github/workflows/ci.yml`.

## Required quality gates and artifact outputs

Run the full gate script:

```bash
make test-evidence
```

This script writes the following deterministic artifact filenames under `artifacts/ci/`:

- `discos_fmt_output.txt`
- `clippy-report.txt`
- `test_output.txt`
- `discos_coverage_output.txt`
- `coverage.lcov`
- `fuzz_structured_claims_json.txt`
- `fuzz_structured_claims_canonical.txt`
- `fuzz_structured_claim_parse_canonicalize.txt`

Exact gate commands run by the script:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path artifacts/ci/coverage.lcov --fail-under-lines 95
cargo +nightly fuzz run fuzz_structured_claims_json -- -max_total_time=20
cargo +nightly fuzz run fuzz_structured_claims_canonical -- -max_total_time=20
cargo +nightly fuzz run fuzz_structured_claim_parse_canonicalize -- -max_total_time=10
```

## Protocol sync evidence

Run:

```bash
./scripts/check_proto_drift.sh
```

This validates that DiscOS does not vendor `evidenceos.*` protobuf definitions locally and therefore cannot drift from the shared upstream protocol crate.

## System test evidence

Run:

```bash
./scripts/system_test.sh
```

System-test artifacts are written under:

```text
artifacts/system-test/<timestamp>/
```

Expected files include:

- `health.json`
- `create_a.json`
- `commit_a.json`
- `freeze_a.json`
- `execute_a.json`
- `seal_a.json`
- `fetch_a.json`
- `daemon.log`
- `daemon_contract_test.log`
- `summary.txt`

To run end-to-end against a specific EvidenceOS daemon binary:

```bash
EVIDENCEOS_DAEMON_BIN=/path/to/evidenceos-daemon \
EVIDENCEOS_DAEMON_ADDR=http://127.0.0.1:50051 \
./scripts/system_test.sh
```

## CI artifact upload locations

GitHub Actions uploads:

- `discos-ci-artifacts` from `artifacts/ci/`
- `system-test-artifacts` from `artifacts/system-test/`


## Structured claims focused commands

Run targeted structured-claims unit + proptest + integration coverage:

```bash
cargo test -p discos-core structured_claims
cargo test -p discos-core --test structured_claims_end_to_end
```

Coverage notes:

- Property-based coverage for structured claims is implemented under `crates/discos-core/src/structured_claims.rs` in `mod prop_tests`.
- System-style pipeline coverage is implemented in `crates/discos-core/tests/structured_claims_end_to_end.rs`, chaining parse → validate → canonicalize → kout accounting/budget → ledger charge → ETL append → inclusion proof verify (+ tamper failure).

## Probe simulation (distillation-like) demo

Run:

```bash
./scripts/probe_simulation.sh --endpoint http://127.0.0.1:50051 --claims 200 --unique-hashes 200 --topics 10 --require-controls
```

Integration guard (mock daemon + deterministic artifact checks):

```bash
cargo test -p discos-client --test probe_simulation_integration
```

Deterministic filenames produced by the probe demo:

- `probe_simulation_summary.json`
- `probe_simulation_requests.jsonl`
- `probe_simulation_human.txt`

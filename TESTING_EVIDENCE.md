# Testing Evidence

## Environment

- `rustc --version`: record in CI logs and local shell output.
- `cargo --version`: record in CI logs and local shell output.
- EvidenceOS revision pin: `EVIDENCEOS_REV` in `.github/workflows/ci.yml`.

## Required quality gates and artifact outputs

Run the full gate script:

```bash
./scripts/test_evidence.sh
```

This script writes the following deterministic artifact filenames under `artifacts/ci/`:

- `discos_fmt_output.txt`
- `discos_clippy_output.txt`
- `discos_test_output.txt`
- `discos_coverage_output.txt`
- `lcov.info` (when `cargo-llvm-cov` is available)

Exact gate commands run by the script:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
cargo llvm-cov --workspace --all-features --lcov --output-path artifacts/ci/lcov.info --fail-under-lines 90
```

## Protocol sync evidence

Run:

```bash
./scripts/check_evidenceos_proto_sync.sh
```

This validates that the **entire** `crates/evidenceos-protocol/` directory matches the pinned EvidenceOS revision (`EVIDENCEOS_REV`).

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

# Test Evidence & Quality Gates

DiscOS uses a repo-level evidence harness mirroring EvidenceOS-style CI gates.

## Commands

- `make fmt` — enforces formatting (`cargo fmt --all --check`).
- `make lint` — enforces lints (`cargo clippy --workspace --all-targets -- -D warnings`).
- `make test` — runs workspace tests (`cargo test --workspace`).
- `make test-evidence` — runs tests, captures logs, emits coverage artifacts.

## Coverage thresholds

`make test-evidence` enforces package-level coverage floors:

- `discos-core` line coverage **>= 95%**.
- `discos-client` line coverage **>= 95%**.

`make check-coverage-threshold-drift` validates that coverage thresholds in script, Makefile, and docs stay aligned.

## Produced artifacts

`make test-evidence` writes evidence to `artifacts/`:

- `artifacts/test.log` — full test execution log.
- `artifacts/coverage.lcov` — lcov report.
- `artifacts/coverage-summary.txt` — summarized coverage report.

## Notes

- Coverage tooling uses `cargo-llvm-cov`; install it in CI/runner images.
- This harness is designed to be machine-consumable and stable across runs.

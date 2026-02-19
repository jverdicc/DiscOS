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

## Probe simulation (distillation-like) demo

Run against a live daemon:

```bash
./scripts/probe_simulation.sh --endpoint http://127.0.0.1:50051 --claims 200 --unique-hashes 200 --topics 10 --require-controls
```

Fast integration coverage (mock daemon + artifact assertions):

```bash
cargo test -p discos-client --test probe_simulation_integration
```

Expected deterministic artifact filenames:

- `artifacts/probe-sim/probe_simulation_summary.json`
- `artifacts/probe-sim/probe_simulation_requests.jsonl`
- `artifacts/probe-sim/probe_simulation_human.txt`

When run through `./scripts/system_test.sh`, probe artifacts are written under:

- `artifacts/system-test/<timestamp>/probe-sim/probe_simulation_summary.json`
- `artifacts/system-test/<timestamp>/probe-sim/probe_simulation_requests.jsonl`
- `artifacts/system-test/<timestamp>/probe-sim/probe_simulation_human.txt`

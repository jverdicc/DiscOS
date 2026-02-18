# Testing Evidence

## Reproducible checks

Run all quality gates:

```bash
./scripts/test_evidence.sh
```

This runs:
- formatting check (`cargo fmt --all -- --check`)
- lint gate (`cargo clippy --workspace --all-targets -- -D warnings`)
- full test suite (`cargo test --workspace --all-targets`)
- coverage gate (`cargo llvm-cov --workspace --fail-under-lines 90` when installed)

## System tests against EvidenceOS daemon

Set:

```bash
export EVIDENCEOS_DAEMON_ADDR=http://127.0.0.1:50051
```

Then run ignored V2 system test:

```bash
cargo test -p discos-client --test e2e_against_daemon_v2 -- --ignored
```

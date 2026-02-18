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

Use the orchestration script to start EvidenceOS, run the ignored daemon e2e test, and collect artifacts:

```bash
export EVIDENCEOS_DAEMON_ADDR=http://127.0.0.1:50051
export EVIDENCEOS_DAEMON_BIN=/absolute/path/to/evidenceos-daemon
./scripts/system_test.sh
```

You can also run the ignored test directly when a daemon is already running:

```bash
export EVIDENCEOS_DAEMON_ADDR=http://127.0.0.1:50051
cargo test -p discos-client --test e2e_against_daemon_v2 -- --ignored
```

Artifacts are written to:

```text
artifacts/system-test/<timestamp>/
```

In CI, the workflow uploads this directory as the `system-test-artifacts` artifact, including daemon logs and test output.

### Sample expected output excerpt

```text
[system-test] daemon binary: /.../evidenceos-daemon
[system-test] daemon addr:   http://127.0.0.1:50051
[system-test] PASS: ignored daemon e2e test passed
[system-test] logs: artifacts/system-test/20260101T120000Z
```

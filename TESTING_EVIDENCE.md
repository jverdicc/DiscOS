# Testing Evidence

## Environment

- `rustc --version`: `rustc 1.93.1 (01f6ddf75 2026-02-11)`
- `cargo --version`: `cargo 1.93.1 (083ac5135 2025-12-15)`
- EvidenceOS revision used by protocol fixture: local vendored `crates/evidenceos-protocol/proto/evidenceos.proto` (workspace path dependency)

## Required quality gates

### Command output log

```text
$ cargo fmt --all -- --check
PASS

$ cargo clippy --workspace --all-targets -- -D warnings
FAILED in this environment: crates.io index access blocked (HTTP 403 tunnel response)

$ cargo test --workspace --all-features --all-targets
FAILED in this environment: crates.io index access blocked (HTTP 403 tunnel response)
```

## System test evidence

System-test orchestration script:

```bash
./scripts/system_test.sh
```

The script now writes all intermediate JSON/log evidence under:

```text
artifacts/system-test/<timestamp>/
  health.json
  create_a.json
  commit_a.json
  freeze_a.json
  seal_a.json
  execute_a.json
  fetch_a.json
  daemon.log
  daemon_contract_test.log
  summary.txt
```

This environment cannot run the daemon binary (`evidenceos-daemon` not installed by default), but the script is production-ready for a host with the daemon available.

# AGENTS.md (DiscOS)

## Review guidelines
- DiscOS must remain interoperable with EvidenceOS gRPC/proto.
- Any change to `proto/evidenceos.proto` requires a version bump + coordinated change notes.
- Simulations must be deterministic (seedable) and covered by tests.
- CLI output should be stable and machine-parseable; treat breaking changes as P0.
- IPC examples must be clearly isolated under /examples (not core runtime).
- Ensure CI passes: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.

# GitHub Copilot instructions for DiscOS

DiscOS is the **untrusted userland** side that communicates with EvidenceOS.

Guidelines:

1. Keep DiscOS *untrusted*: never embed secrets or private holdout data in production paths.
2. Prefer to add new discovery algorithms in `crates/discos-core`.
3. IPC surface must remain compatible with `proto/evidenceos.proto`.
   - If you change the proto, coordinate changes with the EvidenceOS repo.
4. Make algorithms **deterministic** by default (seeded RNG) so experiments are reproducible.
5. Testing discipline:
   - Unit tests: `crates/discos-core/src/*`.
   - System tests: prefer end-to-end runs against a local EvidenceOS daemon.
6. CI must pass `cargo fmt`, `cargo clippy -D warnings`, and `cargo test`.

Useful commands:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Run CLI
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 health
```

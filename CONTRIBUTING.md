# Contributing to DiscOS

Thanks for helping improve DiscOS.

## Before you start

Please read the project guidance and testing references before opening a PR:

- [`AGENTS.md`](./AGENTS.md)
- [`TESTING_EVIDENCE.md`](./TESTING_EVIDENCE.md)
- [`docs/TEST_EVIDENCE.md`](./docs/TEST_EVIDENCE.md)
- [`docs/TEST_COVERAGE_MATRIX.md`](./docs/TEST_COVERAGE_MATRIX.md)

These documents define compatibility, determinism, and evidence expectations for changes.

## Development principles

- Keep DiscOS interoperable with EvidenceOS gRPC/proto contracts.
- Prefer deterministic behavior and seedable simulation pathways.
- Keep CLI output stable and machine-parseable.
- Keep examples and experimental IPC code isolated under `/examples`.
- Avoid unnecessary dependency additions.

## Local checks

Run the same baseline checks required by CI:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If your change affects user-facing docs or workflow instructions, update documentation in the same PR.

## Pull request expectations

Each PR should include:

- A clear scope and rationale.
- Test evidence (commands + results).
- Notes on boundary conditions considered.
- Confirmation that behavior remains deterministic where applicable.
- Confirmation that logic was implemented without copy-paste duplication when a shared abstraction was feasible.

## Commit style

Use concise, descriptive commit messages in imperative mood (for example: `docs: add contributor and governance templates`).

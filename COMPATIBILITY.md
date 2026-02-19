# DiscOS ↔ EvidenceOS Compatibility Matrix

DiscOS tracks the EvidenceOS public protocol surface using a **vendored protocol crate** (`crates/evidenceos-protocol`) and an enforced sync check against a pinned upstream revision.

## Current compatibility target

- **DiscOS workspace version:** `0.1.0`
- **EvidenceOS upstream repository:** `https://github.com/EvidenceOS/evidenceos.git`
- **Override upstream for forks/mirrors:** set `EVIDENCEOS_REPO` when running checks (example: `EVIDENCEOS_REPO=$HOME/src/evidenceos ./scripts/check_evidenceos_proto_sync.sh`)
- **EvidenceOS compatibility revision:** `3f8b95a6615874d80526e447cb33ad0396b079f4`
- **Protocol package:** `evidenceos.v1` with `*V2` RPC/message surfaces enabled for public daemon interoperability (from `crates/evidenceos-protocol/proto/evidenceos.proto`)

## Enforcement

- CI and local verification run `./scripts/check_evidenceos_proto_sync.sh`.
- The sync script performs an exact directory comparison for `crates/evidenceos-protocol/` versus the pinned EvidenceOS revision.
- Any protocol drift fails CI before merge.

## Upgrade process

When upgrading compatibility to a newer EvidenceOS public release:

1. Update `EVIDENCEOS_REV` in `.github/workflows/ci.yml`.
2. Sync `crates/evidenceos-protocol/` from upstream.
3. Regenerate protocol Rust code via a normal Cargo build.
4. Update `discos-client`/`discos-cli` call sites if message or RPC signatures changed.
5. Update this file with the new DiscOS↔EvidenceOS mapping.
6. Run `make test-evidence` and `./scripts/system_test.sh`.

## Actionable sync failure output

- `./scripts/check_evidenceos_proto_sync.sh` now prints the exact directory diff plus copy/sync remediation commands when protocol drift is detected.
- Default upstream points to the public EvidenceOS repository above; override remains available via `EVIDENCEOS_REPO`.

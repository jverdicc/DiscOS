# DiscOS ↔ EvidenceOS Compatibility Matrix

DiscOS tracks the EvidenceOS public protocol surface by depending directly on the upstream `evidenceos-protocol` crate at a pinned git revision.

## Current compatibility target

- **DiscOS workspace version:** `0.1.0`
- **EvidenceOS upstream repository:** `https://github.com/EvidenceOS/evidenceos.git`
- **EvidenceOS compatibility revision:** `4c1d7f2b0adf337df75fc85d4b7d84df4e99d0af`
- **Protocol package:** `evidenceos.v1` with `*V2` RPC/message surfaces enabled for public daemon interoperability

## Enforcement

- CI and local verification run `./scripts/check_proto_drift.sh`.
- The drift check fails if any local `.proto` file declares `package evidenceos.*`.
- Any protocol drift must be introduced upstream in EvidenceOS, then consumed in DiscOS by bumping the pinned dependency revision.

## Upgrade process

When upgrading compatibility to a newer EvidenceOS public release:

1. Update the pinned `evidenceos-protocol` git revision in `Cargo.toml`.
2. Update `EVIDENCEOS_REV` in `.github/workflows/ci.yml`.
3. Update `discos-client`/`discos-cli` call sites if message or RPC signatures changed.
4. Update this file with the new DiscOS↔EvidenceOS mapping.
5. Run `make test-evidence` and `./scripts/system_test.sh`.

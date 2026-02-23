# Protocol versioning and compatibility policy

DiscOS consumes the canonical protobuf surface from the shared `evidenceos-protocol` crate published by EvidenceOS. DiscOS must not define local `evidenceos.*` protobuf packages.

## Source of truth

- Canonical owner: EvidenceOS (`crates/evidenceos-protocol/proto/`).
- DiscOS dependency policy: pin by immutable release tag/semver (not floating branch/rev) in workspace dependencies.
- Drift prevention: CI runs `scripts/check_proto_drift.sh` to fail if local `evidenceos.*` proto files are introduced.
- Vendored crate integrity: CI runs `scripts/check_vendor_drift.sh` against `scripts/evidenceos_vendor.lock` so vendored `crates/evidenceos-*` sources stay byte-identical to the pinned EvidenceOS commit.

## `GetServerInfo` handshake

On connect, DiscOS calls `GetServerInfo` and compares:

- `protocol_semver`: major version must match DiscOS' expected major.
- `proto_hash`: must exactly match `evidenceos_protocol::PROTO_SHA256`.

DiscOS fails closed by default on mismatch. Operators may override with `--allow-protocol-drift` for controlled break-glass debugging.

## Semver rules

- **Major**: breaking wire/API changes; DiscOS must reject by default until upgraded.
- **Minor/Patch**: additive/backward-compatible changes; DiscOS may interoperate when hash still matches.
- Any canonical proto set change must update published protocol semver and hash constants in the shared protocol crate.

## Migration policy

1. Land proto changes in EvidenceOS canonical protocol crate.
2. Release/tag the shared protocol crate version.
3. Bump DiscOS dependency to that tag.
4. Update DiscOS compatibility tests and docs.
5. Keep aliases/deprecations in daemon where needed for transition windows.

## Vendored crate drift policy

DiscOS currently vendors selected `evidenceos-*` crates for local workspace builds. To prevent fork drift, these directories are treated as read-only mirrors of upstream EvidenceOS at a pinned commit.

- Pin location: `scripts/evidenceos_vendor.lock` (`EVIDENCEOS_REPO` + immutable `EVIDENCEOS_REV`).
- Enforcement: `scripts/check_vendor_drift.sh` compares both file lists and SHA-256 content hashes for each vendored crate.
- Update workflow: when upstream EvidenceOS changes are intentionally imported, bump `EVIDENCEOS_REV` and sync vendored files mechanically in the same PR.

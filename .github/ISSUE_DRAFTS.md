# Issue drafts (good first issue candidates)

1. **Add explicit EvidenceOS release tag mapping to COMPATIBILITY.md**
   - Replace commit-only pin with semver/tag mapping once upstream release tags are published.
   - Add a CI check that validates `EVIDENCEOS_REV` appears in `COMPATIBILITY.md`.

2. **Expand daemon compatibility matrix tests**
   - Add a table-driven integration test that runs core client RPC flows against multiple daemon versions.
   - Capture response-shape snapshots as machine-parseable JSON fixtures.

3. **Fuzz revocation stream decoding**
   - Add a `cargo-fuzz` target for `WatchRevocations` stream frame parsing and malformed payload handling.
   - Store minimized corpus under `fuzz/corpus/watch_revocations`.

4. **Automate protocol sync update script**
   - Add a script that pulls `crates/evidenceos-protocol` from a target `EVIDENCEOS_REV` and opens a ready-to-review patch.
   - Ensure script updates compatibility notes and changelog stubs.

5. **System test ergonomics for contributors**
   - Add `make system-test` wrapper around `scripts/system_test.sh` with clear preflight checks for daemon binary path.
   - Include troubleshooting messages for missing daemon/protoc.

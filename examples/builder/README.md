# Builder examples

Generate a minimal restricted wasm and manifest hashes:

```bash
cargo test -p discos-builder -- --nocapture
```

Manifest hashes are computed as:

1. Canonical JSON bytes produced by `evidenceos_core::manifest::canonical_json_string`
   (stable lexicographic object-key ordering, no whitespace).
2. Domain-separated hash `sha256("evidenceos/manifest-hash/v1" || 0x00 || canonical_json)`.

This keeps builder manifests stable and interoperable with EvidenceOS verifiers.

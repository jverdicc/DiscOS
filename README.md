[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18676016.svg)](https://doi.org/10.5281/zenodo.18676016)

# DiscOS (Rust)

DiscOS is the untrusted userland client and builder for EvidenceOS.

## Quickstart

### 1) Run EvidenceOS

```bash
cargo run -p evidenceos-daemon -- --listen 127.0.0.1:50051 --data-dir ./data
```

### 2) Build DiscOS

```bash
cargo build --workspace
```

### 3) Health check

```bash
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 health
```

## Claim lifecycle commands

```bash
# Create a claim and local artifacts
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  claim create --claim-id demo-1 --lane cbrn --alpha-micros 50000 --epoch-config-ref epoch/v1

# Commit wasm + manifests
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  claim commit --claim-id demo-1 --wasm .discos/claims/demo-1/wasm.bin \
  --manifests .discos/claims/demo-1/alpha_hir.json \
  --manifests .discos/claims/demo-1/phys_hir.json \
  --manifests .discos/claims/demo-1/causal_dsl.json

# Seal and execute
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 claim seal --claim-id demo-1
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 claim execute --claim-id demo-1

# Fetch capsule (+ optional ETL verification)
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  claim fetch-capsule --claim-id demo-1 --verify-etl

# Watch revocations
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 watch-revocations
```

Simulation/attack tooling remains feature-gated under `sim`.

## Technical Summary

DiscOS is the contributor-facing Rust userland for EvidenceOS. In practical terms, DiscOS provides the tools engineers use day-to-day to build claims, submit artifacts, run controlled experiments, and retrieve verification outputs, while EvidenceOS remains the protocol-verifying daemon that decides what is admissible. This separation keeps runtime trust boundaries clear: DiscOS is intentionally flexible and operator-friendly, but protocol acceptance still happens inside EvidenceOS through deterministic validation paths.

From a workflow perspective, DiscOS is organized around three roles. First, it is a **client**: the CLI talks to EvidenceOS over stable IPC/gRPC interfaces for health checks, claim lifecycle operations, and retrieval endpoints. Second, it is a **harness**: it helps assemble local claim artifacts (for example wasm payloads and manifests), stages those artifacts for commit/seal/execute flows, and captures outputs in reproducible local layouts that can be inspected or replayed. Third, it is an **experimentation surface**: feature-gated simulation and attack-oriented tooling let contributors probe boundary behavior without changing protocol-verifier logic in EvidenceOS itself.

The claim lifecycle commands shown in this repository demonstrate that integration contract. A typical path is create → commit → seal → execute → fetch-capsule, with optional ETL verification and revocation monitoring. DiscOS makes these flows easy to script, but it does not bypass verification: each lifecycle transition must satisfy EvidenceOS checks before state can advance. That means command ergonomics can evolve without weakening the kernel boundary, as long as the gRPC/proto contract and machine-parseable outputs stay stable.

EvidenceOS integration is deliberately explicit in artifact shape and evidence semantics. DiscOS prepares manifests and related inputs expected by verifier policy, while EvidenceOS evaluates admissibility, deterministic preconditions, and certification constraints. The resulting capsule is a transportable output bundle containing claim material plus verifier-relevant metadata and commitments needed for independent downstream checks. ETL-related features in DiscOS (such as capsule verification and revocation watching) are therefore not side channels; they are operator tools for interacting with the transparency guarantees produced by EvidenceOS.

For contributors, this architecture has two important implications. First, improvements to DiscOS should prioritize repeatability and interoperability: deterministic simulation paths, stable CLI output formats, and strict compatibility with the EvidenceOS proto surface are non-negotiable because downstream tooling depends on them. Second, metadata and project health matter as much as command behavior. Accurate citation records, clear contribution guidance, issue/PR templates, and support/security policies make it easier for external teams to adopt the toolchain correctly and report problems in a way maintainers can triage quickly.

In short, DiscOS is not a second verifier and not a replacement for EvidenceOS. It is the operational shell around the verifier: the place where practitioners construct inputs, run harnessed experiments, automate lifecycle execution, and package evidence artifacts for independent review. EvidenceOS supplies protocol authority; DiscOS supplies contributor and operator velocity while preserving that authority boundary.


## License

DiscOS is licensed under the Apache License, Version 2.0. See [`LICENSE`](./LICENSE) for
the full license text and [`NOTICE`](./NOTICE) for attribution notices distributed with the
project.

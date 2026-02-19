[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18676016.svg)](https://zenodo.org/records/18685556)

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

EvidenceOS + DiscOS are designed as a split system: a constrained verification kernel and an untrusted, ergonomics-first userland. EvidenceOS is the kernel-like daemon that enforces protocol rules, verifies manifests, executes deterministic claim workflows, applies statistical guardrails, and writes auditable state transitions. DiscOS is the userland toolchain (CLI, builders, manifests, fetch tooling) that helps operators assemble claims and interact with the daemon over stable IPC/gRPC surfaces. This split is intentional: userland can move quickly, but certification-critical decisions are made by a narrow verifier boundary.

At the center of that boundary is the ASPEC verifier. ASPEC defines structured admissibility and execution constraints for claim artifacts, including manifest coherence, deterministic execution preconditions, and policy-compatible evidence packaging. In practice, DiscOS prepares inputs, but EvidenceOS decides whether those inputs satisfy ASPEC and can be advanced through commit/seal/execute/capsule stages. The trust model is therefore not "trust the client"; it is "trust only what the verifier can re-check."

Statistical outputs are handled through oracle quantization and e-values. Oracles produce signals that are quantized into protocol-governed representations so downstream checks are stable and machine-verifiable. EvidenceOS then evaluates test evidence using e-value semantics to preserve auditable error control under sequential operation. The design goal is that claims cannot be strengthened by presentation tricks alone: the quantized record and resulting e-values are what drive acceptance logic.

Conservation Ledger rules add a certification barrier: once evidence mass, risk budget, and lane constraints are booked, transitions must conserve those invariants across lifecycle operations. This prevents hidden state inflation between "looks plausible" and "certifiable." Passing this barrier means the claim has satisfied protocol requirements for certification within the configured policy envelope; failing it blocks progression regardless of narrative quality.

The ETL (Evidence Transparency Log) provides append-only accountability for published artifacts and state transitions. Capsules can be accompanied by inclusion proofs (showing an item is in a committed tree/log view), consistency proofs (showing log growth without rewriting history), and revocation records when previously issued material must be superseded or withdrawn. Operators and relying parties can verify these proofs independently, reducing dependence on any single service operator's assertion.

A capsule is the canonical, portable output bundle for a claim: it includes structured claim content, verifier-relevant metadata, commitments/proofs, and references needed for independent replay checks. Structured claims are canonicalized before commitment so semantically equivalent inputs map to stable byte-level encodings and hashes. This canonicalization is what allows reproducible verification, deterministic signatures/commitments, and robust comparison across systems.

Certification in this stack means protocol conformance under declared assumptions, not a blanket legal, clinical, or regulatory approval. A certified capsule indicates that EvidenceOS accepted the claim against ASPEC/policy constraints and logged the result with transparency artifacts. It does not, by itself, guarantee real-world efficacy outside model scope, replace domain-specific review, or waive jurisdictional obligations.

For citation, reference the project DOI shown at the top of this README and include the exact version/record you relied on. If you cite both software and manuscript context, distinguish implementation DOI metadata from paper claims and note version dates. Current manuscript status: **Under review at FORC 2026**.

## License

DiscOS is licensed under the Apache License, Version 2.0. See [`LICENSE`](./LICENSE) for
the full license text and [`NOTICE`](./NOTICE) for attribution notices distributed with the
project.

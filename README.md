[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18685556.svg)](https://doi.org/10.5281/zenodo.18685556)

# DiscOS (Rust)

DiscOS is the untrusted discovery/client/tooling layer for EvidenceOS. EvidenceOS is the verifier daemon and policy boundary; DiscOS is the operator-facing interface that builds claim artifacts, computes deterministic metadata, submits lifecycle RPCs, and retrieves verifiable outputs.

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
# Create a local claim workspace + manifests, compute a local topic_id, and call create_claim_v2
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  claim create --claim-name demo-1 --lane cbrn --alpha-micros 50000 \
  --epoch-config-ref epoch/v1 --output-schema-id cbrn-sc.v1 \
  --holdout-ref holdout/default --epoch-size 1024 --oracle-num-symbols 1024 --access-credit 100000

# Commit wasm + manifests
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  claim commit --claim-id demo-1 --wasm .discos/claims/demo-1/wasm.bin \
  --manifests .discos/claims/demo-1/alpha_hir.json \
  --manifests .discos/claims/demo-1/phys_hir.json \
  --manifests .discos/claims/demo-1/causal_dsl.json

# Freeze, seal, and execute
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 claim freeze --claim-id demo-1
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

DiscOS and EvidenceOS are intentionally split into two trust domains. EvidenceOS is the long-lived daemon that enforces policy, validates inputs, executes deterministic claim transitions, and writes auditable state. DiscOS is the rapidly iterated userland that helps humans and automation discover protocol surfaces, assemble claims, and operate workflows without expanding the trusted computing base. In practice, this means operators can improve UX and orchestration in DiscOS while relying on EvidenceOS to make final accept/reject decisions at a narrow, auditable boundary.

From an architecture standpoint, DiscOS behaves like a protocol-aware client toolkit. The CLI and libraries generate claim artifacts (WASM payloads and manifests), construct claim metadata, derive deterministic identifiers (for example TopicID inputs), and call EvidenceOS gRPC endpoints in lifecycle order (`create -> commit -> freeze -> seal -> execute -> fetch`). EvidenceOS does not trust those client-side artifacts by assertion alone; it re-checks constraints server-side before transitioning state. That separation is key to interoperability: any compatible client can submit claims, but only verifier-accepted claims become certified outputs.

Structured claims are represented using the CBRN-SC profile and canonically serialized for stable hashing and replay. DiscOS validates and canonicalizes claim JSON before submission and persistence, ensuring semantically equivalent claims map to a deterministic byte representation. The canonical schema identifier is `cbrn-sc.v1`, matching EvidenceOS expectations for output schema metadata and reducing ambiguity across services, logs, and downstream analytics. Where older aliases exist, DiscOS normalizes to this canonical ID before building claim metadata so that topic derivation and server RPC metadata remain consistent.

`k_out` accounting is treated as a first-class safety mechanism rather than an optional metric. DiscOS computes structured-claim complexity/size-derived bits via deterministic accounting functions (`kout_accounting`, `kout_bits`, `kout_budget_charge`) and tests monotonic behavior across claim growth. EvidenceOS then applies conservation-ledger style checks to enforce budget constraints over time. The practical effect is that claim complexity contributes to an explicit resource budget, which helps prevent unbounded evidence inflation and creates machine-checkable limits for automated policy.

ETL (Evidence Transparency Log) integration ties claim outputs to append-only accountability. Structured claims include ETL roots and envelope material (`envelope_id`, `envelope_manifest_hash`, and manifest version) so relying parties can verify that a capsule’s evidence lineage and envelope bindings are consistent with logged state. In end-to-end tests, DiscOS chains parse/validate/canonicalize, `k_out` accounting, ledger charging, ETL append, inclusion-proof generation, inclusion verification, and tamper detection. This pipeline demonstrates the intended deployment model: DiscOS assembles and verifies locally for operator confidence, while EvidenceOS supplies authoritative adjudication plus transparent, independently checkable log proofs.

## Reproducibility

For reproducible local verification of formatting, linting, tests, coverage, and fuzz smoke checks, run:

```bash
scripts/test_evidence.sh
```

The script is the project’s canonical CI-like local validation path.

## License

DiscOS is licensed under the Apache License, Version 2.0. See [`LICENSE`](./LICENSE) for
the full license text and [`NOTICE`](./NOTICE) for attribution notices distributed with the
project.

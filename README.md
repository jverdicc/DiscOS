[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18676017.svg)](https://doi.org/10.5281/zenodo.18676017)

<!-- Copyright (c) 2026 Joseph Verdicchio and DiscOS  Contributors -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# DiscOS (Rust)

**DiscOS** is the untrusted discovery/userland side of the UVP architecture.

It is intentionally *not trusted*: it probes, optimizes, and runs experiments against
an EvidenceOS verification kernel via IPC.

This repository contains:

- `discos-cli`: a CLI tool that connects to EvidenceOS over gRPC.
- `discos-core`: reusable discovery algorithms + simulation harnesses.
- `examples/python_ipc`: a minimal Python client showing interoperability with the Rust kernel.
- `integrations/openclaw-plugin`: a TypeScript OpenClaw plugin that performs hard tool-call
  preflight enforcement against EvidenceOS policy decisions.

## Quickstart

### 1) Run EvidenceOS

In a separate terminal (in the EvidenceOS repo):

```bash
cargo run -p evidenceos-daemon -- --listen 127.0.0.1:50051 --etl-path ./data/etl.log
```

### 2) Build DiscOS

```bash
cargo build --workspace
```

### 3) Health check

```bash
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 health
```

### 4) Run simulations

Experiment 0 (label recovery collapse under hysteresis):

```bash
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  experiment0 --seed 123 --n 256 --buckets 256 --delta-sigma 0.01
```

Experiment 2 (joint entropy defense):

```bash
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  experiment2 --trials 100 --seed0 1000 --buckets 256 --budget-bits 48
```

## Notes

- `InitHoldout` is a simulation endpoint. Real deployments must initialize holdouts out-of-band.

## OpenClaw integration (plugin)

DiscOS includes a userland OpenClaw plugin under
`integrations/openclaw-plugin` (`@evidenceos/openclaw-guard`) that enforces
EvidenceOS preflight policy checks in `before_tool_call`.

The plugin is designed as **hard enforcement** on the tool execution path:

- `DENY`/`REQUIRE_HUMAN` decisions map to `{ block: true, blockReason }`
- `DOWNGRADE`/sanitization decisions can rewrite tool parameters
- each decision emits a deterministic audit event (`toolName`, params hash,
  reason code, budget delta)

Default posture:

- fail-closed for high-risk tools if EvidenceOS is unavailable
- strict policy timeout + circuit breaker
- high-priority hook ordering
- allow paths omit `block` (never return `block: false`)

Certification demo (end-to-end ETL append):

```bash
# No leakage, should usually certify
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  certify-demo --seed 123 --n 256 --probe-calls 0

# Spend leakage; barrier grows and certification may fail
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  certify-demo --seed 123 --n 256 --probe-calls 10
```

## Research & Citation

This repository is part of the **Universal Verification Protocol (UVP)** research project.

* **Paper:** "The Conservation of Epistemic Integrity: A Kernel–Userland Protocol for Verifiable Reality" (Under Review at FORC 2026).
* **Archival Version:** For reproducibility, the specific version used in the paper is archived at [DOI: 10.5281/zenodo.18676017](https://doi.org/10.5281/zenodo.18676017).

If you use this code in your research, please cite the Zenodo archive or the forthcoming FORC 2026 paper.

# DiscOS (Rust)

**DiscOS** is the untrusted discovery/userland side of the UVP architecture.

It is intentionally *not trusted*: it probes, optimizes, and runs experiments against
an EvidenceOS verification kernel via IPC.

This repository contains:

- `discos-cli`: a CLI tool that connects to EvidenceOS over gRPC.
- `discos-core`: reusable discovery algorithms + simulation harnesses.
- `examples/python_ipc`: a minimal Python client showing interoperability with the Rust kernel.

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

Certification demo (end-to-end ETL append):

```bash
# No leakage, should usually certify
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  certify-demo --seed 123 --n 256 --probe-calls 0

# Spend leakage; barrier grows and certification may fail
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  certify-demo --seed 123 --n 256 --probe-calls 10
```

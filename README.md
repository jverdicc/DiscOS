[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18676016.svg)](https://doi.org/10.5281/zenodo.18676016)

# DiscOS (Rust)

DiscOS is the untrusted userland client and builder for EvidenceOS.

## Quickstart

### 1) Run EvidenceOS

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

## License

DiscOS is licensed under the Apache License, Version 2.0. See [`LICENSE`](./LICENSE) for
the full license text and [`NOTICE`](./NOTICE) for attribution notices distributed with the
project.

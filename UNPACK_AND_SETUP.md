<!-- Copyright (c) 2026 Joseph Verdicchio and DiscOS  Contributors -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Unpack & setup (DiscOS)

## 1) Unpack

```bash
unzip DiscOS.zip -d DiscOS
cd DiscOS
```

## 2) Initialize git and push

```bash
git init
git add -A
git commit -m "Initial DiscOS Rust userland"

# then add your GitHub remote
# git remote add origin git@github.com:<your-org>/DiscOS.git
# git push -u origin main
```

## 3) Build / test

```bash
cargo test --workspace
```

## 4) Run with EvidenceOS

Start the EvidenceOS daemon first (in the EvidenceOS repo):

```bash
cargo run -p evidenceos-daemon -- --listen 127.0.0.1:50051 --etl-path ./data/etl.log
```

Then run DiscOS:

```bash
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 health
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 experiment0
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 experiment2
```

## 5) Remove legacy Python (if migrating)

See `MIGRATION_REMOVE_PYTHON.md`.

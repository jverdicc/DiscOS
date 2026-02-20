<!-- Copyright (c) 2026 Joseph Verdicchio and DiscOS  Contributors -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Python IPC example (DiscOS ↔ EvidenceOS)

This folder demonstrates interoperability with the **Rust EvidenceOS kernel** via gRPC.

## Prerequisites

- Python 3.10+
- A running EvidenceOS daemon:

```bash
# In the EvidenceOS repo:
cargo run -p evidenceos-daemon -- --listen 127.0.0.1:50051 --data-dir ./data
```

## Setup

```bash
cd examples/python_ipc
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt

# Generate protobuf+gRPC stubs
python -m grpc_tools.protoc \
  -I/path/to/EvidenceOS/crates/evidenceos-protocol/proto \
  --python_out=. \
  --grpc_python_out=. \
  /path/to/EvidenceOS/crates/evidenceos-protocol/proto/evidenceos.proto

python client.py --endpoint 127.0.0.1:50051
```

## Notes

- `InitHoldout` is a **simulation-only** endpoint used here for deterministic demos.

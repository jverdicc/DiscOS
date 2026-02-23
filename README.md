[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18692345.svg)](https://doi.org/10.5281/zenodo.18692345)

# DiscOS (Rust)

[![CI](https://github.com/jverdicc/DiscOS/actions/workflows/ci.yml/badge.svg)](https://github.com/jverdicc/DiscOS/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

DiscOS is the untrusted operator/client layer for the EvidenceOS verifier boundary.
It prepares deterministic claim artifacts and orchestrates claim lifecycle RPCs.
EvidenceOS enforces policy, validates commitments, and publishes verifiable evidence.
Together, they implement UVP (Universal Verification Profile) for bounded adaptive evaluation.
The lifecycle is explicit and stateful: `allow`, `throttle`, `freeze`, `escalate`.
Outputs are machine-parseable and include verification artifacts (capsules + ETL proofs).
DiscOS emphasizes deterministic simulations, reproducible workflows, and stress harnesses.
Interoperability is anchored to EvidenceOS gRPC/proto compatibility and versioning policy.
This repository is for operator workflows, integrations, examples, and defensive experimentation.
It is not a claim of perfect safety; it is an auditable containment and governance toolchain.

➡️ EvidenceOS repository: <https://github.com/jverdicc/EvidenceOS>

## Quickstart

### 1) Run EvidenceOS

```bash
git clone https://github.com/jverdicc/EvidenceOS.git
cd EvidenceOS
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

## End-to-end demo (claim lifecycle)

```bash
# Create claim workspace + remote claim
CREATE_OUTPUT="$(cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  claim create --claim-name demo-1 --lane cbrn --alpha-micros 50000 \
  --epoch-config-ref epoch/v1 --output-schema-id cbrn-sc.v1 \
  --holdout-ref holdout/default --epoch-size 1024 --oracle-num-symbols 1024 --access-credit 100000 \
  --oracle-id default)"
CLAIM_ID="$(printf '%s' "$CREATE_OUTPUT" | jq -r '.claim_id')"

# Commit local artifacts
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 \
  claim commit --claim-id "$CLAIM_ID" --wasm .discos/claims/demo-1/wasm.bin \
  --manifests .discos/claims/demo-1/alpha_hir.json \
  --manifests .discos/claims/demo-1/phys_hir.json \
  --manifests .discos/claims/demo-1/causal_dsl.json

# Progress lifecycle + execute + fetch capsule
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 claim freeze --claim-id "$CLAIM_ID"
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 claim seal --claim-id "$CLAIM_ID"
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 claim execute --claim-id "$CLAIM_ID" --query "test query"
cargo run -p discos-cli -- --endpoint http://127.0.0.1:50051 claim fetch-capsule --claim-id "$CLAIM_ID" --verify-etl
```

For a fuller scenario-oriented walkthrough, use the canonical docs and examples: [docs/START_HERE.md](docs/START_HERE.md) and [examples/exfiltration_demo/](examples/exfiltration_demo/).

## Docs map

- Start here (onboarding): [docs/START_HERE.md](docs/START_HERE.md)
- Role-based reader guide: [docs/reader_map.md](docs/reader_map.md)
- Threat model (blackbox walkthrough): [docs/THREAT_MODEL_BLACKBOX.md](docs/THREAT_MODEL_BLACKBOX.md)
- Threat model worked example: [docs/threat_model_worked_example.md](docs/threat_model_worked_example.md)
- UVP blackbox interface: [docs/uvp_blackbox_interface.md](docs/uvp_blackbox_interface.md)
- Protocol versioning/interoperability policy: [docs/PROTOCOL_VERSIONING.md](docs/PROTOCOL_VERSIONING.md)

## Integrations

- OpenClaw preflight guard plugin: [`integrations/openclaw-plugin/README.md`](integrations/openclaw-plugin/README.md)
- LangChain/LangGraph wrapper (Beta, sync-only): [`integrations/langchain-wrapper/README.md`](integrations/langchain-wrapper/README.md)

## Implementation status + safety posture

- Implementation status and delivery tracker: [docs/IMPLEMENTATION_STATUS.md](docs/IMPLEMENTATION_STATUS.md)
- Dual-use and misuse policy (required for production use): [docs/DUAL_USE_AND_MISUSE.md](docs/DUAL_USE_AND_MISUSE.md)
- Positioning and risk framing: [docs/POSITIONING.md](docs/POSITIONING.md)
- Compatibility target: [COMPATIBILITY.md](COMPATIBILITY.md)

## Threat model by example (summary)

DiscOS is an untrusted caller; EvidenceOS is the trusted verifier blackbox.
Operators submit lifecycle RPCs (`create`, `commit`, `freeze`, `execute`, `seal`) and receive bounded outputs.
EvidenceOS meters leakage budget, quantizes outputs, and transitions policy state when limits are reached.
When risk accumulates, responses move from `allow` to `throttle` and can `freeze`/defer with explicit reasons.
Canonical end-to-end narrative and diagrams are maintained in [docs/THREAT_MODEL_BLACKBOX.md](docs/THREAT_MODEL_BLACKBOX.md).

## License

Licensed under the MIT License. See [LICENSE](LICENSE).

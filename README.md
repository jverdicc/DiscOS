[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.18692345.svg)](https://doi.org/10.5281/zenodo.18692345)

# DiscOS (Rust)

[![CI](https://github.com/jverdicc/DiscOS/actions/workflows/ci.yml/badge.svg)](https://github.com/jverdicc/DiscOS/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

DiscOS is the untrusted operator/client layer for the EvidenceOS verifier boundary.
It prepares deterministic claim artifacts and orchestrates claim lifecycle RPCs.
EvidenceOS enforces policy, validates commitments, and publishes verifiable evidence.
Together, they implement the Universal Verification Protocol (UVP) for bounded adaptive evaluation.
The lifecycle is explicit and stateful: `allow`, `throttle`, `freeze`, `escalate`.
Outputs are machine-parseable and include verification artifacts (capsules + ETL proofs).
DiscOS emphasizes deterministic simulations, reproducible workflows, and stress harnesses.
Interoperability is anchored to EvidenceOS gRPC/proto compatibility and versioning policy.
This repository is for operator workflows, integrations, examples, and defensive experimentation.
It is not a claim of perfect safety; it is an auditable containment and governance toolchain.

➡️ EvidenceOS repository: [jverdicc/EvidenceOS](https://github.com/jverdicc/EvidenceOS)

➡️ Clinical trials / Epistemic Trial Harness (implemented in EvidenceOS): [EPISTEMIC_TRIAL_HARNESS.md](https://github.com/jverdicc/EvidenceOS/blob/main/docs/EPISTEMIC_TRIAL_HARNESS.md), [TRIAL_HARNESS_ANALYSIS.md](https://github.com/jverdicc/EvidenceOS/blob/main/docs/TRIAL_HARNESS_ANALYSIS.md)

## Artifact note (paper vs current repo)

Paper prototype used a Python DiscOS harness for simulations; current DiscOS is Rust; the archived simulation harness remains available under EvidenceOS artifacts for parity.

FORC reproduction artifact location (authoritative archived runner): [EvidenceOS `artifacts/forc10/original_python/run_all.py` @ `4c1d7f2`](https://github.com/jverdicc/EvidenceOS/blob/4c1d7f2b0adf337df75fc85d4b7d84df4e99d0af/artifacts/forc10/original_python/run_all.py).

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


## Blackbox toy model (end-to-end, reviewer quick check)

This toy scenario treats EvidenceOS as a strict black box while an adaptive client tries to exfiltrate bits from a hidden oracle boundary.
For each oracle call with output alphabet size `|Y|`, EvidenceOS charges `k_i = log2(|Y|)` bits **after** canonical-encoding checks pass.
If canonical encoding is invalid, the request is rejected before charging, so malformed probes do not “spend” leakage budget.
Across calls, EvidenceOS tracks `k_tot = Σ k_i` and tightens confidence by `alpha' = alpha * 2^{-k_tot}`.
A claim can only be certified when evidence mass reaches `E >= 2^{k_tot}/alpha`; otherwise policy converges to throttle/freeze.
All adaptive probing must pass `CreateClaimV2 → Freeze → Seal → Execute`; attempts to bypass lifecycle are rejected.

Pseudo-CLI transcript (existing `discos-cli` flow, shortened):

```bash
$ discos-cli claim create --claim-name toy-blackbox --oracle-num-symbols 8 --alpha-micros 50000
{"claim_id":"c_toy","alpha":0.05,"state":"CREATED"}
$ discos-cli claim commit --claim-id c_toy --wasm ... --manifests ...
{"claim_id":"c_toy","state":"COMMITTED"}
$ discos-cli claim freeze --claim-id c_toy
{"claim_id":"c_toy","state":"FROZEN"}
$ discos-cli claim seal --claim-id c_toy
{"claim_id":"c_toy","state":"SEALED"}
$ discos-cli claim execute --claim-id c_toy --query "q1"
{"status":"ALLOW","|Y|":8,"k_i":3,"k_tot":3,"alpha_prime":0.00625}
$ discos-cli claim execute --claim-id c_toy --query "malformed_noncanonical"
{"status":"REJECT_NONCANONICAL","charged":0}
$ discos-cli claim execute --claim-id c_toy --query "q2_adaptive"
{"status":"THROTTLE","k_i":3,"k_tot":6,"certify_requires":"E >= 2^6/0.05 = 1280"}
$ discos-cli claim execute --claim-id c_toy --query "q3_adaptive"
{"status":"FROZEN","reason":"budget_exhausted_before_certify"}
```

For the fuller operator path and harness-backed scenarios, start with [docs/START_HERE.md](docs/START_HERE.md) and the EvidenceOS harness docs: [EPISTEMIC_TRIAL_HARNESS.md](https://github.com/jverdicc/EvidenceOS/blob/main/docs/EPISTEMIC_TRIAL_HARNESS.md).
The complete blackbox narrative remains in [docs/THREAT_MODEL_BLACKBOX.md](docs/THREAT_MODEL_BLACKBOX.md).

## Docs map

- Start here (onboarding): [docs/START_HERE.md](docs/START_HERE.md)
- Role-based reader guide: [docs/reader_map.md](docs/reader_map.md)
- Threat model (blackbox walkthrough): [docs/THREAT_MODEL_BLACKBOX.md](docs/THREAT_MODEL_BLACKBOX.md)
- Threat model worked example: [docs/threat_model_worked_example.md](docs/threat_model_worked_example.md)
- UVP blackbox interface: [docs/uvp_blackbox_interface.md](docs/uvp_blackbox_interface.md)
- Protocol versioning/interoperability policy: [docs/PROTOCOL_VERSIONING.md](docs/PROTOCOL_VERSIONING.md)
- Clinical trials / Epistemic Trial Harness (EvidenceOS): [docs/EPISTEMIC_TRIAL_HARNESS.md](https://github.com/jverdicc/EvidenceOS/blob/main/docs/EPISTEMIC_TRIAL_HARNESS.md), [docs/TRIAL_HARNESS_ANALYSIS.md](https://github.com/jverdicc/EvidenceOS/blob/main/docs/TRIAL_HARNESS_ANALYSIS.md)

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

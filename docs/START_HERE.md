# START HERE

EvidenceOS + DiscOS provide a split architecture where EvidenceOS is the trusted verifier boundary and DiscOS is the fast-moving operator/client layer. EvidenceOS enforces lifecycle validity (`create → commit → freeze → seal → execute → fetch`) and fail-closed policy decisions, while DiscOS focuses on deterministic artifact preparation and transport. The core UVP value proposition is that repeated adaptive probing is treated as a stateful leakage problem, not a single-response prompt-quality problem. This means the trusted service enforces bounded information release, revocation/escalation behavior, and verifiable evidence publication at the protocol boundary. You can understand the model and operational implications through guided docs paths below without reading Rust code first. Demonstrations must remain safety-focused and must not provide real-world harmful instructions.

## Core docs path (new + high-signal)

Use this order to find the newest operational material quickly:

1. [Start here](START_HERE.md)
2. [Threat Model (Blackbox Walkthrough)](THREAT_MODEL_BLACKBOX.md)
3. [Paper reproduction guide](REPRODUCE_PAPER.md)
4. [Epistemic Trial Harness](EPISTEMIC_TRIAL_HARNESS.md)
5. [API refs: UVP blackbox interface](uvp_blackbox_interface.md)
6. [Deployment guardrails (dual-use / misuse)](DUAL_USE_AND_MISUSE.md)
7. [Security validation matrix](TEST_COVERAGE_MATRIX.md)

## Epistemic Trial Harness

The Epistemic Trial Harness is the deterministic trial runner for adaptive probing scenarios. DiscOS orchestrates workloads and captures artifacts; EvidenceOS applies the trusted policy checks.

- **What it is:** a reproducible sequence-level safety harness, backed by deterministic fixtures in [`docs/scenarios/`](scenarios/).
- **How to enable:** follow [docs/EPISTEMIC_TRIAL_HARNESS.md](EPISTEMIC_TRIAL_HARNESS.md) (DiscOS wrapper), then use the canonical EvidenceOS runbook linked there.
- **Where logs go:** harness/test logs and artifacts are documented in [docs/TEST_EVIDENCE.md](TEST_EVIDENCE.md) (including stable paths such as `artifacts/test.log`).
- **Analysis quickstart:** start from the blessed analysis entrypoint linked in [docs/EPISTEMIC_TRIAL_HARNESS.md](EPISTEMIC_TRIAL_HARNESS.md), then map outcomes to guarantees via [docs/TEST_COVERAGE_MATRIX.md](TEST_COVERAGE_MATRIX.md).

## Dual-use / production mode guardrails

- [Dual-use + misuse policy](DUAL_USE_AND_MISUSE.md)
- [Threat-model deployment assumptions](THREAT_MODEL_BLACKBOX.md#f-out-of-scope--deployment-assumptions)

## If you're an alignment researcher

Read in this order:

1. [UVP black-box API (inputs/outputs only)](uvp_blackbox_interface.md)
2. [THREAT_MODEL_BLACKBOX.md](THREAT_MODEL_BLACKBOX.md)
3. [Alignment positioning: UVP vs capability spillover](ALIGNMENT_SPILLOVER_POSITIONING.md)
4. [Paper overview (Sections 3, 5, and 10)](https://doi.org/10.5281/zenodo.18692345)
5. [Experiments summary in README (Evidence status matrix + paper-suite)](../README.md#evidence-status-matrix-paper-suite)

## If you're a systems/security engineer

Read in this order:

1. [EvidenceOS protocol spec (`proto/evidenceos.proto`)](https://github.com/jverdicc/EvidenceOS/blob/main/proto/evidenceos.proto)
2. [EvidenceOS daemon API / gRPC service contract](https://github.com/jverdicc/EvidenceOS/tree/main/crates/evidenceos-daemon)
3. [DiscOS determinism + durability notes (Technical Summary)](../README.md#technical-summary)

## If you're deploying

Read in this order:

1. [Deployment security envelope (keys + holdouts isolation)](THREAT_MODEL_BLACKBOX.md#f-out-of-scope--deployment-assumptions)
2. [EvidenceOS TLS/auth guidance](https://github.com/jverdicc/EvidenceOS#security--auth)
3. [DiscOS runbook-style lifecycle commands](../README.md#claim-lifecycle-commands)
4. [Dual-use + misuse policy](DUAL_USE_AND_MISUSE.md)

## If you're validating test guarantees

Read in this order:

1. [Test Coverage Matrix](TEST_COVERAGE_MATRIX.md)
2. [Test Coverage Parameters Appendix](TEST_COVERAGE_PARAMETERS.md)
3. [Test Evidence](TEST_EVIDENCE.md)



## Additional references

- [Paper reproduction guide](REPRODUCE_PAPER.md)
- [Threat model worked example](threat_model_worked_example.md)
- [Test Coverage Parameters Appendix](TEST_COVERAGE_PARAMETERS.md)
- [Test Evidence](TEST_EVIDENCE.md)
- [Alignment positioning](ALIGNMENT_SPILLOVER_POSITIONING.md)
- [Reader map](reader_map.md)
- [Docs index](README.md)

## ETL FAQ

- **Is this a blockchain?** No. A CT-style append-only Merkle transparency log (Evidence Transparency Log, ETL) is enough; blockchain is not required.
- **What security property do we need?** Append-only behavior plus verifiable inclusion/consistency proofs and signed tree heads.

## Terminology bridge

| Systems term | Safety/evals term |
| --- | --- |
| kernel / userland | trusted boundary / untrusted agent |
| transcript | interactive eval history |
| leakage `k` | bounded info release |

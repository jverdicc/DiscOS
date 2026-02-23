# START HERE

EvidenceOS + DiscOS provide a split architecture where EvidenceOS is the trusted verifier boundary and DiscOS is the fast-moving operator/client layer. EvidenceOS enforces lifecycle validity (`create → commit → freeze → seal → execute → fetch`) and fail-closed policy decisions, while DiscOS focuses on deterministic artifact preparation and transport. The core UVP value proposition is that repeated adaptive probing is treated as a stateful leakage problem, not a single-response prompt-quality problem. This means the trusted service enforces bounded information release, revocation/escalation behavior, and verifiable evidence publication at the protocol boundary. You can understand the model and operational implications through guided docs paths below without reading Rust code first. Demonstrations must remain safety-focused and must not provide real-world harmful instructions.

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


## ETL FAQ

- **Is this a blockchain?** No. A CT-style append-only Merkle transparency log (Evidence Transparency Log, ETL) is enough; blockchain is not required.
- **What security property do we need?** Append-only behavior plus verifiable inclusion/consistency proofs and signed tree heads.

## Terminology bridge

| Systems term | Safety/evals term |
| --- | --- |
| kernel / userland | trusted boundary / untrusted agent |
| transcript | interactive eval history |
| leakage `k` | bounded info release |

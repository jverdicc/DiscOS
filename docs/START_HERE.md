# START HERE

EvidenceOS + DiscOS provide a split architecture where EvidenceOS is the trusted verifier boundary and DiscOS is the fast-moving operator/client layer. EvidenceOS enforces lifecycle validity (`create → commit → freeze → seal → execute → fetch`) and fail-closed policy decisions, while DiscOS focuses on deterministic artifact preparation and transport. The core UVP value proposition is that repeated adaptive probing is treated as a stateful leakage problem, not a single-response prompt-quality problem. This means the trusted service enforces bounded information release, revocation/escalation behavior, and verifiable evidence publication at the protocol boundary. You can understand the model and operational implications through guided docs paths below without reading Rust code first.

## If you're an alignment researcher

Read in this order:

1. [THREAT_MODEL_BLACKBOX.md](THREAT_MODEL_BLACKBOX.md)
2. [Paper overview (Sections 3, 5, and 10)](https://doi.org/10.5281/zenodo.18692345)
3. [Experiments summary in README (Evidence status matrix + paper-suite)](../README.md#evidence-status-matrix-paper-suite)

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

## Terminology bridge

| Systems term | Safety/evals term |
| --- | --- |
| kernel / userland | trusted boundary / untrusted agent |
| transcript | interactive eval history |
| leakage `k` | bounded info release |

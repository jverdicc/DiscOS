# Docs Index

This folder contains threat-model explainers, trial-harness guides, reproducibility notes, test evidence, and deployment guardrails for DiscOS.

## Primary path (ordered)

1. [Start here](START_HERE.md)
2. [Threat Model (Blackbox Walkthrough)](THREAT_MODEL_BLACKBOX.md)
3. [Paper reproduction guide](REPRODUCE_PAPER.md)
4. [Epistemic Trial Harness](EPISTEMIC_TRIAL_HARNESS.md)
5. [API refs: UVP blackbox interface](uvp_blackbox_interface.md)
6. [Deployment and operations guardrails](DUAL_USE_AND_MISUSE.md)
7. [Security validation matrix](TEST_COVERAGE_MATRIX.md)

## Blackbox worked example (outsider-readable)

Treat EvidenceOS as a guarded service you call through DiscOS.

- **Inputs (tool calls / queries):** a caller submits lifecycle calls (`CreateClaim`, artifact commits, `Execute`) and may issue repeated adaptive queries.
- **EvidenceOS behavior over time:** the service meters leakage budget across the sequence, quantizes outputs, and moves to `throttle` or `freeze`/defer when policy limits are reached.
- **Outputs (what a user sees):** users see bounded responses while budget remains, then explicit throttled/frozen outcomes plus capsule + ETL receipt evidence for audit.

For deeper context, see the [threat model walkthrough](threat_model_worked_example.md), [UVP blackbox interface](uvp_blackbox_interface.md), and [Threat Model (Blackbox Walkthrough)](THREAT_MODEL_BLACKBOX.md) (Figure 1 analog for trust-boundary flow).

## Additional key docs (no orphan entrypoints)

- [Start here: threat model walkthrough](threat_model_worked_example.md)
- [Test Coverage Parameters Appendix](TEST_COVERAGE_PARAMETERS.md)
- [Test Evidence](TEST_EVIDENCE.md)
- [Alignment positioning: UVP vs capability spillover](ALIGNMENT_SPILLOVER_POSITIONING.md)
- [Issue Backlog](ISSUE_BACKLOG.md)
- [Reader map](reader_map.md)
- [Protocol versioning notes](PROTOCOL_VERSIONING.md)
- [Paper vs code tracking](PAPER_VS_CODE.md)
- [Implementation status](IMPLEMENTATION_STATUS.md)

## Scenario fixtures

Deterministic scenario inputs are under [`docs/scenarios/`](scenarios/).

## ETL FAQ

- **Is this a blockchain?** No. DiscOS/EvidenceOS require an Evidence Transparency Log (ETL): a CT-style append-only Merkle transparency log. A blockchain is not required.
- **What security property do we need?** Append-only operation with inclusion/consistency proofs and signed tree heads so auditors can verify history.

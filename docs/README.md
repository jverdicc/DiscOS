# Docs Index

This folder contains test evidence, scenario fixtures, and threat-model explainers for DiscOS.

## Start here

## Blackbox worked example (outsider-readable)

Treat EvidenceOS as a guarded service you call through DiscOS.

- **Inputs (tool calls / queries):** a caller submits lifecycle calls (`CreateClaim`, artifact commits, `Execute`) and may issue repeated adaptive queries.
- **EvidenceOS behavior over time:** the service meters leakage budget across the sequence, quantizes outputs, and moves to `throttle` or `freeze`/defer when policy limits are reached.
- **Outputs (what a user sees):** users see bounded responses while budget remains, then explicit throttled/frozen outcomes plus capsule + ETL receipt evidence for audit.

For deeper context, see the [threat model walkthrough](threat_model_worked_example.md), [UVP blackbox interface](uvp_blackbox_interface.md), and [Threat Model (Blackbox Walkthrough)](THREAT_MODEL_BLACKBOX.md) (Figure 1 analog for trust-boundary flow).

- [Start here: threat model walkthrough](threat_model_worked_example.md)
- [Threat Model (Blackbox Walkthrough)](THREAT_MODEL_BLACKBOX.md) — outsider-friendly narrative of baseline leakage vs EvidenceOS controls.
- [Test Coverage Matrix](TEST_COVERAGE_MATRIX.md) — mapping of guarantees to tests.
- [Test Evidence](TEST_EVIDENCE.md) — experiment/test evidence notes.
- [Issue Backlog](ISSUE_BACKLOG.md) — prioritized future work.
- [Dual-use + misuse policy](DUAL_USE_AND_MISUSE.md) — required deployment safety controls and prohibited uses.

## Scenario fixtures

Deterministic scenario inputs are under [`docs/scenarios/`](scenarios/).

## ETL FAQ

- **Is this a blockchain?** No. DiscOS/EvidenceOS require an Evidence Transparency Log (ETL): a CT-style append-only Merkle transparency log. A blockchain is not required.
- **What security property do we need?** Append-only operation with inclusion/consistency proofs and signed tree heads so auditors can verify history.


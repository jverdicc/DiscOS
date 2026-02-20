# Docs Index

This folder contains test evidence, scenario fixtures, and threat-model explainers for DiscOS.

## Start here

- [Start here: threat model walkthrough](threat_model_worked_example.md)
- [Threat Model (Blackbox Walkthrough)](THREAT_MODEL_BLACKBOX.md) — outsider-friendly narrative of baseline leakage vs EvidenceOS controls.
- [Test Coverage Matrix](TEST_COVERAGE_MATRIX.md) — mapping of guarantees to tests.
- [Test Evidence](TEST_EVIDENCE.md) — experiment/test evidence notes.
- [Issue Backlog](ISSUE_BACKLOG.md) — prioritized future work.

## Scenario fixtures

Deterministic scenario inputs are under [`docs/scenarios/`](scenarios/).

## ETL FAQ

- **Is this a blockchain?** No. DiscOS/EvidenceOS require an Evidence Transparency Log (ETL): a CT-style append-only Merkle transparency log. A blockchain is not required.
- **What security property do we need?** Append-only operation with inclusion/consistency proofs and signed tree heads so auditors can verify history.


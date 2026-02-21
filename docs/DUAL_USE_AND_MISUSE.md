# Dual-Use and Misuse Policy (EvidenceOS + DiscOS)

This document defines mandatory safety posture for high-stakes deployments using the EvidenceOS verifier boundary and DiscOS operator tooling.

## Intended use

EvidenceOS + DiscOS are intended for **defensive evaluation, governance, and bounded verification workflows** where:

- outputs are policy-scoped and auditable,
- lifecycle transitions are stateful and enforceable (`create → commit → freeze → seal → execute → fetch`),
- operators can provide verifiable evidence (capsules, ETL proofs, revocation records), and
- high-risk domains run with strict structured-output and escalation controls.

## Prohibited deployments

The following are prohibited:

- Configurations that intentionally bypass verifier-side policy controls for high-risk domains.
- Production deployments that permit unrestricted free-text output in high-risk safety contexts.
- Deployments that use DiscOS examples/docs to generate real-world harmful instructions.
- Any integration that weakens domain-specific schema enforcement for CBRN workflows.

## Human-in-the-loop requirements for high-risk domains

For high-risk domains (including CBRN):

- A qualified human reviewer must approve any operational decision path that reaches execution/release.
- Automated decisions may stage artifacts but must not be treated as autonomous authorization.
- Escalation (`heavy` lane) outcomes are review-required and should be recorded as audit evidence.

## Structured output requirements for high-risk domains

High-risk domains must use structured outputs with stable schema IDs.

- Canonical required schema: `CBRN_SC_V1` (`cbrn-sc.v1`).
- Schema mismatch in high-risk domains must be rejected or hard-escalated to the heavy lane.
- Free-text output in production is denied by default.

## Enforcement knobs (default secure)

The enforcement knobs below are fail-closed by default:

- `require_structured_outputs = true` for specified high-risk domains.
- `deny_free_text_outputs = true` in production.
- `force_heavy_lane_on_domain = ["CBRN"]`.
- `reject_on_high_risk_schema_mismatch = true`.

Reference implementation hook: `crates/evidenceos-core/src/safety_policy.rs`.

## DiscOS safe demonstration scenarios

DiscOS scenario fixtures and demos are for safe testing of controls, not harmful instruction generation.

- Use scenarios under `docs/scenarios/` to test throttling/escalation/freeze behavior.
- Do not repurpose examples for real-world CBRN operational guidance.
- Keep outputs machine-parseable and policy-bounded in integration tests.

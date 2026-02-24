# Threat Model (Blackbox Walkthrough)

> Audience: outsiders and integrators who want to understand the security value proposition quickly, without needing protocol internals.

## A) What problem UVP solves

The UVP model addresses a practical failure mode in AI/system evaluation: even when each individual response looks harmless, repeated adaptive interactions can gradually leak hidden evaluation structure (for example, holdout set boundaries or oracle behavior), eventually undermining trust in the evaluation process; EvidenceOS reduces this by enforcing kernel-visible leakage accounting and fail-closed state transitions at the interface where DiscOS submits and receives claim lifecycle calls.

## B) Entities and trust boundaries

- **DiscOS (untrusted userland/client):** operator tooling, claim preparation, orchestration, and RPC submission.
- **EvidenceOS (trusted kernel/service boundary):** authoritative policy checks, lifecycle state machine, leakage accounting, freeze/escalation decisions, and evidence publication.
- **Model or oracle under evaluation:** service being queried through governed interfaces.
- **Operator / auditor:** humans or automation consuming outputs and ETL-backed evidence.

### Boundary summary

- Treat DiscOS and caller behavior as potentially adversarial.
- Trust only what the EvidenceOS service attests through its lifecycle outcomes and audit artifacts.
- Assume hidden holdouts/secrets exist and must not be inferred through repeated probing.

---

## C) Worked example #1 — Adaptive evaluation leakage (toy holdout / decision boundary)

### Scenario (safe, toy framing)

A toy model is repeatedly queried by an adaptive client trying to infer a hidden holdout boundary. Each query asks for a judgment, then adjusts the next prompt based on prior output. No single call reveals much, but many calls can aggregate signal.

### Baseline system (no governed kernel controls)

- Client sends many near-neighbor queries.
- Service returns high-resolution outputs each time.
- Tiny output differences encode information about the hidden boundary.
- Over enough rounds, client estimates holdout structure with increasing confidence.

**Outcome:** hidden evaluation information leaks over time, even though each interaction seems “normal.”

### EvidenceOS-protected system (blackbox view)

EvidenceOS applies three control layers to the same interaction pattern:

1. **Quantization + hysteresis:** output granularity is intentionally bounded and jitter-resistant, reducing incremental signal extraction.
2. **Ledgered leakage budgets:** each governed interaction debits transcript support budget `k`; repeated probing consumes a finite allowance.
3. **Fail-closed freeze/escalation:** when policy thresholds are reached or invariants fail, execution is throttled/frozen rather than continuing to emit potentially revealing outputs.

**Outcome:** probing shifts from “eventual extraction with enough rounds” to a bounded, auditable failure mode.

### Blackbox I/O walkthrough (same toy task)

| Step | Input to EvidenceOS (via DiscOS) | Output from EvidenceOS |
| --- | --- | --- |
| 1 | `CreateClaim` with claim metadata + holdout ref + oracle id | `claim_id`, normalized topic context, initial policy/budget context |
| 2 | Repeated `Execute` calls carrying adaptive prompts | Quantized reply symbols + per-call leakage debit + updated status |
| 3 | Budget nears threshold | `THROTTLE` / stricter policy response |
| 4 | Threshold crossed or invariant violated | `FROZEN` (fail-closed), no continued revealing progression |
| 5 | Audit retrieval (`fetch-capsule`, ETL verification) | Signed capsule + transparency-log evidence of what happened |

---

## D) Worked example #2 (optional) — Cross-oracle probing on shared holdout

### Baseline

If Oracle A and Oracle B are evaluated against a shared hidden holdout, a client can probe A to reduce uncertainty, then use that information to improve extraction from B.

### EvidenceOS behavior

EvidenceOS can treat leakage accounting jointly across linked contexts (budget entanglement): pressure applied through A reduces remaining budget available for B in the same protected boundary.

**Result:** split-channel probing loses compounding advantage; the combined success path collapses into shared budget exhaustion and auditable throttling/freeze.

---

## E) Worked example #3 (optional) — Timing side-channel

### Baseline

When response latency tracks secret-dependent branches, a client may infer hidden structure from timing variation even if payloads are coarse.

### EvidenceOS behavior

- **Epoch settlement (DLC):** externally visible release happens on governed settlement boundaries rather than per-branch immediate timing.
- **Optional PLN controls:** additional policy-layer normalization can reduce timing-derived distinguishability at the interface.

**Result:** direct per-query timing correlation becomes less useful as an extraction signal.

---


## Quick links for reproducing the blackbox flow

- Onboarding and exact command paths: [docs/START_HERE.md](START_HERE.md)
- Scenario harness and deterministic trial runs: [docs/EPISTEMIC_TRIAL_HARNESS.md](EPISTEMIC_TRIAL_HARNESS.md)

## Glossary bridge (quick translation)

> **`leakage k`** = transcript support budget (how much distinguishable signal interaction can carry), **not** a cryptocurrency token.
>
> **`ETL`** = append-only transparency log for auditability and verification, **not** cryptocurrency.

---

## F) What EvidenceOS guarantees vs does **NOT** guarantee

### In scope (guaranteed by this model)

- Controls over **kernel-visible transcript leakage** at governed interfaces.
- Deterministic policy outcomes (allow/throttle/freeze) based on auditable state and budgets.
- Evidence artifacts showing what was accepted, limited, or blocked.

### Out of scope (not guaranteed by this model)

- Full endpoint/host security outside the trusted service boundary.
- Theft of keys, holdouts, or data via OS compromise, insider compromise, or supply-chain compromise.
- Safety claims about downstream behavior once output leaves the governed interface.

---

## Safety note

This document intentionally stays high-level and defensive. It explains why leakage controls matter and how blackbox policy enforcement changes outcomes, without giving procedural instructions for real-world exfiltration attacks.

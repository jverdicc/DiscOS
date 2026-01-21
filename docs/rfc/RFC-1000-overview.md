# RFC-1000: DiscOS overview, invariants, and config facets

- **Status:** Draft (industrial scaffold)
- **Last updated:** 2026-01-21
- **Target:** DiscOS v1.0 (EvidenceOS companion userland)
- **Audience:** implementers (userland/kernel), auditors, researchers

## 1. Purpose

DiscOS is the **Discovery Userland** that:
- represents hypotheses as typed IR (HIR),
- performs lineage-aware search and scheduling,
- runs multi-fidelity evaluations,
- assembles SafeClaims,
- and submits to an EvidenceOS kernel via UVP syscalls.

EvidenceOS remains authoritative for:
- evidence/adaptivity/privacy/integrity budgets,
- sealed holdout access via oracle,
- deterministic judging,
- proof-carrying capsules,
- transparency logging (ETL).

## 2. Invariants (non-negotiable)

1. **Untrusted-by-default**: all LLM outputs and plugins are adversarial until linted and sandboxed.
2. **Canonicalization**: stable canonical JSON used for hashing, provenance, and dedupe.
3. **Lineage-aware budgeting**: near-duplicate variants share family budgets (prevents budget laundering).
4. **One-way door**: SEALED lane is kernel-only; userland never reads holdout rows.
5. **Replayability**: certified/bundled artifacts must allow third-party verification.

## 3. Config facets (to avoid hard gating while prototyping)

DiscOS supports per-workspace policy facets:

- `gate_mode`: `"hard" | "soft" | "off"` (default: hard for lint; soft for meta-judge)
- `phys_lint`: on/off (default: on)
- `causal_lint`: on/off (default: off in AlphaHIR MVP)
- `meta_judge`: on/off (default: off)
- `human_signoff`: required/optional/off (default: off)
- `prefer_wasm_canary`: true/false (default: true)
- `heavy_lane`: `"local" | "container" | "microvm"` (default: local)
- `ledger_policy`: `"e_process" | "alpha_investing" | "lord" | "saffron"` (default: e_process)
- `sealed_enabled`: true/false (default: false in local-only mode)

Kernel remains authoritative. DiscOS policy controls admission/priority only.

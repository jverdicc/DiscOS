# RFC-1007: UVP syscalls: DiscOS ↔ EvidenceOS

- **Status:** Draft (industrial scaffold)
- **Last updated:** 2026-01-21
- **Target:** DiscOS v1.0 (EvidenceOS companion userland)
- **Audience:** implementers (userland/kernel), auditors, researchers

## 1. Overview

UVP defines a minimal syscall interface between untrusted discovery userland and trusted evidence kernel.

## 2. Syscalls (v0)

- uvp_announce(pds_manifest, causal_dag, policy_facet) -> context_id
- uvp_propose(context_id, hir_blob, hid_struct) -> claim_id
- uvp_budget_request(family_id, lane, amount) -> wealth_token
- uvp_vault_run(claim_id, wealth_token, data_view_id, lane) -> receipt_id
- uvp_certify(receipt_id, publish_policy) -> capsule_sig (+ ETL proof)

## 3. Error codes

Errors must be actionable mutation hints:
- E_DIM_INVALID(node_id, expected, got)
- E_LEAKAGE(node_id, lookahead_offset)
- E_REDUNDANT(neighbor_hid, dist)
- E_BANKRUPT(family_id, wealth)
- E_POLICY_VIOLATION(rule_id)

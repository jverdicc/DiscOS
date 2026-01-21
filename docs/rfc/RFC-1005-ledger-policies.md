# RFC-1005: Ledger policies: e-values, alpha-investing, LORD, SAFFRON

- **Status:** Draft (industrial scaffold)
- **Last updated:** 2026-01-21
- **Target:** DiscOS v1.0 (EvidenceOS companion userland)
- **Audience:** implementers (userland/kernel), auditors, researchers

## 1. Goal

EvidenceOS kernel meters epistemic budgets via a Conservation Ledger.
DiscOS requests budgets and uses remaining wealth to shape search.

## 2. Supported policy families (kernel-side)

### 2.1 E-values / e-processes (anytime-valid inference)
- supports optional stopping and sequential analysis
- natural "betting" interpretation for discovery

### 2.2 Alpha-investing
- maintains alpha-wealth that can be spent and replenished upon discoveries

### 2.3 LORD family (online FDR control)
- online false discovery rate control methods

### 2.4 SAFFRON (adaptive online FDR)
- adaptive online FDR using candidate thresholds / null proportion estimation

## 3. DiscOS behavior

DiscOS must:
- request wealth at the family level,
- stop promoting bankrupt families to SEALED,
- record ledger deltas and policy IDs in artifacts.

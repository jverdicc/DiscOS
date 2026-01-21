# RFC-1004: Scheduler: lane funnel, champions, and zombies

- **Status:** Draft (industrial scaffold)
- **Last updated:** 2026-01-21
- **Target:** DiscOS v1.0 (EvidenceOS companion userland)
- **Audience:** implementers (userland/kernel), auditors, researchers

## 1. Lanes

- FAST: cheap sketches and heuristics (DiscOS only)
- CANARY: deterministic sandbox run (prefer WASM)
- HEAVY: expensive training/validation runs in hardened sandbox
- SEALED: kernel-only oracle evaluation on holdout

## 2. Zombie detection

Kill hypotheses early when:
- runtime too high
- memory blowup
- NaN/Inf output rate high
- low incremental evidence probability

## 3. Policy hooks

Scheduler uses:
- family wealth remaining (queried from kernel)
- redundancy penalties
- meta-judge risk scores (optional)
- diversity bonus across families

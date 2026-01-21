# RFC-1003: Registry, lineage DAG, families, and dedup

- **Status:** Draft (industrial scaffold)
- **Last updated:** 2026-01-21
- **Target:** DiscOS v1.0 (EvidenceOS companion userland)
- **Audience:** implementers (userland/kernel), auditors, researchers

## 1. Registry role

Registry provides:
- content-addressed hypothesis store
- lineage graph
- family clustering for budgeting and scheduling

## 2. Family budgeting

A family is the unit of epistemic spending:
- new families get fresh allocation
- mutations draw from family wealth
- near-duplicates are rejected or de-prioritized

## 3. Fingerprints

- structural hash: exact dedupe
- behavioral signature: near-duplicate detection via sketches

## 4. Champion policy

Only the champion of a family may be promoted to SEALED lane.

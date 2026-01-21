# RFC-1008: Proof-Carrying Discovery Bundles (PCDB) wire format

- **Status:** Draft (industrial scaffold)
- **Last updated:** 2026-01-21
- **Target:** DiscOS v1.0 (EvidenceOS companion userland)
- **Audience:** implementers (userland/kernel), auditors, researchers

## 1. Goal

A PCDB is a self-contained artifact bundle suitable for:
- reproduction,
- verification,
- publication anchoring (ETL).

## 2. Contents

- canonical HIR + `hid_struct`
- lint report + config
- lane receipts (FAST/CANARY/HEAVY/SEALED)
- optional kernel capsule signature + ETL inclusion proof
- manifest.json with hashes of all files

## 3. Verification

Verifier recomputes:
- canonical hashes
- integrity of receipts
- signature chain (if capsule present)
- (optional) ETL inclusion proof

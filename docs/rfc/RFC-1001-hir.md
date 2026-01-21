# RFC-1001: HIR core, canonicalization, patches, and hashing

- **Status:** Draft (industrial scaffold)
- **Last updated:** 2026-01-21
- **Target:** DiscOS v1.0 (EvidenceOS companion userland)
- **Audience:** implementers (userland/kernel), auditors, researchers

## 1. HIR goals

HIR is a typed, deterministic representation of a hypothesis.

HIR must be:
- acyclic DAG (unless dialect explicitly allows bounded loops),
- deterministic (declared seeds; no implicit time/random/network),
- self-contained (declared inputs, operators, constraints),
- canonicalizable for hashing.

## 2. Canonicalization rules

- recursive key sorting for JSON objects
- deterministic float encoding
- commutative op normalization (where supported by dialect)
- alpha-renaming normalization (variable renaming invariance)

Structural ID:
`hid_struct = SHA256(canonical_json(HIR))`

Behavioral ID:
`hid_behav = SHA256(sketch(output_signature))`

## 3. Patch protocol (minimize "LLM JSON tax")

LLMs should output *patches*, not full HIR, after the first draft.

Patch ops (MVP):
- `ADD_NODE`
- `REMOVE_NODE`
- `UPDATE_NODE`
- `REWIRE_EDGE`
- `SET_METADATA`
- `SET_CONSTRAINT`

A patch is applied and then canonicalized; the result becomes a new `hid_struct`.

## 4. Dialects

- AlphaHIR (quant factors)
- EqHIR (symbolic regression skeletons)
- CausalHIR (identification designs)
- AlgoHIR (template + patch search)
- PhysHIR (PDS manifests, conservation/invariance checks)

This repo implements AlphaHIR + PhysHIR lint (PDS) first.

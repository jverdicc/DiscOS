# RFC-1002: PhysHIR: PDS dimensional analysis and invariance lint

- **Status:** Draft (industrial scaffold)
- **Last updated:** 2026-01-21
- **Target:** DiscOS v1.0 (EvidenceOS companion userland)
- **Audience:** implementers (userland/kernel), auditors, researchers

## 1. Motivation

Physics/chemistry verticals require rejecting nonsensical hypotheses *before execution*.
PhysHIR provides a dimensional type system (PDS) that enforces homogeneity.

## 2. Physical Dimension Signature (PDS)

PDS is a sparse exponent map over base dimensions (e.g., SI basis):
`L, M, T, I, Θ, N, J`, plus optional custom bases (e.g., USD as currency dimension).

Example:
- velocity: `L^1 T^-1`
- energy: `M^1 L^2 T^-2`
- dimensionless: `1`

## 3. Operator rules

- add/sub: require same PDS for both args
- mul: add exponents
- div: subtract exponents
- log/exp/trig: require dimensionless argument; output dimensionless
- clip(x, lo, hi): lo/hi must match x PDS
- constants: dimensionless unless explicitly annotated

## 4. Lint outputs

Errors must be actionable mutation hints:
- `E_DIM_INVALID(node_id, expected_pds, got_pds)`
- `E_DIM_MIXED_SUM(node_id, left_pds, right_pds)`
- `E_NON_DIMLESS_ARG(node_id, op, arg_pds)`

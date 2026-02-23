# Paper ↔ Repo Parity (Living Document)

This document is the fast truth table for review-time questions about what is implemented in DiscOS vs what is still paper/prototype scope.

## Snapshot intent

- **Scope:** DiscOS repository implementation status vs paper claims/artifacts.
- **Audience:** reviewers who need a yes/no/partial answer quickly without source-diving.
- **Update rule:** when paper-facing claims or implementation boundaries change, update this file in the same PR.

## Paper claim → repo parity matrix

| Paper section / claim | Repo implementation | Status | Links |
| --- | --- | --- | --- |
| DiscOS is the operator/client layer; EvidenceOS is the trusted verifier boundary. | Implemented as a Rust client/CLI + deterministic artifact tooling talking to EvidenceOS over gRPC. | Implemented | `README.md` (architecture split), `docs/START_HERE.md` |
| Paper artifact bundle contains reproducible experiments. | **Authoritative path is in EvidenceOS** (`artifacts/forc10/original_python`). DiscOS provides a thin wrapper (`paper_artifacts/reproduce_paper.py`) that delegates and never generates local synthetic outputs. | Implemented (delegated) | `docs/REPRODUCE_PAPER.md`, `paper_artifacts/reproduce_paper.py` |
| Mainline DiscOS runtime is Python. | **Not true for mainline.** Mainline DiscOS is Rust; Python in this repo is for wrappers, demos, and tests. | Not Implemented (for Python mainline) | `README.md`, `docs/REPRODUCE_PAPER.md` |
| PLN controls from the paper are fully shipped in DiscOS default path. | Documented as optional/high-assurance controls discussed in threat-model docs, not represented as fully-on-by-default DiscOS behavior. | Partial | `docs/THREAT_MODEL_BLACKBOX.md`, `docs/ALIGNMENT_SPILLOVER_POSITIONING.md` |
| Experiment 11/12 trends are reproducible and deterministic. | Implemented with deterministic simulations/tests; reviewer-facing paper reproduction should be executed through the EvidenceOS artifact path. | Implemented | `tests/experiments_integration.rs`, `crates/discos-core/tests/exp11_properties.rs`, `crates/discos-core/tests/exp12_tests.rs` |
| Appendix B structured claim DSL is fully implemented end-to-end as described in the paper appendix. | Structured claim canonicalization and bounded ingestion are implemented/tested; appendix-specific full DSL parity is not claimed here. | Partial | `README.md`, `docs/TEST_COVERAGE_MATRIX.md` |

## Explicit language note: toy experiments vs paper reproduction

DiscOS includes deterministic experiment code under `crates/discos-core/src/experiments/` for local validation and regression testing. Treat these as toy/internal models unless explicitly mapped to the authoritative EvidenceOS paper artifact runner.

## FORC artifact reproduction path

Use this path when reviewers ask for the paper-aligned artifact baseline:

1. **EvidenceOS authoritative command path:** `make -C artifacts/forc10/original_python verify`.
2. **DiscOS wrapper path (optional convenience):** `python3 paper_artifacts/reproduce_paper.py --evidenceos-repo ../EvidenceOS -- --verify`.
3. **Exact code snapshot convention for reviews:** pin and record `git rev-parse HEAD` for both DiscOS and EvidenceOS used to produce artifacts.

For cross-repo parity reviews, always include the paired EvidenceOS commit hash in notes.

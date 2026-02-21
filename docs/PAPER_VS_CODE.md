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
| Paper artifact bundle contains reproducible experiments. | Implemented through a vendored deterministic Python artifact generator (`paper_artifacts/reproduce_paper.py`) and documented commands. | Implemented | `docs/REPRODUCE_PAPER.md`, `paper_artifacts/reproduce_paper.py` |
| Mainline DiscOS runtime is Python. | **Not true for mainline.** Mainline DiscOS is Rust; Python in this repo is for paper artifact generation and selected demos/tests. | Not Implemented (for Python mainline) | `README.md` ("DiscOS (Rust)"), `docs/REPRODUCE_PAPER.md`, `crates/discos-cli/tests/paper_artifacts_smoke.rs` |
| PLN controls from the paper are fully shipped in DiscOS default path. | Documented as optional/high-assurance controls discussed in threat-model docs, not represented as fully-on-by-default DiscOS behavior. | Partial | `docs/THREAT_MODEL_BLACKBOX.md`, `docs/ALIGNMENT_SPILLOVER_POSITIONING.md` |
| Experiment 11/12 trends are reproducible and deterministic. | Implemented with deterministic simulations/tests and reproducible paper-suite artifacts. | Implemented | `tests/experiments_integration.rs`, `crates/discos-core/tests/exp11_properties.rs`, `crates/discos-core/tests/exp12_tests.rs`, `README.md` evidence matrix |
| Appendix B structured claim DSL is fully implemented end-to-end as described in the paper appendix. | Structured claim canonicalization and bounded ingestion are implemented/tested; appendix-specific full DSL parity is not claimed here. | Partial | `README.md` (Structured Claims + verification matrix), `docs/TEST_COVERAGE_MATRIX.md` |

## Explicit language note: Python artifacts vs Rust mainline

The paper artifact bundle includes Python reference + Python experiment reproduction paths. DiscOS mainline client/runtime in this repository is Rust. Python here is intentionally retained for reproducible artifact generation and a few test/demo paths; it is not the primary runtime surface.

## FORC artifact reproduction path

Use this path when reviewers ask for the paper-aligned artifact baseline:

1. **Paper/artifact snapshot DOI:** `10.5281/zenodo.18692345` (badge-linked from `README.md`).
2. **DiscOS repo artifact command path:** `make reproduce-paper` (or `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts`).
3. **Exact code snapshot convention for reviews:** pin and record `git rev-parse HEAD` from the DiscOS checkout used to produce artifacts, and include it in review notes alongside the generated `artifacts/paper-artifacts/index.json`.

For cross-repo parity reviews, pair this with the matching EvidenceOS commit used for the same run.

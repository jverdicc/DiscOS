# Paper ↔ Repo Parity (Living Document)

## Snapshot intent

- **Scope:** DiscOS repository implementation status vs paper claims/artifacts.
- **Audience:** reviewers who need a yes/no/partial answer quickly without source-diving.
- **Update rule:** when paper-facing claims or implementation boundaries change, update this file in the same PR.

## Pinned cross-repo snapshot

- DiscOS commit: `dba10c96cb5882d8738538c9a1cfe20c811ebfe8`
- EvidenceOS commit (authoritative paper runner): `4c1d7f2b0adf337df75fc85d4b7d84df4e99d0af`
- DOI artifact record: `10.5281/zenodo.18692345`

## Paper claim → repo parity matrix

| Paper section / claim | Repo implementation | Status | Links |
| --- | --- | --- | --- |
| DiscOS is the operator/client layer; EvidenceOS is the trusted verifier boundary. | Implemented as a Rust client/CLI + deterministic artifact tooling talking to EvidenceOS over gRPC. | Implemented | `README.md` (architecture split), `docs/START_HERE.md` |
| Paper artifact bundle contains reproducible experiments. | FULL mode fetches DOI artifacts with SHA-256 verification, then delegates to EvidenceOS `artifacts/forc10/original_python/run_all.py`. QUICK mode remains local deterministic compatibility checks only. | Implemented (split QUICK/FULL) | `artifacts/forc10/Makefile`, `artifacts/forc10/FULL_ARTIFACT_MANIFEST.json`, `scripts/fetch_forc10_artifacts.sh`, `docs/REPRODUCE_PAPER.md` |
| Mainline DiscOS runtime is Python. | Not true for mainline: this repository is Rust-first. Paper-faithful experiment reproduction points to the archived Python artifact runner in EvidenceOS. | Not Implemented (for Python mainline) | `README.md`, `docs/REPRODUCE_PAPER.md` |
| PLN controls from the paper are fully shipped in DiscOS default path. | Documented as optional/high-assurance controls discussed in threat-model docs, not represented as fully-on-by-default DiscOS behavior. | Partial | `docs/THREAT_MODEL_BLACKBOX.md`, `docs/ALIGNMENT_SPILLOVER_POSITIONING.md` |
| Experiment 11/12 trends are reproducible and deterministic. | Implemented with deterministic simulations/tests; reviewer-facing paper reproduction should be executed through FULL mode/EvidenceOS authoritative path. | Implemented | `tests/experiments_integration.rs`, `crates/discos-core/tests/exp11_properties.rs`, `crates/discos-core/tests/exp12_tests.rs`, `docs/REPRODUCE_PAPER.md` |
| Legacy `exp1`/`exp2` placeholders are used for paper reproduction in DiscOS. | Not implemented by design. Synthetic placeholders were removed from `discos-core`; paper-fidelity path is EvidenceOS Python artifacts. | Removed | `crates/discos-core/src/experiments/mod.rs`, `docs/REPRODUCE_PAPER.md` |
| Appendix B structured claim DSL is fully implemented end-to-end as described in the paper appendix. | Structured claim canonicalization and bounded ingestion are implemented/tested; appendix-specific full DSL parity is not claimed here. | Partial | `README.md`, `docs/TEST_COVERAGE_MATRIX.md` |

## Explicit language note: toy experiments vs paper reproduction

DiscOS includes deterministic experiment code under `crates/discos-core/src/experiments/` for local validation and regression testing. Treat these as toy/internal models unless explicitly mapped to FULL mode and the authoritative EvidenceOS paper artifact runner.

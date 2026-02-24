# Reproduce paper artifacts

DiscOS provides a two-lane reproduction story so reviewers can run fast CI-safe checks or the full paper-faithful pipeline.

## 1) QUICK verify (CI-friendly, no network)

```bash
make -C artifacts/forc10 verify
```

This runs `MODE=quick` by default and verifies the deterministic DiscOS-local compatibility harness against committed golden outputs.

**Covers:** deterministic regeneration + comparison of local legacy result/figure subset.

**Does not cover:** authoritative full paper artifact bundle and complete Python paper pipeline.

## 2) FULL reproduction (paper-faithful)

```bash
make -C artifacts/forc10 MODE=full verify EVIDENCEOS_REPO=../EvidenceOS
```

FULL mode is strict:
1. Fetches the DOI bundle pinned in `artifacts/forc10/FULL_ARTIFACT_MANIFEST.json`.
2. Verifies SHA-256 exactly against the manifest.
3. Aborts on checksum mismatch.
4. Runs the authoritative EvidenceOS Python runner (`artifacts/forc10/original_python/run_all.py --verify`) through `paper_artifacts/reproduce_paper.py`.

## Paper/code relationship (Python paper DiscOS vs Rust mainline DiscOS)

- The paper experiments were produced with the archived Python artifact pipeline hosted in EvidenceOS.
- Mainline DiscOS in this repository is Rust.
- Reviewer-facing paper-fidelity claims therefore point to the authoritative archived Python path in EvidenceOS, while DiscOS QUICK mode remains a deterministic local compatibility check.

## Pinned references used by FULL mode

- DOI record: `10.5281/zenodo.18692345`
- EvidenceOS runner commit: `4c1d7f2b0adf337df75fc85d4b7d84df4e99d0af`
- Manifest: `artifacts/forc10/FULL_ARTIFACT_MANIFEST.json`

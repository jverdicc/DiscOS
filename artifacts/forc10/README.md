# FORC10 local harness (legacy, non-authoritative)

> **Status:** Legacy compatibility harness for DiscOS-internal deterministic checks.
> It is **not** the authoritative paper reproduction path.

## Authoritative paper reproduction

Use EvidenceOS instead:

```bash
make -C ../EvidenceOS/artifacts/forc10/original_python verify
```

(or run the DiscOS wrapper: `python3 paper_artifacts/reproduce_paper.py --evidenceos-repo ../EvidenceOS -- --verify`).

## Why this directory still exists

The scripts and committed outputs here are retained only for backward-compatible CI/regression checks in DiscOS. They should be treated as toy/legacy outputs and must not be cited as paper-fidelity reproductions.

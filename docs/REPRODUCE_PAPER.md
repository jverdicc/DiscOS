# Reproduce paper artifacts (authoritative path)

DiscOS no longer ships a local synthetic paper generator. The authoritative FORC10 reproduction pipeline lives in the EvidenceOS repository under `artifacts/forc10/original_python/`.
Specifically, DiscOS does **not** provide local `exp1`/`exp2` paper experiment implementations; those placeholders were removed from `discos-core` to avoid misleading reviewer workflows.

## One authoritative command

From an EvidenceOS checkout:

```bash
make -C artifacts/forc10/original_python verify
```

This is the only command path that should be used for reviewer-facing paper reproduction claims.

## Running from DiscOS (wrapper)

If you are currently in a DiscOS checkout, use the wrapper script that delegates to EvidenceOS:

```bash
python3 paper_artifacts/reproduce_paper.py --evidenceos-repo ../EvidenceOS -- --verify
```

The wrapper fails fast when EvidenceOS is missing and intentionally does not synthesize experiment outputs.

## Why this changed

Previous DiscOS-local scripts produced deterministic placeholders for several experiments. Those placeholders are no longer acceptable for paper-fidelity claims. DiscOS now links to the EvidenceOS artifact runner as the source of truth.

# FORC10 reproduction entrypoint (DiscOS wrapper)

This directory exposes two explicit verification modes so reviewers can distinguish fast local checks from paper-faithful reproduction.

## Modes

### QUICK (CI-friendly, no network)

Runs the DiscOS-local deterministic compatibility harness:

```bash
make -C artifacts/forc10 verify
# equivalent: make -C artifacts/forc10 MODE=quick verify
```

What QUICK covers:
- deterministic regeneration of the committed legacy result/figure subset in `artifacts/forc10/golden`
- parity checks via `scripts/verify.py`

What QUICK does **not** cover:
- authoritative paper artifact bundle execution
- complete paper figure/table regeneration from the original Python pipeline

### FULL (paper-faithful)

Runs the authoritative Python reproduction path through EvidenceOS after fetching the DOI artifact bundle and verifying its checksum:

```bash
make -C artifacts/forc10 MODE=full verify EVIDENCEOS_REPO=../EvidenceOS
```

FULL mode behavior:
1. Downloads the bundle URL pinned in `FULL_ARTIFACT_MANIFEST.json`.
2. Verifies the bundle SHA-256 from that manifest.
3. Refuses to continue on checksum mismatch.
4. Delegates execution to `paper_artifacts/reproduce_paper.py`, which runs EvidenceOS `artifacts/forc10/original_python/run_all.py --verify`.

## Files

- `FULL_ARTIFACT_MANIFEST.json`: DOI URL + pinned SHA-256 + pinned EvidenceOS commit.
- `../../scripts/fetch_forc10_artifacts.sh`: download + SHA-256 verification helper.
- `scripts/`: DiscOS legacy deterministic harness used by QUICK mode only.

# FORC10 paper artifact harness

This directory provides an auditable, deterministic harness for reproducing DiscOS paper artifact outputs from a clean checkout.

## Prerequisites (pinned)

- Python 3.11+
- Rust toolchain pinned by `rust-toolchain.toml` in repository root
- No third-party Python packages are required (`paper_artifacts/requirements.txt` is stdlib-only)

## Expected runtime

- `make setup`: ~2-10 minutes (depends on first-time Rust workspace build)
- `make reproduce`: <10 seconds
- `make figures`: <5 seconds
- `make verify`: <15 seconds after setup

## One-command reproduction + verification

```bash
make -C artifacts/forc10 verify
```

This command regenerates all reproducible outputs in `artifacts/forc10/generated/` and checks them against committed goldens in `artifacts/forc10/golden/`.

## Targets

- `make setup`: install pinned prerequisites and build workspace
- `make reproduce`: regenerate raw JSON and normalized CSV summaries
- `make figures`: render figure/table CSVs used by the paper appendix
- `make verify`: compare generated outputs to goldens with numeric tolerance (`1e-12` absolute)

## Output layout

- `generated/raw/exp01.json` ... `generated/raw/exp12.json`
- `generated/raw/index.json`
- `generated/results/experiments.csv`
- `generated/results/summary.csv`
- `generated/results/index.json`
- `generated/figures/figure_exp01_mae.csv`
- `generated/figures/table_exp11_success.csv`

## Provenance and non-reproducible scope

- The harness reuses the deterministic script at `paper_artifacts/reproduce_paper.py` as the canonical source for experiment JSON.
- There is no external paper ZIP fetch in this workflow; the vendored deterministic Rust/Python-backed artifact generator is treated as the replacement for original paper ad-hoc scripts.
- Non-reproducible items: none currently (`generated/results/index.json` encodes this as an empty list).

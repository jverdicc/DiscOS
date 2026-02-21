# Reproduce paper artifacts (DiscOS)

This repository vendors a deterministic artifact suite under `paper_artifacts/` so a fresh clone can regenerate paper experiment outputs without downloading external bundles.

## One-command entrypoint

```bash
make reproduce-paper
```

Expected terminal output includes:

- a JSON line with `"ok": true`
- `artifacts/paper-artifacts/index.json`

## Outputs

All artifacts are written to:

- `artifacts/paper-artifacts/index.json`
- `artifacts/paper-artifacts/exp01.json` ... `artifacts/paper-artifacts/exp12.json`

The output path is consistent and can be overridden with:

```bash
python3 paper_artifacts/reproduce_paper.py --out <custom-dir>
```

## Paper experiment mapping

| Paper experiment | Repo command |
|---|---|
| 1 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 1` |
| 2 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 2` |
| 3 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 3` |
| 4 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 4` |
| 5 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 5` |
| 6 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 6` |
| 7 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 7` |
| 8 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 8` |
| 9 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 9` |
| 10 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 10` |
| 11 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 11` |
| 12 | `python3 paper_artifacts/reproduce_paper.py --out artifacts/paper-artifacts --experiments 12` |

## CI smoke subset

Use the smoke mode in CI to keep runtime low while still validating deterministic generation:

```bash
python3 paper_artifacts/reproduce_paper.py --smoke --out artifacts/paper-artifacts-smoke
```

Smoke mode generates Exp1, Exp11, and Exp12 plus an `index.json`.

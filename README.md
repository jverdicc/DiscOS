# DiscOS — Discovery OS userland for EvidenceOS (UVP-compatible)

**DiscOS** is the *untrusted userland* that generates, mutates, lint-checks, schedules, and executes hypotheses (HIRs)
in a multi‑fidelity funnel (FAST → CANARY → HEAVY → SEALED) and then submits **SafeClaims** to an EvidenceOS kernel
via **UVP syscalls**.

This repo is a **fully working reference system**:
- RFC-based spec in `docs/rfc/`
- JSON Schemas in `schemas/`
- Working Python implementation in `src/discos/`
- CLI (`python -m discos`) for init/lint/run/bundle
- Minimal AlphaHIR → WAT/WASM → CANARY demo (runs with or without `wasmtime`)

> ⚠️ Security note: this repo is a **userland reference**. Do not treat it as production-hardened.  
> In production, CANARY/HEAVY/SEALED execution must be attested and controlled by the EvidenceOS kernel.

## Quickstart

### 1) Create a virtual environment

```bash
python -m venv .venv
source .venv/bin/activate
pip install -U pip
```

### 2) Install DiscOS

```bash
pip install -e ".[dev,wasm]"
```

> `wasmtime` is optional. If it fails to install on your platform, run:
> `pip install -e ".[dev]"` and DiscOS will use a Python fallback runner for CANARY demos.

### 3) Run tests

```bash
pytest -q
```

### 4) Initialize a workspace

```bash
python -m discos init
```

This creates a `.discos/` directory that stores:
- content‑addressed hypothesis objects
- run receipts
- bundles

### 5) Create a sample AlphaHIR hypothesis

```bash
python -m discos alphahir new --name simple_return > simple_return.hir.json
```

### 6) Lint it

```bash
python -m discos lint simple_return.hir.json
```

### 7) Run CANARY (WASM preferred)

```bash
python -m discos run simple_return.hir.json --lane CANARY
```

### 8) Bundle into a Proof‑Carrying Discovery Bundle (PCDB)

```bash
python -m discos bundle simple_return.hir.json --out bundle_simple_return.zip
```

The bundle includes:
- canonical HIR + hashes
- lint report
- run receipts
- manifest

## VSCode setup

1. Open this folder in VSCode.
2. Install the Python extension.
3. Select interpreter: `.venv/bin/python`
4. Run tests from the testing panel or `pytest -q`.

Optional: install `ruff` and `mypy` (included in `[dev]`).

## Upload to GitHub

```bash
git init
git add -A
git commit -m "Initial DiscOS v1.0 scaffold"
git branch -M main
git remote add origin https://github.com/<you>/<repo>.git
git push -u origin main
```

## Relationship to EvidenceOS

DiscOS assumes an EvidenceOS kernel exists that implements:
- Conservation Ledger (e-values / online FDR / privacy / integrity lanes)
- Evidence Oracle (bandwidth-limited sealed feedback)
- Deterministic Judge
- Proof‑Carrying Capsules
- Evidence Transparency Log (ETL)

DiscOS talks to it via UVP syscalls (see RFC‑1007).

## License
Apache-2.0

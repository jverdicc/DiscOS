# DiscOS — Discovery OS userland for EvidenceOS (UVP-compatible)

**DiscOS** is a friendly, local workspace for exploring ideas safely before they ever touch a protected kernel.
You can draft a hypothesis, lint it for safety checks, run quick test lanes, and bundle the results for review.
Think of it as a **lab notebook + sandbox** that prepares experiments for EvidenceOS.

This repo is a **fully working reference system**:
- RFC-based spec in `docs/rfc/`
- JSON Schemas in `schemas/`
- Working Python implementation in `src/discos/`
- CLI (`python -m discos`) for init/lint/run/bundle
- Minimal AlphaHIR → WAT/WASM → CANARY demo (runs with or without `wasmtime`)

> ⚠️ Security note: this repo is a **userland reference**. Do not treat it as production-hardened.  
> In production, CANARY/HEAVY/SEALED execution must be attested and controlled by the EvidenceOS kernel.

## Getting started (new to DiscOS?)

If you are new, start here:

1. **Create a workspace**
   ```bash
   python -m discos init
   ```
2. **Generate a tiny example hypothesis**
   ```bash
   python -m discos alphahir new --name simple_return > simple_return.hir.json
   ```
3. **Lint it (safety/structure checks)**
   ```bash
   python -m discos lint simple_return.hir.json
   ```
4. **Run the CANARY lane (quick, deterministic demo)**
   ```bash
   python -m discos run simple_return.hir.json --lane CANARY
   ```
5. **Bundle the results for sharing**
   ```bash
   python -m discos bundle simple_return.hir.json --out bundle_simple_return.zip
   ```

If you want to understand the deeper design, skim `docs/rfc/` starting with the overview RFC.

## How DiscOS works (end-to-end)

DiscOS is a **local userland** that prepares hypotheses for kernel-grade evaluation. The canonical lifecycle looks like:

1. **Author a hypothesis** in HIR (currently AlphaHIR). HIR is a typed, deterministic DAG and can be canonicalized for hashing and lineage tracking.
2. **Lint for admissibility** (structure + optional physics/PDS checks). This blocks invalid ops, cycles, missing inputs, or dimensionally inconsistent expressions.
3. **Store in the workspace** (`.discos/`), using content-addressed HIR hashes (`hid_struct`) so duplicates are detected and lineage is trackable.
4. **Run execution lanes** (FAST/CANARY/HEAVY locally, SEALED only with a kernel). Receipts are written per lane.
5. **Bundle as a PCDB** (Proof-Carrying Discovery Bundle) that includes canonical HIR, receipts, and a manifest with hashes for reproducibility.

DiscOS never “finalizes” discoveries by itself. That is EvidenceOS’s job: the kernel meters evidence budgets, controls SEALED holdouts, and performs deterministic judging. DiscOS prepares artifacts and metadata for that pipeline.

## Execution overview (lanes + substrates)

DiscOS models a **lane funnel** that mirrors the EvidenceOS scheduler:

| Lane | Purpose | DiscOS behavior |
| --- | --- | --- |
| FAST | Cheap heuristics/sketches | Local-only quick passes |
| CANARY | Deterministic sandbox run | WASM-preferred execution, with a Python fallback |
| HEAVY | Expensive training/eval | MVP uses a local synthetic run (hardening is a roadmap item) |
| SEALED | Kernel-only holdout | DiscOS blocks this without EvidenceOS |

**Determinism + safety.** The CANARY lane emphasizes deterministic WASM execution (no WASI by default, pinned engine, NaN canonicalization). HEAVY is designed for hardened substrates (microVM/gVisor/nsjail), but the local reference implementation keeps it lightweight for demonstration. SEALED is intentionally unavailable in userland.

## Quant rationale & implementation details

DiscOS is quant-focused by design, with safeguards aimed at reproducible sequential discovery:

- **Canonical HIR + hashing.** Hypotheses are canonicalized and hashed into `hid_struct` identifiers for provenance and replayable bundles.
- **Lineage-aware budgeting.** The kernel’s Conservation Ledger meters per-family wealth (e-values, alpha-investing, LORD, SAFFRON). DiscOS uses those signals to guide which families advance.
- **Physics/units lint (PDS).** Optional Physical Dimension System checks enforce that arithmetic respects dimension algebra (e.g., sums require matching dimensions; `log/exp` require dimensionless inputs).
- **AlphaHIR operator whitelist.** MVP AlphaHIR focuses on common quantitative ops (`add`, `sub`, `mul`, `safe_div`, `abs`, `clip`, `log`, `exp`) and enforces DAG acyclicity.
- **Receipts + PCDBs.** Every lane emits receipts, and bundles package the canonical HIR, receipts, and manifest hashes for third‑party verification.

The reference implementation uses synthetic inputs for quick demos; the pipeline is designed to plug into real data sources and kernel‑attested runs once EvidenceOS is present.

## Who DiscOS is for

DiscOS is built for teams who want **auditability without slowing down exploration**:

- **Quant researchers** who need structured, repeatable hypothesis pipelines.
- **Safety/infra engineers** who need clear boundaries between untrusted ideation and kernel‑certified evaluation.
- **Auditors** who want replayable, self‑contained bundles with deterministic hashes and receipts.

If you are experimenting locally, DiscOS is a friendly “lab notebook.” If you are running production‑grade discovery, DiscOS is the **userland front‑end** to EvidenceOS.

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

## How do I commit and merge?

If you are new to Git, here is a simple happy-path:

1. **See what changed**
   ```bash
   git status --short
   ```
2. **Stage your changes**
   ```bash
   git add -A
   ```
3. **Commit with a message**
   ```bash
   git commit -m "Describe what you changed"
   ```
4. **Push your branch**
   ```bash
   git push -u origin your-branch-name
   ```
5. **Open a pull request** on GitHub and merge it once checks pass.

If you are working directly on `main`, you can skip the PR step and run:
```bash
git push origin main
```

## Relationship to EvidenceOS

DiscOS assumes an EvidenceOS kernel exists that implements:
- Conservation Ledger (e-values / online FDR / privacy / integrity lanes)
- Evidence Oracle (bandwidth-limited sealed feedback)
- Deterministic Judge
- Proof‑Carrying Capsules
- Evidence Transparency Log (ETL)

DiscOS talks to it via UVP syscalls (see RFC‑1007).

## Current scope, gaps, and roadmap

DiscOS is intentionally a **userland reference**. The pieces below are either MVP-only or depend on a real EvidenceOS kernel:

- **SEALED lane is not implemented in the CLI.** SEALED requires a kernel-backed oracle and is intentionally blocked in this repo.
- **Hardening layers are not wired in for MVP runs.** The RFCs describe microVM/gVisor/nsjail-style isolation for HEAVY and deterministic WASM constraints for CANARY, but the local demo runs use simple synthetic data.
- **Config facets are a scaffold.** Settings like `heavy_lane` and `sealed_enabled` are present for future use but do not yet map to real container/microVM backends in this repo.
- **Kernel-side services are required for full pipeline behavior.** Budgeting, sealed holdout access, and judging live in EvidenceOS; DiscOS only prepares and bundles artifacts.

If you want to contribute: help wire in a hardened runner, add a container/microVM backend, or prototype a real UVP client.

## Provenance and novelty

DiscOS builds on established ideas and tooling. The detailed maps live in:

- **Provenance map** (`docs/PROVENANCE.md`) — citations for adaptive holdout reuse,
  sequential testing, provenance standards, transparency logs, sandboxing, and
  evaluation platforms.
- **Novelty map** (`docs/NOVELTY.md`) — what is new (kernel-enforced conservation,
  lineage-aware budgets, proof-carrying discovery, UVP syscall boundary) vs.
  what is intentionally reused.

These documents make the dependency chain explicit and help reviewers distinguish
foundational work from system integration.

## License
Apache-2.0

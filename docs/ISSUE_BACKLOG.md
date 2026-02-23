# Issue Backlog Drafts

Use the following issue drafts to create GitHub issues.

---

## 1) README: Fix discos-cli flags + add reproducible sim/test commands

**Labels:** `documentation`, `good first issue`

**Description**
Update README Quickstart and command examples so all `discos-cli` flags match current CLI behavior.
Add reproducible simulation and test commands, including explicit seeds/parameters where applicable.

**Acceptance Criteria**
- Quickstart section runs exactly as written.
- README includes:
  - `cargo test --features sim --test experiments_integration`
- Commands are copy-pasteable and deterministic where expected.

---

## 2) Add discos-cli sim run exp0|exp11|exp12

**Labels:** `ux`, `good first issue`

**Description**
Add a top-level CLI path to run selected simulations without writing Rust code:
`discos-cli sim run exp0|exp11|exp12`.

**Acceptance Criteria**
- CLI accepts each experiment selector (`exp0`, `exp11`, `exp12`).
- No synthetic placeholder experiments are exposed as paper-reproduction commands.
- Simulations execute from CLI only (no code changes needed by user).
- Output is emitted as JSON and is machine-parseable.

---

## 3) Implement Experiment 3 (timing side-channel) as a sim + test

**Labels:** `enhancement`, `testing`

**Description**
Implement Experiment 3 as a deterministic simulation module and add coverage for core behavior.

**Acceptance Criteria**
- Deterministic simulation behavior (seedable/reproducible).
- Unit and/or integration tests validate expected outcomes.
- Test assertions cover timing-side-channel signal behavior at a qualitative level.

---

## 4) Implement Experiment 7b (correlation hole) sim for e-merge vs product

**Labels:** `math-heavy`, `enhancement`

**Description**
Implement Experiment 7b simulation comparing e-merge versus product construction under a correlation hole setup.

**Acceptance Criteria**
- Simulation reproduces expected qualitative relationship.
- Tests enforce inequality and/or bound properties between compared outputs.
- Results are deterministic under fixed seed/configuration.

---

## 5) Implement Experiment 12 (TopicID false-split sensitivity)

**Labels:** `enhancement`, `testing`

**Description**
Add Experiment 12 simulation to evaluate TopicID false-split sensitivity across a deterministic parameter sweep.

**Acceptance Criteria**
- Deterministic sweep implementation.
- Tests verify sweep outputs and expected sensitivity trend/constraints.
- Outputs are stable and suitable for CI comparison.

---

## 6) System test runner improvements: one command to spin up EvidenceOS + run full lifecycle

**Labels:** `testing`, `automation`

**Description**
Create/extend a system-test runner that starts EvidenceOS and executes full lifecycle tests in one command.

**Acceptance Criteria**
- One command runs end-to-end flow (startup, exercise, teardown).
- Artifacts are written under `artifacts/system-test/`.
- A summary report is produced with pass/fail and key metrics.

---

## 7) Python IPC example maintenance

**Labels:** `documentation`, `good first issue`

**Description**
Update Python IPC example docs and scripts to use `--data-dir` and ensure the example still executes successfully.

**Acceptance Criteria**
- Documentation uses `--data-dir` consistently.
- Python IPC example runs as documented.
- Any stale flags/usages are removed.

---

## 8) Docs: “Structured Claims in practice” with 3 safe sample JSON claims

**Labels:** `documentation`, `good first issue`

**Description**
Add a documentation section/page: “Structured Claims in practice” containing three safe sample JSON claims.
Include canonicalization notes and k-out-of-n budgeting explanation without procedural dual-use content.

**Acceptance Criteria**
- Three examples are schema-valid.
- Examples demonstrate canonicalization behavior.
- Includes clear k-out budgeting explanation.
- No dual-use procedural content.


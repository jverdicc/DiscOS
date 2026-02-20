# Reader map

Use this map to pick a fast path based on your role. Goal: get to a useful mental model in under 10 minutes.

## If you are a security reviewer

Read in this order:
1. [docs/threat_model_worked_example.md](threat_model_worked_example.md)
2. [docs/uvp_blackbox_interface.md](uvp_blackbox_interface.md)
3. [docs/THREAT_MODEL_BLACKBOX.md](THREAT_MODEL_BLACKBOX.md)
4. [docs/TEST_COVERAGE_MATRIX.md](TEST_COVERAGE_MATRIX.md)

What you should get:
- Threat model scope and assumptions.
- Blackbox control points (`CreateClaim` → `Freeze` → `Execute` → `Capsule/ETL`).
- Where leakage budgets, throttling, and freeze/escalation are tested.

## If you are an ML eval engineer

Read in this order:
1. [docs/uvp_blackbox_interface.md](uvp_blackbox_interface.md)
2. [examples/exfiltration_demo/](../examples/exfiltration_demo/)
3. [docs/threat_model_worked_example.md](threat_model_worked_example.md)
4. [docs/scenarios/](scenarios/)

What you should get:
- Expected blackbox I/O contract.
- How adaptive probing appears in practice and how controls respond.
- Scenario fixtures you can reuse for evaluation pipelines.

## If you are in governance / policy

Read in this order:
1. [docs/threat_model_worked_example.md](threat_model_worked_example.md)
2. [docs/uvp_blackbox_interface.md](uvp_blackbox_interface.md)
3. [docs/ALIGNMENT_SPILLOVER_POSITIONING.md](ALIGNMENT_SPILLOVER_POSITIONING.md)
4. [docs/TEST_EVIDENCE.md](TEST_EVIDENCE.md)

What you should get:
- Plain-language explanation of why sequence-level controls matter.
- Decision/state vocabulary that can be mapped to policy gates.
- Evidence artifacts available for audits and incident review.

## If you are a contributor

Read in this order:
1. [README.md](../README.md)
2. [docs/START_HERE.md](START_HERE.md)
3. [docs/README.md](README.md)
4. [docs/test_coverage_matrix.md](test_coverage_matrix.md)

Then run:
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

What you should get:
- Repo orientation, coding/testing expectations, and where to add new docs/examples safely.

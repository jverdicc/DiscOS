# Contributing to DiscOS

DiscOS is the interface between the agentic userland and the EvidenceOS kernel. We welcome contributions that expand the "real-world" reach of the protocol.

## Where We Need Help
- **Use Case Oracles:** Building adapters for new domains (e.g., Cybersecurity, Finance, Bio-safety).
- **Agent Resilience:** Improving how DiscOS handles kernel-level `FROZEN` states.
- **Latency Profiling:** Benchmarking the "Verification Tax" on agent performance.

## Getting Started
1. **The Sandbox:** Use the provided Docker Compose file to spin up a local EvidenceOS/DiscOS environment.
2. **Issue Labels:** Check for `good-first-issue` or `use-case-request`.
3. **Documentation:** We highly value improvements to the documentation and use-case examples.

## Local Validation
Run the same baseline checks required by CI before opening a PR:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Pull Request Expectations
Each pull request should include:
- A clear scope and rationale.
- Test evidence (commands + results).
- Notes on boundary conditions considered.
- Confirmation that behavior remains deterministic where applicable.

## Code of Conduct
We follow the Contributor Covenant. Please be professional and focused on the mission of verifiable AI safety.

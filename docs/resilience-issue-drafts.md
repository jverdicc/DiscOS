# Resilience Hardening Issue Drafts

Generated from:
`rg -n --glob "!**/node_modules/**" --glob "!**/target/**" "unwrap\(|panic!\(|TODO"`

## UNWRAP findings (14)

### crates/discos-core/src/boundary.rs:299
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: boundary.rs`
- Body: `The current implementation uses an unwrap at line 299. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/src/boundary.rs:300
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: boundary.rs`
- Body: `The current implementation uses an unwrap at line 300. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/src/boundary.rs:301
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: boundary.rs`
- Body: `The current implementation uses an unwrap at line 301. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/src/boundary.rs:308
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: boundary.rs`
- Body: `The current implementation uses an unwrap at line 308. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/src/boundary.rs:311
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: boundary.rs`
- Body: `The current implementation uses an unwrap at line 311. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/src/boundary.rs:319
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: boundary.rs`
- Body: `The current implementation uses an unwrap at line 319. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/src/boundary.rs:320
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: boundary.rs`
- Body: `The current implementation uses an unwrap at line 320. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/src/boundary.rs:335
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: boundary.rs`
- Body: `The current implementation uses an unwrap at line 335. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/src/boundary.rs:338
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: boundary.rs`
- Body: `The current implementation uses an unwrap at line 338. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/tests/structured_claims_vectors.rs:8
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: structured_claims_vectors.rs`
- Body: `The current implementation uses an unwrap at line 8. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/tests/structured_claims_vectors.rs:22
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: structured_claims_vectors.rs`
- Body: `The current implementation uses an unwrap at line 22. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/tests/structured_claims_vectors.rs:24
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: structured_claims_vectors.rs`
- Body: `The current implementation uses an unwrap at line 24. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/tests/structured_claims_vectors.rs:26
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: structured_claims_vectors.rs`
- Body: `The current implementation uses an unwrap at line 26. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/tests/structured_claims_vectors.rs:30
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: structured_claims_vectors.rs`
- Body: `The current implementation uses an unwrap at line 30. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

## PANIC findings (6)

### crates/discos-builder/src/lib.rs:260
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: lib.rs`
- Body: `The current implementation uses an unwrap at line 260. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/tests/structured_claims_vectors.rs:8
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: structured_claims_vectors.rs`
- Body: `The current implementation uses an unwrap at line 8. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/tests/structured_claims_vectors.rs:22
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: structured_claims_vectors.rs`
- Body: `The current implementation uses an unwrap at line 22. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/tests/structured_claims_vectors.rs:24
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: structured_claims_vectors.rs`
- Body: `The current implementation uses an unwrap at line 24. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/tests/structured_claims_vectors.rs:26
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: structured_claims_vectors.rs`
- Body: `The current implementation uses an unwrap at line 26. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

### crates/discos-core/tests/structured_claims_vectors.rs:30
- Title: `[Resilience] Replace unsafe unwrap in DiscOS: structured_claims_vectors.rs`
- Body: `The current implementation uses an unwrap at line 30. For an agentic userland environment, this is a crash vector. Refactor to implement a retry-logic or a graceful shutdown.`
- Labels: `agent-safety`, `technical-debt`, `disc-os-hardening`

## TODO findings (0)

- No TODO comments found in DiscOS-owned code (excluding vendored dependencies under `node_modules`).

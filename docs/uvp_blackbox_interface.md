# UVP Black-Box API (Inputs/Outputs Only)

This page is the **system contract** view of UVP: treat EvidenceOS as a trusted service with a narrow, auditable interface. You do not need internals to use this contract.

For a concrete attack-story walkthrough, see the worked example in [`docs/threat_model_worked_example.md`](threat_model_worked_example.md). For deeper formal and experimental context, see the paper ([DOI: 10.5281/zenodo.18692345](https://doi.org/10.5281/zenodo.18692345)).

## Trust boundary (black-box view)

```mermaid
flowchart LR
    subgraph U[DiscOS / Client (Untrusted)]
      A[Prepare claim metadata\nmanifest refs\nWASM hash\nholdout ref\noracle request]
      B[Submit lifecycle calls]
      C[Consume returned decision + artifacts]
    end

    subgraph T[EvidenceOS Kernel / Service (Trusted)]
      D[Lifecycle state machine\npolicy checks\nleakage accounting]
      E[Deterministic execution + canonicalization]
      F[Fail-closed decisions\n(throttle/freeze/reject)]
      G[Capsule + inclusion proof / log receipt]
    end

    A --> B --> D --> E --> F --> G --> C
```

## You provide

At the API boundary, the caller supplies only the claim package and execution request context:

- **Claim metadata** (identity/context fields for the claim).
- **WASM module hash** (the module identity to bind execution).
- **Artifact manifests** (inputs/resources by digest/reference).
- **Holdout reference** (opaque pointer/identifier for protected eval material).
- **Requested oracle** (which evaluation oracle/service to run against).

## Kernel guarantees

EvidenceOS guarantees these properties at the trusted boundary:

- **Determinism:** same frozen claim state + same governed inputs produce consistent outcomes.
- **Canonicalization:** artifacts and execution context are normalized before policy/accounting decisions.
- **Leak charging:** governed interactions debit leakage budget/accounting state.
- **Fail-closed behavior:** on invariant/policy failure, service returns safe terminal/limited outcomes rather than continuing permissive execution.

## You get back

The service returns a bounded output surface plus verifiable evidence:

- **Quantized decision/output** (policy-governed response, not unconstrained raw channel).
- **Capsule** (signed/attested lifecycle evidence package for the claim).
- **Inclusion proof / log receipt** (verifiable record anchored in transparency logging).

## Minimal lifecycle (call list)

Use this minimal sequence to understand the contract end-to-end:

1. `CreateClaim`
2. `CommitArtifacts`
3. `CommitWasm`
4. `Freeze`
5. `Execute`
6. `FetchCapsule`

## ETL append-only log (one-paragraph role)

The ETL (Evidence Transparency Log) is the append-only audit substrate for lifecycle outcomes: each relevant claim event is recorded so operators and auditors can verify what the trusted service accepted, rejected, throttled, froze, and emitted, without trusting the untrusted caller’s narration. In practice, the returned inclusion proof/log receipt links a claim outcome to this immutable history, enabling independent replay/verification of the system contract at the boundary.

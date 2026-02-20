# Alignment positioning: UVP and capability spillover

## What spillover means in this context

In alignment and eval security discussions, **capability spillover** usually means performance gains that move from one context to another (for example, from interaction with hidden evaluations into later behavior on new tasks). In this project, we use a narrower lens: UVP is designed to constrain **information released through an evaluation interface over repeated interaction**.

That distinction matters. UVP's leakage accounting (`k`) is about bounded release from transcripts and protocol responses. It does **not** claim to measure all forms of capability gain or transfer.

## Where UVP helps

### 1) Evaluation leakage control

UVP's lifecycle and policy checks are built to reduce adaptive extraction of holdout labels, decision boundaries, and scoring logic through repeated probing. This includes stateful controls (budgeting, freeze/seal transitions, and fail-closed decisions) that treat leakage as cumulative across turns, not independent per request.

### 2) Preventing cross-oracle probing on shared holdouts

A recurring risk in eval pipelines is querying multiple tools/oracles against the same hidden set and combining differences to infer the holdout. UVP helps by centralizing policy and evidence at a trusted boundary, making cross-oracle probing on shared holdouts easier to detect, limit, or block under one governance surface.

### 3) Reducing timing channels in high-assurance modes

When configured with high-assurance controls (for example DLC/PLN modes discussed in the paper), UVP can reduce timing-side signal available to an adaptive client. This does not eliminate all side channels, but it narrows one practical class of leakage routes during interactive scoring.

## Where UVP does NOT help

UVP is not a general solution to all capability spillover channels. In particular, it does not prevent:

- capability gains from reading papers, model weights, benchmarks, code, or other external tools/resources;
- compromise of serving endpoints, key theft, or other credential misuse;
- OS/runtime-level compromise or exfiltration below/around the protocol boundary.

Those are separate security and governance problems that require endpoint hardening, key management, infrastructure security, and broader model-release controls.

## Why this matters for alignment research programs

Many alignment and safety programs rely on hidden evaluations and interactive scoring loops to estimate model behavior under uncertainty. UVP's role is to harden that interface so repeated interaction leaks bounded information, with auditable evidence of policy decisions. This helps preserve eval validity longer while staying explicit about scope: leakage controls for interaction transcripts, not a complete account of all capability development pathways.

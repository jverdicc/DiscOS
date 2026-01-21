# RFC-1006: Execution substrate hardening: deterministic WASM, microVM, gVisor, nsjail

- **Status:** Draft (industrial scaffold)
- **Last updated:** 2026-01-21
- **Target:** DiscOS v1.0 (EvidenceOS companion userland)
- **Audience:** implementers (userland/kernel), auditors, researchers

## 1. CANARY lane: deterministic WASM

WASM execution can drift if host imports are nondeterministic or NaN payloads vary.
DiscOS records a determinism profile with:
- no WASI by default
- strict import whitelist
- NaN canonicalization
- pinned engine version

## 2. HEAVY lane: defense in depth

For sensitive workloads:
- Firecracker microVM
- gVisor container sandbox
- nsjail + seccomp-bpf syscall filtering

DiscOS provides config facets; kernel should enforce execution witnessing for SEALED runs.

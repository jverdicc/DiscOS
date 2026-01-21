# Novelty map (what is new vs. established)

This document summarizes the **defensible wedge** of novelty for the EvidenceOS/DiscOS
ecosystem, and distinguishes it from established work the project intentionally
reuses.

## Novel (the defensible wedge)

These items describe the primary novel contributions, anchored in the EvidenceOS
kernel repository and its associated system design:

* **Kernel-enforced epistemic validity conservation**  
  Turning sequential validity (e-values / alpha-wealth) from “paper math” into
  deny/allow syscalls that govern disclosure and evaluation.  
  EvidenceOS anchor: https://github.com/jverdicc/evidenceos
* **Lineage-aware budget enforcement**  
  Prevent “budget laundering” via near-duplicate hypothesis mutations by tracking
  family-level wealth and lineage.  
  EvidenceOS anchor: https://github.com/jverdicc/evidenceos
* **Proof-Carrying Discovery capsules**  
  Bind HIR + DataView provenance + lane receipts + ledger trace + kernel signature
  + transparency inclusion proof into a verifiable capsule.  
  EvidenceOS anchor: https://github.com/jverdicc/evidenceos
* **UVP-style syscall protocol**  
  A hardened boundary between untrusted DiscOS userland and a trusted EvidenceOS
  kernel (the “reality arbiter”).  
  EvidenceOS anchor: https://github.com/jverdicc/evidenceos

## Not novel (but required to be industrial)

These are deliberate dependencies on established work; see the full provenance
map for citations and references:

* Sequential testing math itself (α-investing, LORD, SAFFRON, e-values)
* Provenance standards (W3C PROV, OpenLineage)
* Transparency log / attestation concepts (Rekor, in-toto, SLSA)
* Sandboxing tech (Firecracker, gVisor, nsjail, WASM/WASI, Wasmtime)
* General “evaluation platform” pattern (CodaLab, EvalAI, OpenAI Evals)

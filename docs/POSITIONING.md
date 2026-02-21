# EvidenceOS Positioning & Risk Matrix

This document outlines where EvidenceOS and the DiscOS userland bridge fit within the broader AI safety and evaluation landscape. It explicitly defines the threat models the Universal Verification Protocol is designed to mitigate, and acknowledges the inherent dual-use nature of mathematical capability bounding.

## Section 1: Where EvidenceOS Operates

Standard AI safety evaluations largely focus on static intelligence or behavioral alignment. EvidenceOS operates on a fundamentally different layer: enforcing physical, mathematical bounds on dynamic, multi-step agentic state.

| Risk Category | Static Leaderboards | Behavioral Guardrails | EvidenceOS | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Toxicity / Bias** | 90% (RealToxicityPrompts) | 95% (Constitutional AI / RLHF) | **0%** — out of scope by design | — |
| **Single-Shot Hallucinations** | 80% (TruthfulQA) | 60% (System Prompts) | **0%** — out of scope by design | — |
| **Agentic Reward Hacking** | 10% | 30% | **85%** — Sealed Vault bounds state | Live |
| **Data Exfiltration / Privacy** | 0% | 20% | **95%** — $\epsilon, \delta$ ledger limits extraction | Sim-tested |
| **Capability Spillover** | 5% | 10% | **100%*** — meters cumulative leakage $k$ | Architecture specified |
| **CBRN Proliferation** | 0% | 15% | **100%*** — mathematical halt via $W$ depletion | Architecture specified |

*\* 100% coverage means EvidenceOS provides the only formal mechanism addressing this risk class. It does not mean deployment is complete or that all assumptions are resolved. See NullSpec governance (Open Problem 1 in the paper) for current limitations. Mechanism coverage is protocol-level. Sim-tested evidence is in TEST_COVERAGE_MATRIX.md. Live test evidence is in TEST_EVIDENCE.md.*

---

## Section 2: Industry Applications

The enforcement of a Conservation Ledger introduces novel defensive capabilities, but simultaneously creates vectors for adversarial optimization.

| Industry | Protective Application (Defense) | Dual-Use Risk (Weaponization) |
| :--- | :--- | :--- |
| **Intelligence / Cyber** | Zero-trust LLM containment over classified data. | Stealth extraction swarms staying under detection thresholds. |
| **Bio-Pharma / CBRN** | Synthesis gatekeeper preventing restricted sequence discovery. | Boundary-optimized probing (EvidenceOS's CBRN hardening specifically addresses this vector via TopicHash budgeting and PhysHIR envelopes). |
| **Quantitative Finance** | HFT flash-crash guardrails via $W$ depletion. | Regulatory evasion through bounded-intent certification. |
| **Consumer Privacy** | $\epsilon, \delta$ budget enforcement on personal agent queries. | Maximum PII extraction calibrated to OS privacy thresholds. |
| **AI Evaluation Infrastructure** | Adaptive benchmark gaming prevention. | NullSpec manipulation to inflate certified performance. |

---

## Section 3: Dual-Use Acknowledgment

EvidenceOS is inherently dual-use. Any mechanism that mathematically meters and bounds information extraction can be inverted to optimize extraction right up to that boundary. A DiscOS agent operating under a Conservation Ledger could, in principle, be used to guarantee its own probing stays beneath network anomaly detection thresholds rather than to prevent probing. 

The authors acknowledge this reality. We note that the deployment of EvidenceOS in high-risk domains requires strict governance controls outside the protocol itself—specifically NullSpec pre-commitment, operator key management, and cryptographic audit transparency—to close the dual-use gap. This open-source release is intended to advance foundational defensive systems research, not to provide a blueprint for offensive use.

---

## Section 4: Connection to Active Research

The **Capability Spillover** vector mapped above is the focus of active, global research programs (such as SPAR) studying how highly capable AI systems might accumulate disproportionate influence or forbidden knowledge through incremental, individually innocuous steps. 

EvidenceOS addresses this gap at the protocol level: rather than attempting to detect spillover behaviorally *after* the fact, it meters cumulative adaptivity leakage ($k$) as a physically conserved resource. By enforcing a hard boundary, it makes capability spillover mathematically expensive and ultimately impossible to execute past the predefined budget, shifting the paradigm from behavioral detection to architectural prevention.

---
*Reference: Universal Verification Protocol: Bounding AI Adaptivity Leakage via Conservation Ledgers (Zenodo DOI: [10.5281/zenodo.18685556](https://zenodo.org/records/18685556)).*


## Section 5: Use-Case Deep Dives (Defense vs. Dual-Use)

The matrix in Section 2 is intentionally compact. This section expands each row so operators, auditors, and policy teams can reason about how the same protocol primitives can either reduce risk or be strategically misused.

### 5.1 Intelligence / Cyber

**Defensive pattern:**
- Place DiscOS-mediated agent workflows between analysts and sensitive corpora, then require every retrieval/transformation step to pass through EvidenceOS policy checks.
- Track cumulative leakage (`k`) across related tools and data domains rather than per-endpoint in isolation.
- Freeze or escalate lanes when activity transitions from legitimate analysis to suspicious iterative extraction patterns.

**Dual-use pressure:**
- Adversaries may attempt to distribute probes across many low-volume workflows so each step appears benign while aggregate extraction remains high value.
- Without strong governance and review, a team could tune policy budgets for stealth persistence rather than genuine containment.

### 5.2 Bio-Pharma / CBRN

**Defensive pattern:**
- Enforce topic-bound claim execution so sequence design, pathway exploration, and protocol optimization requests are evaluated under explicit CBRN policy envelopes.
- Use deterministic manifests and auditable capsules to prove what was requested, what was denied, and when freeze transitions occurred.
- Apply strict budget depletion and mandatory escalation paths before high-risk synthesis-adjacent boundaries are reached.

**Dual-use pressure:**
- A malicious operator could attempt boundary surfing: maximizing useful sensitive signal while staying just below hard policy triggers.
- This is why protocol controls must be paired with external governance (NullSpec commitments, operator accountability, independent audit).

### 5.3 Quantitative Finance

**Defensive pattern:**
- Insert EvidenceOS between adaptive trading agents and exchange connectivity so every order strategy mutation consumes bounded operating budget (`W`).
- If an algorithm enters a runaway loop (for example, pathological rapid re-pricing), budget depletion deterministically pushes the lane to a halt state before uncontrolled escalation.
- This creates a mathematically enforced fail-closed behavior, not just a best-effort software guardrail.

**Dual-use pressure:**
- Firms could try to tune bounded behavior to sit just under surveillance thresholds, then present cryptographic logs as proof of operational discipline while still pursuing manipulative intent.
- Therefore, attestations about bounded operation must not be treated as standalone proof of lawful market behavior.

### 5.4 Consumer Privacy

**Defensive pattern:**
- Treat personal-data access as a finite privacy budget (`\epsilon, \delta`) consumed by adaptive querying over time.
- Enforce cross-session accounting so repeated “small” requests cannot silently aggregate into large de-anonymization risk.
- Require machine-verifiable evidence artifacts for audits, user redress, and regulator inspection.

**Dual-use pressure:**
- A platform may optimize extraction efficiency per unit budget, maximizing monetizable signal while claiming formal compliance.
- This creates an accountability challenge: protocol conformance can be necessary but insufficient for ethical data stewardship.

### 5.5 AI Evaluation Infrastructure

**Defensive pattern:**
- Use EvidenceOS to cap adaptive benchmark probing, preventing iterative overfitting to hidden holdouts through repeated micro-queries.
- Bind evaluations to deterministic claim manifests so downstream consumers can verify evaluation provenance and replay assumptions.
- Detect and stop cross-tool leakage composition that would otherwise defeat per-benchmark safeguards.

**Dual-use pressure:**
- Evaluation operators may game declared assumptions (NullSpec, budget choices, policy thresholds) to produce inflated “certified” outcomes.
- Cryptographic traceability reduces this risk, but only when independent reviewers can inspect the assumptions and challenge governance decisions.

# Provenance map (non-novel primitives and upstream influences)

This document catalogues prior work and external references that DiscOS/EvidenceOS
builds upon. The goal is to make provenance explicit, not to claim novelty where
there is none.

## Adaptive data analysis + holdout reuse

* **Preserving validity in adaptive data analysis (Reusable Holdout, Science 2015)**  
  https://www.science.org/doi/10.1126/science.aaa9375  
  **Not novel:** the core problem framing + “safe reuse of holdout via controlled access”.
* **Generalization in Adaptive Data Analysis and Holdout Reuse (arXiv 2015)**  
  https://arxiv.org/abs/1506.02629  
  **Not novel:** adaptive reuse theory + mechanisms for avoiding overfitting when queries are adaptive.
* **The Ladder: A Reliable Leaderboard for ML Competitions (arXiv 2015)**  
  https://arxiv.org/abs/1502.04585  
  **Not novel:** restricted-feedback oracle design for adaptive submissions.
* **Climbing a Shaky Ladder (Shaky Ladder variant, PDF)**  
  https://arxiv.org/pdf/1706.02733  
  **Not novel:** randomized restricted feedback for leaderboard/oracle robustness.

## Sequential / online error control (alpha-wealth, online FDR, e-values)

* **α-Investing (Foster & Stine, JRSSB 2008)**  
  https://doi.org/10.1111/j.1467-9868.2007.00643.x  
  **Not novel:** the “wealth/budget” abstraction for sequential testing.
* **On Online Control of False Discovery Rate (LOND/LORD, arXiv 2015)**  
  https://arxiv.org/abs/1502.06197  
  **Not novel:** core online-FDR LORD family.
* **Online rules for control of FDR and false discovery exceedance (Annals of Statistics, 2018 PDF)**  
  https://projecteuclid.org/journals/annals-of-statistics/volume-46/issue-2/Online-rules-for-control-of-false-discovery-rate-and-false/10.1214/17-AOS1559.pdf  
  **Not novel:** refined/rigorous online FDR guarantees and variants.
* **SAFFRON (arXiv 2018)**  
  https://arxiv.org/abs/1802.09098  
  **Not novel:** adaptive online FDR with alpha-wealth.
* **False discovery rate control with e-values (arXiv 2020)**  
  https://arxiv.org/abs/2009.02824  
  **Not novel:** e-BH / e-values for FDR control under dependence.
* **False Discovery Rate Control with E-values (JRSSB / Oxford Academic page)**  
  https://academic.oup.com/jrsssb/article/84/3/822/7056146  
  **Not novel:** peer-reviewed e-BH reference.
* **Game-Theoretic Statistics and Safe Anytime-Valid Inference (Project Euclid)**  
  https://projecteuclid.org/journals/statistical-science/volume-38/issue-4/Game-Theoretic-Statistics-and-Safe-Anytime-Valid-Inference/10.1214/23-STS894.full  
  **Not novel:** SAVI / e-process framing for anytime validity.
* **Hypothesis Testing with E-values (Ramdas & Wang, PDF “ebook”)**  
  https://stat.cmu.edu/~aramdas/ebook-final.pdf  
  **Not novel:** consolidated modern reference on e-values.

## Differential privacy “budget” pattern + tooling

* **OpenDP core library (DP algorithms)**  
  https://github.com/opendp/opendp  
  **Not novel:** DP tooling; useful substrate if you borrow DP-like “budgeting” patterns.
* **Google Differential Privacy libraries**  
  https://github.com/google/differential-privacy  
  **Not novel:** DP tooling; alternative library lineage.
* **OpenDP tools overview**  
  https://opendp.org/tools/  
  **Not novel:** DP ecosystem reference.

## Secure aggregation / multi-party evidence

* **Practical Secure Aggregation for Privacy-Preserving ML (Google Research page)**  
  https://research.google/pubs/practical-secure-aggregation-for-privacy-preserving-machine-learning/  
  **Not novel:** secure aggregation pattern (useful for federated evidence merges).
* **Same work (ACM DOI page)**  
  https://dl.acm.org/doi/10.1145/3133956.3133982  
  **Not novel:** canonical publication reference.

## Provenance standards / lineage

* **W3C PROV-DM (Provenance Data Model)**  
  https://www.w3.org/TR/prov-dm/  
  **Not novel:** provenance modeling (entities/activities/agents).
* **OpenLineage main repo**  
  https://github.com/OpenLineage/OpenLineage  
  **Not novel:** lineage event standard.
* **OpenLineage spec document**  
  https://github.com/OpenLineage/OpenLineage/blob/main/spec/OpenLineage.md  
  **Not novel:** spec baseline you can extend with “epistemic” facets.

## Transparency logs + attestations

* **Sigstore Rekor (transparency log, GitHub)**  
  https://github.com/sigstore/rekor  
  **Not novel:** append-only verifiable log concept + implementation.
* **Sigstore Rekor docs (logging overview)**  
  https://docs.sigstore.dev/logging/overview/  
  **Not novel:** operational reference for transparency logging.
* **in-toto (supply chain integrity framework, GitHub)**  
  https://github.com/in-toto/in-toto  
  **Not novel:** signed step attestations concept.
* **in-toto website**  
  https://in-toto.github.io/  
  **Not novel:** overview/spec access.
* **SLSA (Supply-chain Levels for Software Artifacts)**  
  https://slsa.dev/  
  **Not novel:** provenance/security maturity model; useful vocabulary.

## Proof-carrying concept ancestor

* **Proof-Carrying Code (Necula, ACM DOI)**  
  https://dl.acm.org/doi/10.1145/263699.263712  
  **Not novel:** “attach machine-checkable proof to untrusted artifact”.
* **Proof-Carrying Code (PDF copy)**  
  https://courses.grainger.illinois.edu/cs421/fa2010/papers/necula-pcc.pdf  
  **Not novel:** accessible copy for readers.

## Sandbox / deterministic execution substrate

* **Firecracker microVMs (GitHub)**  
  https://github.com/firecracker-microvm/firecracker  
  **Not novel:** microVM isolation layer.
* **gVisor (GitHub)**  
  https://github.com/google/gvisor  
  **Not novel:** “application kernel” sandbox.
* **nsjail (GitHub)**  
  https://github.com/google/nsjail  
  **Not novel:** process-level isolation tooling.
* **WASI (overview)**  
  https://wasi.dev/  
  **Not novel:** WASI system interface standard.
* **WASI (WebAssembly org repo)**  
  https://github.com/WebAssembly/WASI  
  **Not novel:** spec development repo.
* **Wasmtime runtime (GitHub)**  
  https://github.com/bytecodealliance/wasmtime  
  **Not novel:** WASM execution engine.
* **Wasmtime deterministic execution guide**  
  https://docs.wasmtime.dev/examples-deterministic-wasm-execution.html  
  **Not novel:** how to harden determinism (NaN canonicalization, etc.).

## Evaluation / reproducible research platforms

* **CodaLab Worksheets (site)**  
  https://worksheets.codalab.org/  
  **Not novel:** experiment bundles + reproducibility platform.
* **CodaLab Worksheets (GitHub)**  
  https://github.com/codalab/codalab-worksheets  
  **Not novel:** implementation reference.
* **EvalAI paper (arXiv)**  
  https://arxiv.org/abs/1902.03570  
  **Not novel:** evaluation-as-a-service platform concept.
* **OpenAI Evals (GitHub)**  
  https://github.com/openai/evals  
  **Not novel:** eval harness approach for LLMs/systems.
* **OpenAI platform “Working with evals”**  
  https://platform.openai.com/docs/guides/evals  
  **Not novel:** operational eval guidance.

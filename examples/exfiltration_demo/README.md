# Exfiltration demo: baseline oracle vs EvidenceOS-style mediation

Run from repository root:

```bash
make demo-exfil-baseline
make demo-exfil-evidenceos-mock
```

Expected behavior:
- baseline: high recovery accuracy (leaks labels)
- evidenceos-mock: near-chance recovery accuracy (leakage channel suppressed)

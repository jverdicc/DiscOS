from __future__ import annotations

from discos.hir.alphahir import alphahir_template_simple_return
from discos.admissibility.lint import lint_alphahir

def test_lint_ok_simple_return() -> None:
    hir = alphahir_template_simple_return().to_canonical_dict()
    rep = lint_alphahir(hir, phys_lint=True)
    assert rep["ok"] is True

def test_lint_detects_cycle() -> None:
    hir = alphahir_template_simple_return().to_canonical_dict()
    # Introduce a cycle by making n_open depend on n_out (invalid but for test)
    hir["nodes"].append({"id": "n_cycle", "kind": "op", "op": "add", "args": ["n_out", "n_open"]})
    # Make output depend on cycle and cycle depend on output by rewiring
    for n in hir["nodes"]:
        if n.get("id") == "n_out":
            n["args"] = ["n_cycle", "n_open"]
    rep = lint_alphahir(hir, phys_lint=False)
    assert rep["ok"] is False
    assert any(e["code"] == "E_CYCLE" for e in rep["errors"])

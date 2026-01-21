from __future__ import annotations

import numpy as np
from discos.hir.alphahir import alphahir_template_simple_return
from discos.admissibility.lint import lint_alphahir
from discos.compiler.wasm.watgen import generate_wat_for_alphahir
from discos.compiler.wasm.runner import run_canary

def test_watgen_exports() -> None:
    hir = alphahir_template_simple_return().to_canonical_dict()
    rep = lint_alphahir(hir)
    assert rep["ok"]
    wat = generate_wat_for_alphahir(hir, input_order=["open", "close"]).wat
    assert '(export "eval_series")' in wat

def test_canary_runs_fallback() -> None:
    hir = alphahir_template_simple_return().to_canonical_dict()
    wat = generate_wat_for_alphahir(hir, input_order=["open", "close"]).wat
    rng = np.random.default_rng(0)
    open_ = 100 + rng.normal(0, 1, size=1024).cumsum()
    close_ = open_ * (1 + rng.normal(0, 0.01, size=1024))
    out, rep = run_canary(wat, inputs={"open": open_, "close": close_}, input_order=["open", "close"], use_wasmtime=False)
    assert out.shape[0] <= 512
    assert "hid_behav" in rep.to_dict()

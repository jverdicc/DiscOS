from __future__ import annotations

import json
import numpy as np
from discos.hir.alphahir import alphahir_template_simple_return
from discos.admissibility.lint import lint_alphahir
from discos.compiler.wasm.watgen import generate_wat_for_alphahir
from discos.compiler.wasm.runner import run_canary

def main() -> None:
    hir = alphahir_template_simple_return().to_canonical_dict()
    print("HIR:", json.dumps(hir, indent=2))

    rep = lint_alphahir(hir)
    print("LINT:", json.dumps(rep, indent=2))

    wat = generate_wat_for_alphahir(hir, input_order=["open", "close"]).wat
    print("WAT (first 25 lines):")
    print("\n".join(wat.splitlines()[:25]))

    rng = np.random.default_rng(0)
    open_ = 100 + rng.normal(0, 1, size=2048).cumsum()
    close_ = open_ * (1 + rng.normal(0, 0.01, size=2048))
    out, canary = run_canary(wat, inputs={"open": open_, "close": close_}, input_order=["open", "close"])
    print("CANARY:", json.dumps(canary.to_dict(), indent=2))

if __name__ == "__main__":
    main()

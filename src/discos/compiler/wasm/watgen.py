from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, List

@dataclass(frozen=True)
class WatArtifact:
    wat: str
    exports: List[str]
    notes: List[str]

SUPPORTED_OPS = {"add", "sub", "mul", "safe_div", "neg", "abs", "clip"}

def generate_wat_for_alphahir(hir: Dict[str, Any], *, input_order: List[str]) -> WatArtifact:
    """Compile AlphaHIR (subset) to a pure WASM module (WAT) with no imports.

    Exports:
      - memory
      - eval_series(ptr_<input>..., out_ptr, n)

    Notes:
      - log/exp are not supported in this pure module (need imports or custom intrinsics).
    """
    nodes = hir["nodes"]
    output_id = hir["output_node"]

    by_id = {n["id"]: n for n in nodes}
    node_ids = list(by_id.keys())

    # topo
    indeg = {nid: 0 for nid in node_ids}
    succ = {nid: [] for nid in node_ids}
    for n in nodes:
        if n.get("kind") == "op":
            for a in n.get("args", []) or []:
                succ[a].append(n["id"])
                indeg[n["id"]] += 1
    q = [nid for nid, d in indeg.items() if d == 0]
    topo: List[str] = []
    while q:
        cur = q.pop()
        topo.append(cur)
        for nxt in succ[cur]:
            indeg[nxt] -= 1
            if indeg[nxt] == 0:
                q.append(nxt)

    # locals
    local_lines = [f"(local ${nid} f64)" for nid in topo]
    ptr_params = " ".join([f'(param $ptr_{name} i32)' for name in input_order])

    body: List[str] = []
    body.append("(local $i i32)")
    body.append("i32.const 0")
    body.append("local.set $i")
    body.append("(block $exit")
    body.append("  (loop $loop")
    body.append("    local.get $i")
    body.append("    local.get $n")
    body.append("    i32.ge_u")
    body.append("    br_if $exit")

    def push_local(nid: str) -> List[str]:
        return [f"    local.get ${nid}"]

    for nid in topo:
        n = by_id[nid]
        kind = n.get("kind")
        if kind == "input":
            name = n.get("name")
            if name not in input_order:
                raise ValueError(f"Input {name} not in input_order")
            body.extend([
                f"    local.get $ptr_{name}",
                "    local.get $i",
                "    i32.const 8",
                "    i32.mul",
                "    i32.add",
                "    f64.load",
                f"    local.set ${nid}",
            ])
        elif kind == "const":
            val = float(n.get("value", 0.0))
            body.extend([f"    f64.const {val}", f"    local.set ${nid}"])
        elif kind == "op":
            op = n.get("op")
            if op not in SUPPORTED_OPS:
                raise ValueError(f"Unsupported op in pure WASM: {op}")
            args = n.get("args", []) or []

            if op in {"add", "sub", "mul"}:
                a, b = args
                instr = {"add": "f64.add", "sub": "f64.sub", "mul": "f64.mul"}[op]
                body.extend(push_local(a) + push_local(b) + [f"    {instr}", f"    local.set ${nid}"])
            elif op == "safe_div":
                a, b = args
                body.extend([
                    *push_local(b),
                    "    f64.abs",
                    "    f64.const 1e-12",
                    "    f64.lt",
                    "    if (result f64)",
                    "      f64.const 0",
                    "    else",
                    *push_local(a),
                    *push_local(b),
                    "      f64.div",
                    "    end",
                    f"    local.set ${nid}",
                ])
            elif op == "neg":
                a = args[0]
                body.extend(["    f64.const -1", *push_local(a), "    f64.mul", f"    local.set ${nid}"])
            elif op == "abs":
                a = args[0]
                body.extend([*push_local(a), "    f64.abs", f"    local.set ${nid}"])
            elif op == "clip":
                x, lo, hi = args
                body.extend([
                    *push_local(x), *push_local(lo), "    f64.max",
                    *push_local(hi), "    f64.min",
                    f"    local.set ${nid}",
                ])
        else:
            raise ValueError(f"Unknown node kind: {kind}")

    body.extend([
        "    local.get $out",
        "    local.get $i",
        "    i32.const 8",
        "    i32.mul",
        "    i32.add",
        f"    local.get ${output_id}",
        "    f64.store",
        "    local.get $i",
        "    i32.const 1",
        "    i32.add",
        "    local.set $i",
        "    br $loop",
        "  )",
        ")",
    ])

    wat = "\n".join([
        "(module",
        "  (memory (export \"memory\") 2)",
        f"  (func (export \"eval_series\") {ptr_params} (param $out i32) (param $n i32)",
        "    " + "\n    ".join(local_lines),
        "    " + "\n    ".join(body),
        "  )",
        ")",
    ])

    notes = [
        "Pure WASM module; no imports; deterministic given deterministic engine profile.",
        "log/exp are not supported in this MVP pure module.",
    ]
    return WatArtifact(wat=wat, exports=["memory", "eval_series"], notes=notes)

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, List, Optional

from discos.hir.phys import PDS

ALLOWED_OPS = {"add", "sub", "mul", "safe_div", "neg", "abs", "clip", "log", "exp"}

class LintError(Exception):
    def __init__(self, code: str, message: str, details: Optional[Dict[str, Any]] = None) -> None:
        super().__init__(message)
        self.code = code
        self.message = message
        self.details = details or {}

    def to_dict(self) -> Dict[str, Any]:
        return {"code": self.code, "message": self.message, "details": self.details}

def lint_alphahir(hir: Dict[str, Any], *, phys_lint: bool = True) -> Dict[str, Any]:
    errors: List[Dict[str, Any]] = []
    warnings: List[Dict[str, Any]] = []

    nodes = hir.get("nodes", [])
    node_id_list = [n.get("id") for n in nodes if n.get("id") is not None]
    node_ids = set(node_id_list)
    out_id = hir.get("output_node")

    if len(node_id_list) != len(node_ids):
        counts: Dict[str, int] = {}
        for nid in node_id_list:
            counts[nid] = counts.get(nid, 0) + 1
        dupes = sorted([nid for nid, count in counts.items() if count > 1])
        errors.append({"code": "E_DUP_NODE_ID", "details": {"duplicate_ids": dupes}})

    if out_id not in node_ids:
        errors.append({"code": "E_OUTPUT_MISSING", "details": {"output_node": out_id}})

    # op whitelist and args exist
    for n in nodes:
        if n.get("kind") == "op":
            op = n.get("op")
            if op not in ALLOWED_OPS:
                errors.append({"code": "E_OP_FORBIDDEN", "details": {"node_id": n.get("id"), "op": op}})
            for a in n.get("args", []) or []:
                if a not in node_ids:
                    errors.append({"code": "E_ARG_MISSING", "details": {"node_id": n.get("id"), "arg": a}})

    # acyclicity (Kahn)
    succ: Dict[str, List[str]] = {nid: [] for nid in node_ids}
    indeg: Dict[str, int] = {nid: 0 for nid in node_ids}
    for n in nodes:
        if n.get("kind") == "op":
            for a in n.get("args", []) or []:
                succ[a].append(n["id"])
                indeg[n["id"]] += 1
    q = [nid for nid, d in indeg.items() if d == 0]
    visited = 0
    while q:
        cur = q.pop()
        visited += 1
        for nxt in succ[cur]:
            indeg[nxt] -= 1
            if indeg[nxt] == 0:
                q.append(nxt)
    if visited != len(node_ids):
        errors.append({"code": "E_CYCLE", "details": {"visited": visited, "total": len(node_ids)}})

    # physics lint (PDS) (optional)
    inferred_pds: Dict[str, PDS] = {}

    if phys_lint:
        inputs = hir.get("inputs", {})
        # seed input nodes
        for n in nodes:
            if n.get("kind") == "input":
                name = n.get("name")
                if name in inputs:
                    inferred_pds[n["id"]] = PDS.parse(inputs[name].get("pds", "1"))
                else:
                    errors.append({"code": "E_INPUT_MISSING", "details": {"node_id": n.get("id"), "name": name}})

        # topological compute
        # reuse topo order by indeg from earlier computation:
        # reconstruct topo quickly
        indeg2 = {nid: 0 for nid in node_ids}
        succ2: Dict[str, List[str]] = {nid: [] for nid in node_ids}
        for n in nodes:
            if n.get("kind") == "op":
                for a in n.get("args", []) or []:
                    succ2[a].append(n["id"])
                    indeg2[n["id"]] += 1
        q2 = [nid for nid, d in indeg2.items() if d == 0]
        topo: List[str] = []
        while q2:
            cur = q2.pop()
            topo.append(cur)
            for nxt in succ2[cur]:
                indeg2[nxt] -= 1
                if indeg2[nxt] == 0:
                    q2.append(nxt)

        by_id = {n["id"]: n for n in nodes}
        for nid in topo:
            n = by_id[nid]
            if n.get("kind") == "const":
                inferred_pds[nid] = PDS.dimensionless()
            if n.get("kind") == "op":
                op = n.get("op")
                args = n.get("args", []) or []
                def p(arg: str) -> Optional[PDS]:
                    return inferred_pds.get(arg)

                if op in ("neg", "abs"):
                    pa = p(args[0])
                    if pa is None:
                        continue
                    inferred_pds[nid] = pa
                elif op in ("add", "sub"):
                    pa, pb = p(args[0]), p(args[1])
                    if pa and pb and (not pa.same_as(pb)):
                        errors.append({"code": "E_DIM_MIXED_SUM", "details": {"node_id": nid, "left": pa.to_canonical_str(), "right": pb.to_canonical_str()}})
                    inferred_pds[nid] = pa or pb or PDS.dimensionless()
                elif op == "mul":
                    pa, pb = p(args[0]), p(args[1])
                    if pa and pb:
                        inferred_pds[nid] = pa * pb
                elif op == "safe_div":
                    pa, pb = p(args[0]), p(args[1])
                    if pa and pb:
                        inferred_pds[nid] = pa / pb
                elif op == "clip":
                    px, plo, phi = p(args[0]), p(args[1]), p(args[2])
                    if px and plo and (not px.same_as(plo)):
                        errors.append({"code": "E_DIM_INVALID", "details": {"node_id": nid, "expected": px.to_canonical_str(), "got": plo.to_canonical_str()}})
                    if px and phi and (not px.same_as(phi)):
                        errors.append({"code": "E_DIM_INVALID", "details": {"node_id": nid, "expected": px.to_canonical_str(), "got": phi.to_canonical_str()}})
                    inferred_pds[nid] = px or PDS.dimensionless()
                elif op in ("log", "exp"):
                    pa = p(args[0])
                    if pa and pa.to_canonical_str() != "1":
                        errors.append({"code": "E_NON_DIMLESS_ARG", "details": {"node_id": nid, "op": op, "arg_pds": pa.to_canonical_str()}})
                    inferred_pds[nid] = PDS.dimensionless()

        # output pds must match declared
        declared = PDS.parse(hir.get("declared_output_pds", "1"))
        outp = inferred_pds.get(out_id) if out_id else None
        if outp and (not outp.same_as(declared)):
            errors.append({"code": "E_DIM_INVALID", "details": {"node_id": out_id, "expected_pds": declared.to_canonical_str(), "got_pds": outp.to_canonical_str()}})

    ok = len(errors) == 0
    return {"ok": ok, "errors": errors, "warnings": warnings}

def require_ok(report: Dict[str, Any]) -> None:
    if not report.get("ok", False):
        raise LintError("E_ADMISSIBILITY", "HIR failed lint", {"errors": report.get("errors", [])})

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, List, Literal, Optional

PatchOp = Literal["ADD_NODE", "REMOVE_NODE", "UPDATE_NODE", "REWIRE_EDGE", "SET_METADATA"]

@dataclass(frozen=True)
class HIRPatch:
    ops: List[Dict[str, Any]]

def apply_patch(hir: Dict[str, Any], patch: HIRPatch) -> Dict[str, Any]:
    """Apply a patch to a HIR dict (MVP).

    This function is intentionally strict; invalid ops raise ValueError.
    """
    out = json_clone(hir)

    for op in patch.ops:
        kind = op.get("op")
        if kind == "SET_METADATA":
            meta = out.setdefault("metadata", {})
            meta.update(op.get("metadata", {}))
            continue

        if kind == "ADD_NODE":
            node = op["node"]
            out.setdefault("nodes", []).append(node)
            continue

        if kind == "REMOVE_NODE":
            node_id = op["node_id"]
            out["nodes"] = [n for n in out.get("nodes", []) if n.get("id") != node_id]
            continue

        if kind == "UPDATE_NODE":
            node_id = op["node_id"]
            patch_fields = op.get("fields", {})
            found = False
            for n in out.get("nodes", []):
                if n.get("id") == node_id:
                    n.update(patch_fields)
                    found = True
                    break
            if not found:
                raise ValueError(f"UPDATE_NODE: missing node {node_id}")
            continue

        if kind == "REWIRE_EDGE":
            node_id = op["node_id"]
            new_args = op["args"]
            found = False
            for n in out.get("nodes", []):
                if n.get("id") == node_id:
                    n["args"] = new_args
                    found = True
                    break
            if not found:
                raise ValueError(f"REWIRE_EDGE: missing node {node_id}")
            continue

        raise ValueError(f"Unknown patch op: {kind}")

    return out

def json_clone(obj: Any) -> Any:
    import json
    return json.loads(json.dumps(obj))

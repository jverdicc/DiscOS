from __future__ import annotations

from discos.hir.alphahir import alphahir_template_simple_return
from discos.hir.patch import HIRPatch, apply_patch

def test_apply_patch_set_metadata() -> None:
    hir = alphahir_template_simple_return().to_canonical_dict()
    patch = HIRPatch(ops=[{"op": "SET_METADATA", "metadata": {"foo": "bar"}}])
    out = apply_patch(hir, patch)
    assert out["metadata"]["foo"] == "bar"

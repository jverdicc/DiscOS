# Copyright 2026 Joseph Verdicchio
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0

from __future__ import annotations

import io
import json
import zipfile
from pathlib import Path
from typing import Any, Dict

from discos.registry.canonicalize import canonical_json, sha256_hex
from discos.registry.workspace import Workspace

def build_pcdb_bundle(ws: Workspace, hid_struct: str, out_zip: Path) -> Path:
    """Build a Proof-Carrying Discovery Bundle (PCDB).

    MVP contents:
    - hir.json (canonical)
    - manifest.json (hashes)
    - receipts/*.json
    """
    hir = ws.load_hypothesis(hid_struct)
    hir_canon = canonical_json(hir)

    receipts = ws.list_receipts(hid_struct)

    manifest: Dict[str, Any] = {
        "hid_struct": hid_struct,
        "files": {},
    }

    def add_file(z: zipfile.ZipFile, arcname: str, data: bytes) -> None:
        z.writestr(arcname, data)
        manifest["files"][arcname] = sha256_hex(data.decode("utf-8") if arcname.endswith(".json") else data.hex())

    with zipfile.ZipFile(out_zip, "w", zipfile.ZIP_DEFLATED) as z:
        add_file(z, "hir.json", json.dumps(json.loads(hir_canon), indent=2).encode("utf-8"))

        for rp in receipts:
            add_file(z, f"receipts/{rp.name}", rp.read_bytes())

        add_file(z, "manifest.json", json.dumps(manifest, indent=2).encode("utf-8"))

    return out_zip

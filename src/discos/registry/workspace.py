from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Dict, Optional

from discos.config import DiscOSConfig
from discos.registry.canonicalize import canonical_json, sha256_hex

@dataclass(frozen=True)
class Workspace:
    cfg: DiscOSConfig

    @property
    def root(self) -> Path:
        return self.cfg.workspace_path()

    def init(self) -> None:
        self.cfg.workspace_path().mkdir(parents=True, exist_ok=True)
        self.cfg.objects_path().mkdir(parents=True, exist_ok=True)
        self.cfg.receipts_path().mkdir(parents=True, exist_ok=True)
        self.cfg.bundles_path().mkdir(parents=True, exist_ok=True)

    def store_hypothesis(self, hir: Dict[str, Any], *, family_id: str) -> str:
        """Store HIR in content-addressed object store and return hid_struct."""
        hid_struct = sha256_hex(canonical_json(hir))
        obj_path = self.cfg.objects_path() / f"{hid_struct}.json"
        if not obj_path.exists():
            obj_path.write_text(json.dumps(hir, indent=2), encoding="utf-8")
            meta = {"hid_struct": hid_struct, "family_id": family_id}
            (self.cfg.objects_path() / f"{hid_struct}.meta.json").write_text(json.dumps(meta, indent=2), encoding="utf-8")
        return hid_struct

    def load_hypothesis(self, hid_struct: str) -> Dict[str, Any]:
        p = self.cfg.objects_path() / f"{hid_struct}.json"
        return json.loads(p.read_text(encoding="utf-8"))

    def write_receipt(self, hid_struct: str, *, lane: str, payload: Dict[str, Any]) -> Path:
        rec = {"hid_struct": hid_struct, "lane": lane, "payload": payload}
        p = self.cfg.receipts_path() / f"{hid_struct}.{lane.lower()}.receipt.json"
        p.write_text(json.dumps(rec, indent=2), encoding="utf-8")
        return p

    def list_receipts(self, hid_struct: str) -> list[Path]:
        return sorted(self.cfg.receipts_path().glob(f"{hid_struct}.*.receipt.json"))

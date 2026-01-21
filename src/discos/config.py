from __future__ import annotations

from pydantic import BaseModel, Field
from typing import Literal, Optional
import yaml
from pathlib import Path

GateMode = Literal["hard", "soft", "off"]
LaneMode = Literal["local", "container", "microvm"]
LedgerPolicy = Literal["e_process", "alpha_investing", "lord", "saffron"]

class DiscOSConfig(BaseModel):
    # Global gating mode (lint).
    gate_mode: GateMode = "hard"

    # Optional gates
    phys_lint: bool = True
    causal_lint: bool = False
    meta_judge: bool = False
    human_signoff: bool = False

    # Execution preferences
    prefer_wasm_canary: bool = True
    heavy_lane: LaneMode = "local"  # local runner for MVP
    sealed_enabled: bool = False    # local dev defaults to false

    # Ledger policy hint (kernel enforces actual policy)
    ledger_policy: LedgerPolicy = "e_process"

    # Workspace paths
    workspace_dir: str = ".discos"
    objects_dir: str = "objects"
    receipts_dir: str = "receipts"
    bundles_dir: str = "bundles"

    # WASM determinism profile record
    wasm_nan_canonicalization: bool = True
    wasm_allow_simd: bool = False

    @classmethod
    def load(cls, path: str | Path | None) -> "DiscOSConfig":
        if path is None:
            p = Path("discos.yaml")
            if p.exists():
                path = p
            else:
                return cls()
        p = Path(path)
        data = yaml.safe_load(p.read_text(encoding="utf-8")) or {}
        return cls(**data)

    def workspace_path(self) -> Path:
        return Path(self.workspace_dir)

    def objects_path(self) -> Path:
        return self.workspace_path() / self.objects_dir

    def receipts_path(self) -> Path:
        return self.workspace_path() / self.receipts_dir

    def bundles_path(self) -> Path:
        return self.workspace_path() / self.bundles_dir

# Copyright 2026 Joseph Verdicchio
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, Optional
from discos.registry.canonicalize import canonical_json, sha256_hex

@dataclass(frozen=True)
class DataView:
    raw_snapshot_id: str
    preprocess_dag_id: str
    split_policy: str  # e.g. train/val/holdout
    access_policy: str # e.g. rows|oracle_only
    schema_id: Optional[str] = None
    pds_manifest_id: Optional[str] = None

    def to_dict(self) -> Dict[str, Any]:
        return {
            "raw_snapshot_id": self.raw_snapshot_id,
            "preprocess_dag_id": self.preprocess_dag_id,
            "split_policy": self.split_policy,
            "access_policy": self.access_policy,
            "schema_id": self.schema_id,
            "pds_manifest_id": self.pds_manifest_id,
        }

    def id(self) -> str:
        return sha256_hex(canonical_json(self.to_dict()))

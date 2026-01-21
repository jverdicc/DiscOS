from __future__ import annotations

import json
import math
from typing import Any, Dict

def _normalize(obj: Any) -> Any:
    if isinstance(obj, float):
        if math.isnan(obj):
            return "NaN"
        if math.isinf(obj):
            return "Inf" if obj > 0 else "-Inf"
        return format(obj, ".17g")
    if isinstance(obj, list):
        return [_normalize(x) for x in obj]
    if isinstance(obj, dict):
        return {k: _normalize(obj[k]) for k in sorted(obj.keys())}
    return obj

def canonical_json(obj: Dict[str, Any]) -> str:
    normalized = _normalize(obj)
    return json.dumps(normalized, sort_keys=True, separators=(",", ":"), ensure_ascii=False)

def sha256_hex(text: str) -> str:
    import hashlib
    return hashlib.sha256(text.encode("utf-8")).hexdigest()

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict

@dataclass(frozen=True)
class UvpError(Exception):
    code: str
    message: str
    details: Dict[str, Any] | None = None

    def to_dict(self) -> Dict[str, Any]:
        return {"code": self.code, "message": self.message, "details": self.details or {}}

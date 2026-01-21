from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, Mapping, Tuple

_BASE_ORDER = ("L", "M", "T", "I", "Theta", "N", "J")  # SI basis (extensible)

@dataclass(frozen=True)
class PDS:
    """Physical Dimension Signature.

    Represented as sparse exponent map {base -> exponent}.
    Supports custom bases like 'USD' by allowing any base symbol.
    """
    exponents: Mapping[str, int]

    @staticmethod
    def dimensionless() -> "PDS":
        return PDS({})

    @staticmethod
    def parse(text: str) -> "PDS":
        t = text.strip()
        if t in ("", "1", "dimensionless"):
            return PDS.dimensionless()

        # Allow simple unit tags like "USD" or "m" as custom base with exponent 1
        # Production should have a full unit registry; MVP treats unknown units as base dims.
        if all(ch.isalnum() or ch in ("_", "-", "/") for ch in t) and ("^" not in t) and ("*" not in t) and (" " not in t):
            return PDS({t: 1})

        # Parse forms like "L^1*T^-2*M^1" (separators: '*' or whitespace)
        parts = []
        for sep in ("*", " "):
            if sep in t:
                parts = [p for p in t.replace("*", " ").split(" ") if p]
                break
        if not parts:
            parts = [t]

        exps: Dict[str, int] = {}
        for p in parts:
            if "^" in p:
                base, pow_ = p.split("^", 1)
                base = base.strip()
                powi = int(pow_.strip())
            else:
                base = p.strip()
                powi = 1
            exps[base] = exps.get(base, 0) + powi

        # Remove zeros
        exps = {k: v for k, v in exps.items() if v != 0}
        return PDS(exps)

    def __mul__(self, other: "PDS") -> "PDS":
        exps = dict(self.exponents)
        for k, v in other.exponents.items():
            exps[k] = exps.get(k, 0) + v
            if exps[k] == 0:
                del exps[k]
        return PDS(exps)

    def __truediv__(self, other: "PDS") -> "PDS":
        exps = dict(self.exponents)
        for k, v in other.exponents.items():
            exps[k] = exps.get(k, 0) - v
            if exps[k] == 0:
                del exps[k]
        return PDS(exps)

    def same_as(self, other: "PDS") -> bool:
        return dict(self.exponents) == dict(other.exponents)

    def to_canonical_str(self) -> str:
        # Deterministic ordering: SI bases first, then others
        keys = list(self.exponents.keys())
        si = [k for k in _BASE_ORDER if k in self.exponents]
        rest = sorted([k for k in keys if k not in _BASE_ORDER])
        ordered = si + rest
        if not ordered:
            return "1"
        parts = []
        for k in ordered:
            v = self.exponents[k]
            parts.append(f"{k}^{v}")
        return "*".join(parts)

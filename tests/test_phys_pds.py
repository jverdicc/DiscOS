from __future__ import annotations

from discos.hir.phys import PDS

def test_pds_parse_dimensionless() -> None:
    assert PDS.parse("1").to_canonical_str() == "1"
    assert PDS.parse("dimensionless").to_canonical_str() == "1"

def test_pds_mul_div() -> None:
    v = PDS.parse("L^1") / PDS.parse("T^1")
    assert v.to_canonical_str() in ("L^1*T^-1", "L^1*T^-1")  # deterministic ordering
    e = PDS.parse("M^1") * PDS.parse("L^2") / PDS.parse("T^2")
    assert e.to_canonical_str() == "L^2*M^1*T^-2" or e.to_canonical_str() == "M^1*L^2*T^-2"

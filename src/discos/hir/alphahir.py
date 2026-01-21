from __future__ import annotations

from typing import Any, Dict, List, Literal, Optional
from pydantic import BaseModel, Field, ConfigDict

NodeKind = Literal["input", "const", "op"]

class AlphaHIRInputSpec(BaseModel):
    model_config = ConfigDict(extra="forbid")
    dtype: Literal["f64"] = "f64"
    pds: str

class AlphaHIRNode(BaseModel):
    model_config = ConfigDict(extra="forbid")
    id: str
    kind: NodeKind
    name: Optional[str] = None
    value: Optional[float] = None
    op: Optional[str] = None
    args: Optional[List[str]] = None

class AlphaHIR(BaseModel):
    model_config = ConfigDict(extra="forbid")
    version: str = "0.1.0"
    inputs: Dict[str, AlphaHIRInputSpec]
    nodes: List[AlphaHIRNode]
    output_node: str
    declared_output_pds: str
    metadata: Dict[str, Any] = Field(default_factory=dict)

    def to_canonical_dict(self) -> Dict[str, Any]:
        return self.model_dump(mode="json", by_alias=True, exclude_none=True)

def alphahir_template_simple_return(name: str = "simple_return") -> AlphaHIR:
    """Template: (close - open) / open."""
    return AlphaHIR(
        inputs={
            "open": AlphaHIRInputSpec(dtype="f64", pds="USD"),
            "close": AlphaHIRInputSpec(dtype="f64", pds="USD"),
        },
        nodes=[
            AlphaHIRNode(id="n_open", kind="input", name="open"),
            AlphaHIRNode(id="n_close", kind="input", name="close"),
            AlphaHIRNode(id="n_num", kind="op", op="sub", args=["n_close", "n_open"]),
            AlphaHIRNode(id="n_out", kind="op", op="safe_div", args=["n_num", "n_open"]),
        ],
        output_node="n_out",
        declared_output_pds="1",
        metadata={"name": name},
    )

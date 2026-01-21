from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, Optional, Protocol

from discos.uvp.errors import UvpError

@dataclass(frozen=True)
class UvpReceipt:
    receipt_id: str
    payload: Dict[str, Any]
    signature: str

class EvidenceKernelClient(Protocol):
    def uvp_announce(self, pds_manifest: Dict[str, Any], causal_dag: Optional[Dict[str, Any]], policy_facet: Optional[Dict[str, Any]]) -> str:
        ...
    def uvp_propose(self, context_id: str, hir_blob: Dict[str, Any], hid_struct: str) -> str:
        ...
    def uvp_budget_request(self, family_id: str, lane: str, amount: float) -> str:
        ...
    def uvp_vault_run(self, claim_id: str, wealth_token: str, data_view_id: str, lane: str) -> UvpReceipt:
        ...
    def uvp_certify(self, receipt_id: str, publish_policy: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        ...

class InMemoryKernelStub:
    """Non-secure stub kernel used for local dev only."""
    def uvp_announce(self, pds_manifest: Dict[str, Any], causal_dag: Optional[Dict[str, Any]], policy_facet: Optional[Dict[str, Any]]) -> str:
        return "ctx_stub"

    def uvp_propose(self, context_id: str, hir_blob: Dict[str, Any], hid_struct: str) -> str:
        if context_id != "ctx_stub":
            raise UvpError("E_CONTEXT", "unknown context", {"context_id": context_id})
        return f"claim_{hid_struct[:12]}"

    def uvp_budget_request(self, family_id: str, lane: str, amount: float) -> str:
        return f"wealth_{family_id[:8]}_{lane}"

    def uvp_vault_run(self, claim_id: str, wealth_token: str, data_view_id: str, lane: str) -> UvpReceipt:
        return UvpReceipt(receipt_id=f"rcpt_{claim_id}_{lane}", payload={
            "claim_id": claim_id,
            "wealth_token": wealth_token,
            "data_view_id": data_view_id,
            "lane": lane,
            "status": "OK",
        }, signature="stub-signature")

    def uvp_certify(self, receipt_id: str, publish_policy: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        return {"capsule_sig": f"sig_{receipt_id}", "etl_inclusion_proof": None}

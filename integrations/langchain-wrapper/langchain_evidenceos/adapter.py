from __future__ import annotations

import importlib.util
from dataclasses import dataclass
from typing import Any, Callable, Mapping

from .guard import EvidenceOSGuardCallbackHandler, PolicyReceipt

if importlib.util.find_spec("langchain_core"):
    from langchain_core.runnables import RunnableLambda
else:
    RunnableLambda = None  # type: ignore[assignment]


@dataclass(frozen=True)
class ToolExecutionResult:
    output: Any
    policy_receipt: PolicyReceipt


class EvidenceOSRunnableAdapter:
    """Runnable-compatible wrapper that applies EvidenceOS preflight to a Python callable."""

    def __init__(
        self,
        *,
        tool_name: str,
        tool_func: Callable[[dict[str, Any]], Any],
        guard: EvidenceOSGuardCallbackHandler,
    ) -> None:
        self.tool_name = tool_name
        self.tool_func = tool_func
        self.guard = guard

    def invoke(self, tool_input: Mapping[str, Any] | str) -> ToolExecutionResult:
        preflight = self.guard.preflight_tool_call(tool_name=self.tool_name, tool_input=tool_input)
        output = self.tool_func(preflight.params)
        return ToolExecutionResult(output=output, policy_receipt=preflight.receipt)

    def as_langchain_runnable(self) -> Any:
        if RunnableLambda is None:
            raise RuntimeError("langchain-core is required for Runnable integration")
        return RunnableLambda(self.invoke)

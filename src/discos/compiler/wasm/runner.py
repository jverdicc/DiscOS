from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Dict, List, Tuple
import time

import numpy as np

from discos.registry.canonicalize import sha256_hex

@dataclass(frozen=True)
class CanaryReport:
    hid_behav: str
    n: int
    mean: float
    std: float
    nan_rate: float
    inf_rate: float
    runtime_ms: float
    engine: str
    notes: List[str]

    def to_dict(self) -> Dict[str, Any]:
        return {
            "hid_behav": self.hid_behav,
            "n": self.n,
            "mean": self.mean,
            "std": self.std,
            "nan_rate": self.nan_rate,
            "inf_rate": self.inf_rate,
            "runtime_ms": self.runtime_ms,
            "engine": self.engine,
            "notes": self.notes,
        }

def _sketch_hash(series: np.ndarray) -> str:
    # Very cheap sketch: quantiles + sign-bits
    x = series[np.isfinite(series)]
    if x.size == 0:
        return sha256_hex("empty")
    qs = np.quantile(x, [0.0, 0.1, 0.5, 0.9, 1.0])
    qstr = ",".join([f"{float(v):.6g}" for v in qs])
    sign = (series[: min(256, series.size)] > 0).astype(np.uint8)
    return sha256_hex(qstr + "|" + bytes(sign.tolist()).hex())

def run_canary(
    wat: str,
    *,
    inputs: Dict[str, np.ndarray],
    input_order: List[str],
    use_wasmtime: bool = True,
) -> Tuple[np.ndarray, CanaryReport]:
    if not input_order:
        raise ValueError("input_order must include at least one input name")
    missing = [name for name in input_order if name not in inputs]
    if missing:
        raise ValueError(f"inputs missing required keys: {missing}")
    lengths = [len(inputs[name]) for name in input_order]
    if len(set(lengths)) != 1:
        raise ValueError(f"inputs have mismatched lengths: {dict(zip(input_order, lengths))}")
    n = int(min(lengths[0], 512))
    out = np.empty(n, dtype=np.float64)
    notes: List[str] = []

    start = time.time()
    engine = "python-fallback"

    if use_wasmtime:
        try:
            import wasmtime  # type: ignore
        except Exception as e:
            notes.append(f"wasmtime not available: {e}; using python fallback")
        else:
            engine = f"wasmtime-{getattr(wasmtime, '__version__', 'unknown')}"
            store = wasmtime.Store()
            module = wasmtime.Module(store.engine, wasmtime.wat2wasm(wat))
            linker = wasmtime.Linker(store.engine)
            instance = linker.instantiate(store, module)
            memory = instance.exports(store)["memory"]

            offset = 0
            ptrs: Dict[str, int] = {}
            for name in input_order:
                arr = inputs[name][:n].astype(np.float64)
                b = arr.tobytes(order="C")
                ptrs[name] = offset
                memory.write(store, b, offset)
                offset += len(b)

            out_ptr = offset
            memory.write(store, out.tobytes(order="C"), out_ptr)

            func = instance.exports(store)["eval_series"]
            args = [ptrs[name] for name in input_order] + [out_ptr, n]
            func(store, *args)  # type: ignore

            out_bytes = memory.read(store, out_ptr, out_ptr + n * 8)
            out = np.frombuffer(out_bytes, dtype=np.float64).copy()

            runtime_ms = (time.time() - start) * 1000.0
            nan_rate = float(np.mean(np.isnan(out)))
            inf_rate = float(np.mean(np.isinf(out)))
            finite = out[np.isfinite(out)]
            mean = float(np.mean(finite)) if finite.size else 0.0
            std = float(np.std(finite)) if finite.size else 0.0
            hid_behav = _sketch_hash(out)

            return out, CanaryReport(
                hid_behav=hid_behav,
                n=n,
                mean=mean,
                std=std,
                nan_rate=nan_rate,
                inf_rate=inf_rate,
                runtime_ms=runtime_ms,
                engine=engine,
                notes=notes,
            )

    # Python fallback demo: assumes simple_return shape (open, close)
    open_ = inputs[input_order[0]][:n].astype(np.float64)
    close_ = inputs[input_order[1]][:n].astype(np.float64)
    out = (close_ - open_) / np.where(np.abs(open_) < 1e-12, np.nan, open_)
    runtime_ms = (time.time() - start) * 1000.0

    nan_rate = float(np.mean(np.isnan(out)))
    inf_rate = float(np.mean(np.isinf(out)))
    finite = out[np.isfinite(out)]
    mean = float(np.mean(finite)) if finite.size else 0.0
    std = float(np.std(finite)) if finite.size else 0.0
    hid_behav = _sketch_hash(out)

    notes.append("python fallback CANARY is not sandboxed; for demo only")
    return out, CanaryReport(
        hid_behav=hid_behav,
        n=n,
        mean=mean,
        std=std,
        nan_rate=nan_rate,
        inf_rate=inf_rate,
        runtime_ms=runtime_ms,
        engine=engine,
        notes=notes,
    )

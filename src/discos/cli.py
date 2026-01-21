from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any, Dict, Optional

from discos.config import DiscOSConfig
from discos.hir.alphahir import alphahir_template_simple_return, AlphaHIR
from discos.registry.workspace import Workspace
from discos.admissibility.lint import lint_alphahir, LintError
from discos.compiler.wasm.watgen import generate_wat_for_alphahir
from discos.compiler.wasm.runner import run_canary
from discos.artifact.bundle import build_pcdb_bundle


def _read_json(path: Path) -> Dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def cmd_init(args: argparse.Namespace) -> int:
    cfg = DiscOSConfig.load(args.config)
    ws = Workspace(cfg)
    ws.init()
    print(f"Initialized workspace at {ws.root}")
    return 0


def cmd_alphahir_new(args: argparse.Namespace) -> int:
    hir = alphahir_template_simple_return(name=args.name)
    print(json.dumps(hir.to_canonical_dict(), indent=2))
    return 0


def cmd_lint(args: argparse.Namespace) -> int:
    cfg = DiscOSConfig.load(args.config)
    ws = Workspace(cfg)
    hir = _read_json(Path(args.hir))
    try:
        report = lint_alphahir(hir, phys_lint=cfg.phys_lint)
    except LintError as e:
        print(json.dumps({"ok": False, "error": e.to_dict()}, indent=2))
        return 2
    print(json.dumps(report, indent=2))
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    cfg = DiscOSConfig.load(args.config)
    ws = Workspace(cfg)
    ws.init()

    hir_path = Path(args.hir)
    hir = _read_json(hir_path)
    hid = ws.store_hypothesis(hir, family_id=args.family)

    lane = args.lane.upper()
    if lane == "FAST":
        # FAST lane = compute behavior sketch via python evaluator on small synthetic data
        # For MVP, reuse CANARY code path with python fallback if wasm unavailable.
        pass

    if lane == "CANARY":
        report = lint_alphahir(hir, phys_lint=cfg.phys_lint)
        if not report["ok"]:
            print(json.dumps(report, indent=2))
            return 2

        wat = generate_wat_for_alphahir(hir, input_order=["open", "close"]).wat
        # synthetic data
        import numpy as np
        rng = np.random.default_rng(0)
        open_ = 100 + rng.normal(0, 1, size=2048).cumsum()
        close_ = open_ * (1 + rng.normal(0, 0.01, size=2048))
        outputs, canary = run_canary(wat, inputs={"open": open_, "close": close_}, input_order=["open", "close"], use_wasmtime=cfg.prefer_wasm_canary)
        receipt = ws.write_receipt(hid, lane="CANARY", payload=canary.to_dict())
        print(json.dumps({"hid_struct": hid, "receipt": str(receipt), "canary": canary.to_dict()}, indent=2))
        return 0

    if lane == "HEAVY":
        # MVP: local heavy run = evaluate on larger synthetic dataset and save stats.
        import numpy as np
        rng = np.random.default_rng(1)
        open_ = 100 + rng.normal(0, 1, size=20000).cumsum()
        close_ = open_ * (1 + rng.normal(0, 0.01, size=20000))
        # python eval of simple_return
        out = (close_ - open_) / np.where(np.abs(open_) < 1e-12, np.nan, open_)
        finite = out[np.isfinite(out)]
        payload = {
            "n": int(out.size),
            "mean": float(np.mean(finite)) if finite.size else 0.0,
            "std": float(np.std(finite)) if finite.size else 0.0,
            "nan_rate": float(np.mean(np.isnan(out))),
            "inf_rate": float(np.mean(np.isinf(out))),
        }
        receipt = ws.write_receipt(hid, lane="HEAVY", payload=payload)
        print(json.dumps({"hid_struct": hid, "receipt": str(receipt), "heavy": payload}, indent=2))
        return 0

    if lane == "SEALED":
        print("SEALED lane requires an EvidenceOS kernel. Configure uvp client in discos.yaml (not implemented in MVP).")
        return 3

    print(f"Unknown lane: {args.lane}")
    return 2


def cmd_bundle(args: argparse.Namespace) -> int:
    cfg = DiscOSConfig.load(args.config)
    ws = Workspace(cfg)
    ws.init()
    hir = _read_json(Path(args.hir))
    hid = ws.store_hypothesis(hir, family_id=args.family)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    bundle_path = build_pcdb_bundle(ws, hid, out)
    print(f"Wrote PCDB bundle: {bundle_path}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="discos", description="DiscOS userland CLI")
    p.add_argument("--config", default=None, help="Path to discos.yaml (optional)")

    sub = p.add_subparsers(dest="cmd", required=True)

    sp = sub.add_parser("init", help="Initialize .discos workspace")
    sp.set_defaults(fn=cmd_init)

    sp = sub.add_parser("lint", help="Lint a HIR JSON file")
    sp.add_argument("hir")
    sp.set_defaults(fn=cmd_lint)

    sp = sub.add_parser("run", help="Run a HIR in a given lane (FAST/CANARY/HEAVY/SEALED)")
    sp.add_argument("hir")
    sp.add_argument("--lane", default="CANARY")
    sp.add_argument("--family", default="fam_default")
    sp.set_defaults(fn=cmd_run)

    sp = sub.add_parser("bundle", help="Build a Proof-Carrying Discovery Bundle (PCDB)")
    sp.add_argument("hir")
    sp.add_argument("--out", default="bundle.zip")
    sp.add_argument("--family", default="fam_default")
    sp.set_defaults(fn=cmd_bundle)

    alpha = sub.add_parser("alphahir", help="AlphaHIR helpers")
    alpha_sub = alpha.add_subparsers(dest="alpha_cmd", required=True)
    alpha_new = alpha_sub.add_parser("new", help="Generate a template AlphaHIR")
    alpha_new.add_argument("--name", default="simple_return")
    alpha_new.set_defaults(fn=cmd_alphahir_new)

    return p


def main(argv: Optional[list[str]] = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return int(args.fn(args))

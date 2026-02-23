#!/usr/bin/env python3
"""DiscOS wrapper to the authoritative EvidenceOS FORC10 artifact runner.

This script intentionally does not generate synthetic outputs.
It shells out to EvidenceOS's `artifacts/forc10/original_python/run_all.py`.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--evidenceos-repo",
        default=os.environ.get("EVIDENCEOS_REPO", "../EvidenceOS"),
        help="Path to an EvidenceOS checkout containing artifacts/forc10/original_python/run_all.py",
    )
    parser.add_argument(
        "runner_args",
        nargs=argparse.REMAINDER,
        help="Arguments passed through to EvidenceOS artifact runner (prefix with --).",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    evidenceos_repo = Path(args.evidenceos_repo).resolve()
    runner = evidenceos_repo / "artifacts" / "forc10" / "original_python" / "run_all.py"

    if not runner.exists():
        print(
            "ERROR: authoritative paper reproduction runner not found. "
            f"Expected {runner}. Clone EvidenceOS and set --evidenceos-repo or EVIDENCEOS_REPO.",
            file=sys.stderr,
        )
        return 2

    passthrough = list(args.runner_args)
    if passthrough and passthrough[0] == "--":
        passthrough = passthrough[1:]

    cmd = ["python3", runner.as_posix(), *passthrough]
    return subprocess.call(cmd)


if __name__ == "__main__":
    raise SystemExit(main())

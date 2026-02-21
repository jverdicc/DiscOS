#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import math
from pathlib import Path


def is_number(value: str) -> bool:
    try:
        float(value)
    except ValueError:
        return False
    return True


def compare_scalars(actual, expected, atol: float) -> bool:
    if isinstance(actual, (int, float)) and isinstance(expected, (int, float)):
        return math.isclose(float(actual), float(expected), abs_tol=atol, rel_tol=0.0)
    return actual == expected


def compare_json(actual_path: Path, expected_path: Path, atol: float) -> list[str]:
    actual = json.loads(actual_path.read_text(encoding="utf-8"))
    expected = json.loads(expected_path.read_text(encoding="utf-8"))
    errors: list[str] = []

    def walk(a, e, prefix: str) -> None:
        if isinstance(a, dict) and isinstance(e, dict):
            if set(a) != set(e):
                errors.append(f"{prefix}: key mismatch {sorted(a)} != {sorted(e)}")
                return
            for key in sorted(a):
                walk(a[key], e[key], f"{prefix}.{key}")
            return
        if isinstance(a, list) and isinstance(e, list):
            if len(a) != len(e):
                errors.append(f"{prefix}: length mismatch {len(a)} != {len(e)}")
                return
            for idx, (av, ev) in enumerate(zip(a, e)):
                walk(av, ev, f"{prefix}[{idx}]")
            return
        if not compare_scalars(a, e, atol):
            errors.append(f"{prefix}: {a!r} != {e!r}")

    walk(actual, expected, actual_path.name)
    return errors


def compare_csv(actual_path: Path, expected_path: Path, atol: float) -> list[str]:
    errors: list[str] = []
    with actual_path.open("r", encoding="utf-8") as a_handle, expected_path.open("r", encoding="utf-8") as e_handle:
        a_rows = list(csv.reader(a_handle))
        e_rows = list(csv.reader(e_handle))

    if len(a_rows) != len(e_rows):
        return [f"{actual_path.name}: row count {len(a_rows)} != {len(e_rows)}"]

    for row_idx, (a_row, e_row) in enumerate(zip(a_rows, e_rows)):
        if len(a_row) != len(e_row):
            errors.append(f"{actual_path.name}[{row_idx}]: column count {len(a_row)} != {len(e_row)}")
            continue
        for col_idx, (av, ev) in enumerate(zip(a_row, e_row)):
            if is_number(av) and is_number(ev):
                if not math.isclose(float(av), float(ev), abs_tol=atol, rel_tol=0.0):
                    errors.append(f"{actual_path.name}[{row_idx},{col_idx}]: {av} != {ev}")
            elif av != ev:
                errors.append(f"{actual_path.name}[{row_idx},{col_idx}]: {av!r} != {ev!r}")
    return errors


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--actual", default="artifacts/forc10/generated")
    parser.add_argument("--golden", default="artifacts/forc10/golden")
    parser.add_argument("--atol", type=float, default=1e-12)
    args = parser.parse_args()

    actual = Path(args.actual)
    golden = Path(args.golden)

    errors: list[str] = []
    for expected_path in sorted(golden.rglob("*")):
        if expected_path.is_dir():
            continue
        rel = expected_path.relative_to(golden)
        actual_path = actual / rel
        if not actual_path.exists():
            errors.append(f"missing output: {rel.as_posix()}")
            continue
        if expected_path.suffix == ".json":
            errors.extend(compare_json(actual_path, expected_path, args.atol))
        elif expected_path.suffix == ".csv":
            errors.extend(compare_csv(actual_path, expected_path, args.atol))
        else:
            if actual_path.read_bytes() != expected_path.read_bytes():
                errors.append(f"byte mismatch: {rel.as_posix()}")

    if errors:
        print("verification failed")
        for error in errors:
            print(f" - {error}")
        raise SystemExit(1)

    print(json.dumps({"ok": True, "verified": str(actual), "golden": str(golden)}))


if __name__ == "__main__":
    main()

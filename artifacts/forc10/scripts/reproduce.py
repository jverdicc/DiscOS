#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import subprocess
from pathlib import Path


def run(cmd: list[str]) -> None:
    subprocess.run(cmd, check=True)


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def write_csv(path: Path, rows: list[dict], fieldnames: list[str]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default="artifacts/forc10/generated")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parents[3]
    out_dir = Path(args.out)
    raw_dir = out_dir / "raw"
    run(["python3", (repo_root / "paper_artifacts" / "reproduce_paper.py").as_posix(), "--", "--out", raw_dir.as_posix()])

    index = load_json(raw_dir / "index.json")
    exp_rows: list[dict] = []
    summary_rows: list[dict] = []

    for exp_id, meta in sorted(index["experiments"].items(), key=lambda item: int(item[0])):
        exp_file = raw_dir / meta["artifact"]
        payload = load_json(exp_file)
        rows = payload.get("rows", [])
        exp_rows.append(
            {
                "experiment": int(exp_id),
                "seed": meta["seed"],
                "schema_version": payload["schema_version"],
                "row_count": len(rows),
            }
        )
        summary_rows.append(
            {
                "experiment": int(exp_id),
                "min_value": min((row.get("value", 0.0) for row in rows), default=0.0),
                "max_value": max((row.get("value", 0.0) for row in rows), default=0.0),
            }
        )

    write_csv(out_dir / "results" / "experiments.csv", exp_rows, ["experiment", "seed", "schema_version", "row_count"])
    write_csv(out_dir / "results" / "summary.csv", summary_rows, ["experiment", "min_value", "max_value"])

    write_json(
        out_dir / "results" / "index.json",
        {
            "schema_version": "discos.forc10.legacy.index.v1",
            "source": "legacy-discos-wrapper",
            "outputs": [
                "raw/index.json",
                "results/experiments.csv",
                "results/summary.csv",
            ],
            "non_reproducible": ["authoritative paper reproduction delegated to EvidenceOS"],
        },
    )


if __name__ == "__main__":
    main()

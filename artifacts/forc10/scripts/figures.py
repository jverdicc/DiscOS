#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default="artifacts/forc10/generated")
    args = parser.parse_args()

    out_dir = Path(args.out)
    raw_dir = out_dir / "raw"
    fig_dir = out_dir / "figures"
    fig_dir.mkdir(parents=True, exist_ok=True)

    exp01 = load_json(raw_dir / "exp01.json")
    with (fig_dir / "figure_exp01_mae.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["queries", "mae_no_hysteresis", "mae_with_hysteresis"])
        for row in exp01["rows"]:
            writer.writerow([row["queries"], row["mae_no_hysteresis"], row["mae_with_hysteresis"]])

    exp11 = load_json(raw_dir / "exp11.json")
    with (fig_dir / "table_exp11_success.csv").open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["n_identities", "naive_success_prob", "topichash_success_prob"])
        for row in exp11["rows"]:
            writer.writerow([row["n_identities"], row["naive_success_prob"], row["topichash_success_prob"]])


if __name__ == "__main__":
    main()

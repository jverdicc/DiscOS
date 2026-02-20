#!/usr/bin/env python3
"""Naive oracle that leaks labels through raw accuracy responses."""

from __future__ import annotations

import argparse
import random
from dataclasses import dataclass
from typing import List


@dataclass
class BaselineOracle:
    labels: List[int]

    @classmethod
    def from_seed(cls, n: int, seed: int) -> "BaselineOracle":
        rng = random.Random(seed)
        labels = [rng.randint(0, 1) for _ in range(n)]
        return cls(labels=labels)

    def score(self, prediction: List[int]) -> float:
        if len(prediction) != len(self.labels):
            raise ValueError(
                f"prediction length {len(prediction)} != label length {len(self.labels)}"
            )
        matches = sum(int(p == y) for p, y in zip(prediction, self.labels))
        return matches / len(self.labels)


def parse_prediction(bits: str, n: int) -> List[int]:
    if len(bits) != n:
        raise ValueError(f"expected {n} bits, got {len(bits)}")
    if any(ch not in {"0", "1"} for ch in bits):
        raise ValueError("prediction must be a binary string (0/1)")
    return [int(ch) for ch in bits]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--n", type=int, default=64, help="label vector length")
    parser.add_argument("--seed", type=int, default=7, help="deterministic seed")
    parser.add_argument(
        "--prediction",
        required=True,
        help="binary prediction vector (for example 010101)",
    )
    args = parser.parse_args()

    oracle = BaselineOracle.from_seed(n=args.n, seed=args.seed)
    prediction = parse_prediction(args.prediction, args.n)
    print(f"{oracle.score(prediction):.6f}")


if __name__ == "__main__":
    main()

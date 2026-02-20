#!/usr/bin/env python3
"""Bit-flip exfiltration demo: baseline oracle vs EvidenceOS-style controls."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from typing import Dict, List

from baseline_oracle import BaselineOracle


@dataclass
class EvidenceOSMockOracle:
    """Protocol-faithful mock of mediated responses.

    Controls applied:
    - Quantization of score outputs.
    - Hysteresis threshold to suppress small score movement.
    - Transcript budget limiting query count.
    - Capsule-like receipts for each query.
    """

    labels: List[int]
    quantization_step: float
    hysteresis_threshold: float
    budget: int

    def __post_init__(self) -> None:
        self.query_count = 0
        self._last_emitted = None

    @classmethod
    def from_seed(
        cls,
        n: int,
        seed: int,
        quantization_step: float,
        hysteresis_threshold: float,
        budget: int,
    ) -> "EvidenceOSMockOracle":
        base = BaselineOracle.from_seed(n=n, seed=seed)
        return cls(
            labels=base.labels,
            quantization_step=quantization_step,
            hysteresis_threshold=hysteresis_threshold,
            budget=budget,
        )

    def _quantize(self, value: float) -> float:
        bins = round(value / self.quantization_step)
        return bins * self.quantization_step

    def score(self, prediction: List[int]) -> Dict[str, object]:
        if self.query_count >= self.budget:
            return {
                "score": self._last_emitted if self._last_emitted is not None else 0.0,
                "receipt": {
                    "type": "capsule",
                    "query_count": self.query_count,
                    "budget": self.budget,
                    "budget_exhausted": True,
                    "quantization_step": self.quantization_step,
                    "hysteresis_threshold": self.hysteresis_threshold,
                },
            }

        raw = BaselineOracle(self.labels).score(prediction)
        quantized = self._quantize(raw)

        if self._last_emitted is None:
            emitted = quantized
        elif abs(quantized - self._last_emitted) < self.hysteresis_threshold:
            emitted = self._last_emitted
        else:
            emitted = quantized

        self._last_emitted = emitted
        self.query_count += 1

        return {
            "score": emitted,
            "receipt": {
                "type": "capsule",
                "query_count": self.query_count,
                "budget": self.budget,
                "budget_exhausted": self.query_count >= self.budget,
                "quantization_step": self.quantization_step,
                "hysteresis_threshold": self.hysteresis_threshold,
            },
        }


def bitflip_attack_baseline(oracle: BaselineOracle) -> Dict[str, object]:
    n = len(oracle.labels)
    guess = [0] * n
    queries = 1
    baseline = oracle.score(guess)

    for i in range(n):
        guess[i] = 1
        updated = oracle.score(guess)
        queries += 1
        if updated > baseline:
            baseline = updated
        else:
            guess[i] = 0

    recovered_accuracy = sum(int(a == b) for a, b in zip(guess, oracle.labels)) / n
    return {
        "mode": "baseline",
        "n": n,
        "queries": queries,
        "recovered_accuracy": recovered_accuracy,
    }


def bitflip_attack_evidenceos(oracle: EvidenceOSMockOracle) -> Dict[str, object]:
    n = len(oracle.labels)
    guess = [0] * n

    initial = oracle.score(guess)
    observed = float(initial["score"])
    queries = 1
    last_receipt = initial["receipt"]

    for i in range(n):
        guess[i] = 1
        response = oracle.score(guess)
        score = float(response["score"])
        last_receipt = response["receipt"]
        queries += 1

        if response["receipt"]["budget_exhausted"]:
            guess[i] = 0
            break

        if score > observed:
            observed = score
        else:
            guess[i] = 0

    recovered_accuracy = sum(int(a == b) for a, b in zip(guess, oracle.labels)) / n
    return {
        "mode": "evidenceos-mock",
        "n": n,
        "queries": queries,
        "recovered_accuracy": recovered_accuracy,
        "last_receipt": last_receipt,
    }


def print_result(result: Dict[str, object], fmt: str) -> None:
    if fmt == "json":
        print(json.dumps(result, sort_keys=True))
        return

    print(f"mode: {result['mode']}")
    print(f"recovered-label accuracy: {result['recovered_accuracy']:.4f}")
    print(f"number of queries: {result['queries']}")
    if "last_receipt" in result:
        print(f"capsule-like receipt: {json.dumps(result['last_receipt'], sort_keys=True)}")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=["baseline", "evidenceos-mock"], default="baseline")
    parser.add_argument("--n", type=int, default=64)
    parser.add_argument("--seed", type=int, default=7)
    parser.add_argument("--format", choices=["text", "json"], default="text")
    parser.add_argument("--quant-step", type=float, default=0.05)
    parser.add_argument("--hysteresis", type=float, default=0.03)
    parser.add_argument("--budget", type=int, default=48)
    args = parser.parse_args()

    if args.mode == "baseline":
        result = bitflip_attack_baseline(BaselineOracle.from_seed(n=args.n, seed=args.seed))
    else:
        result = bitflip_attack_evidenceos(
            EvidenceOSMockOracle.from_seed(
                n=args.n,
                seed=args.seed,
                quantization_step=args.quant_step,
                hysteresis_threshold=args.hysteresis,
                budget=args.budget,
            )
        )

    print_result(result, args.format)


if __name__ == "__main__":
    main()

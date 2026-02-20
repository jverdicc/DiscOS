# EvidenceOS-mediated exfiltration variant

This demo keeps the same prediction-vector interface as the baseline oracle, but applies
EvidenceOS-style response controls before returning any score.

## Control surface

1. **Quantized output**
   - Raw accuracy is rounded to fixed increments (`quantization_step`, default `0.05`).
   - Single-bit flips at `N=64` move raw accuracy by `1/N = 0.015625`, which is below one quantization step.

2. **Hysteresis threshold**
   - Even after quantization, a new score is emitted only when movement from the last emitted score
     is at least `hysteresis_threshold` (default `0.03`).
   - Small oscillations are collapsed to the previous emitted value.

3. **Transcript/query budget**
   - Each score request consumes budget (`budget`, default `48`).
   - Once exhausted, no additional information is revealed beyond a budget-exhausted receipt.

4. **Capsule-like receipt**
   - Every response includes metadata with query count, budget, and control parameters.

## Why bit-flip exfiltration breaks

The classic attack depends on observing precise `±1/N` score deltas for each index.
Quantization + hysteresis remove these tiny deltas from the externally visible channel, and the
transcript budget prevents retry-heavy probing strategies.

Result: the attack becomes unproductive and recovered label accuracy stays near chance.

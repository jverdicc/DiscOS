// Copyright 2026 Joseph Verdicchio
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Returns the leakage charge `k_i = log2(|Y_i|)` for one oracle interaction.
///
/// `alphabet_size` must be finite and at least 1 (an empty output alphabet is invalid).
pub fn leakage_bits_for_alphabet(alphabet_size: usize) -> Result<f64, String> {
    if alphabet_size == 0 {
        return Err("alphabet_size must be >= 1".to_string());
    }
    Ok((alphabet_size as f64).log2())
}

/// Sums per-interaction leakage charges and optional non-negative joint-tax terms.
pub fn total_transcript_leakage(
    interaction_bits: &[f64],
    joint_tax_bits: &[f64],
) -> Result<f64, String> {
    let mut total = 0.0;
    for bits in interaction_bits.iter().chain(joint_tax_bits.iter()) {
        if !bits.is_finite() || *bits < 0.0 {
            return Err("all leakage components must be finite and non-negative".to_string());
        }
        total += bits;
    }
    Ok(total)
}

/// Computes the Theorem-1-style adjusted alpha: `alpha' = alpha * 2^(-k_tot)`.
pub fn adjusted_alpha(alpha: f64, k_tot_bits: f64) -> Result<f64, String> {
    if !alpha.is_finite() || alpha < 0.0 || alpha > 1.0 {
        return Err("alpha must be finite and in [0, 1]".to_string());
    }
    if !k_tot_bits.is_finite() || k_tot_bits < 0.0 {
        return Err("k_tot_bits must be finite and non-negative".to_string());
    }
    Ok(alpha * 2f64.powf(-k_tot_bits))
}

/// Maintains a monotone high-water mark `Wmax = max(Wmax, next)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighWaterMark {
    wmax: f64,
}

impl HighWaterMark {
    pub fn new(initial: f64) -> Result<Self, String> {
        if !initial.is_finite() {
            return Err("initial high-water value must be finite".to_string());
        }
        Ok(Self { wmax: initial })
    }

    pub fn observe(&mut self, next: f64) -> Result<f64, String> {
        if !next.is_finite() {
            return Err("observed value must be finite".to_string());
        }
        self.wmax = self.wmax.max(next);
        Ok(self.wmax)
    }

    pub fn value(&self) -> f64 {
        self.wmax
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leakage_charge_is_log2_alphabet_size() {
        assert_eq!(leakage_bits_for_alphabet(1).expect("valid"), 0.0);
        assert_eq!(leakage_bits_for_alphabet(2).expect("valid"), 1.0);
        assert_eq!(leakage_bits_for_alphabet(8).expect("valid"), 3.0);
    }

    #[test]
    fn leakage_rejects_empty_alphabet() {
        assert!(leakage_bits_for_alphabet(0).is_err());
    }

    #[test]
    fn total_transcript_leakage_sums_components_and_joint_tax() {
        let total = total_transcript_leakage(&[1.0, 2.0], &[0.5, 0.25]).expect("valid");
        assert!((total - 3.75).abs() < f64::EPSILON);
    }

    #[test]
    fn adjusted_alpha_applies_theorem_scaling() {
        let alpha_prime = adjusted_alpha(0.01, 3.0).expect("valid");
        assert!((alpha_prime - 0.00125).abs() < 1e-12);
    }

    #[test]
    fn adjusted_alpha_rejects_invalid_inputs() {
        assert!(adjusted_alpha(-0.1, 1.0).is_err());
        assert!(adjusted_alpha(0.1, -1.0).is_err());
        assert!(adjusted_alpha(f64::NAN, 1.0).is_err());
        assert!(adjusted_alpha(0.1, f64::INFINITY).is_err());
    }

    #[test]
    fn high_water_mark_is_monotone() {
        let mut wmax = HighWaterMark::new(0.5).expect("valid");
        assert_eq!(wmax.observe(0.1).expect("valid"), 0.5);
        assert_eq!(wmax.observe(0.6).expect("valid"), 0.6);
        assert_eq!(wmax.observe(0.4).expect("valid"), 0.6);
        assert_eq!(wmax.value(), 0.6);
    }
}

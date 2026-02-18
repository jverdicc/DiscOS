use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LikelihoodRatioE {
    pub null_accuracy: f64,
    pub n_observations: usize,
}

impl LikelihoodRatioE {
    pub fn new(null_accuracy: f64, n_observations: usize) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&null_accuracy) {
            return Err("null_accuracy must be in [0,1]".to_string());
        }
        if n_observations == 0 {
            return Err("n_observations must be > 0".to_string());
        }
        Ok(Self {
            null_accuracy,
            n_observations,
        })
    }

    pub fn compute(&self, observed_accuracy: f64) -> f64 {
        if self.null_accuracy == 0.0 || observed_accuracy == 0.0 {
            return 0.0;
        }
        let ratio = observed_accuracy / self.null_accuracy;
        ratio.powf(self.n_observations as f64).clamp(0.0, f64::MAX)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BettingE {
    pub null_p: f64,
    wealth: f64,
}

impl BettingE {
    pub fn new(null_p: f64) -> Result<Self, String> {
        if !(0.0..1.0).contains(&null_p) {
            return Err("null_p must be in (0,1)".to_string());
        }
        Ok(Self {
            null_p,
            wealth: 1.0,
        })
    }

    pub fn update(&mut self, observation: u8) -> Result<f64, String> {
        let multiplier = match observation {
            0 => 1.0,
            1 => 1.0 / self.null_p,
            _ => return Err("observation must be 0 or 1".to_string()),
        };
        self.wealth = (self.wealth * multiplier).clamp(0.0, f64::MAX);
        Ok(self.wealth)
    }

    pub fn wealth(&self) -> f64 {
        self.wealth
    }

    pub fn reset(&mut self) {
        self.wealth = 1.0;
    }
}

pub fn e_merge_sequential(e_values: &[f64], weights: Option<&[f64]>) -> Result<f64, String> {
    if e_values.is_empty() {
        return Err("e_values must be non-empty".to_string());
    }

    let default_weights;
    let ws = if let Some(w) = weights {
        if w.len() != e_values.len() {
            return Err("weights length mismatch".to_string());
        }
        w
    } else {
        default_weights = vec![1.0; e_values.len()];
        &default_weights
    };

    let mut sum_w = 0.0;
    let mut weighted = 0.0;
    for (e, w) in e_values.iter().zip(ws.iter()) {
        if *w < 0.0 {
            return Err("weights must be >= 0".to_string());
        }
        sum_w += *w;
        weighted += *w * *e;
    }
    if sum_w == 0.0 {
        return Err("sum of weights must be > 0".to_string());
    }
    Ok(weighted / sum_w)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lr_e_at_null_is_one() {
        let lr = LikelihoodRatioE::new(0.5, 10).expect("lr config is valid");
        assert!((lr.compute(0.5) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn betting_e_reset_returns_to_one() {
        let mut b = BettingE::new(0.5).expect("betting config is valid");
        let _ = b.update(1).expect("update succeeds");
        b.reset();
        assert_eq!(b.wealth(), 1.0);
    }
}

#[derive(Debug, Clone)]
pub struct ConservationLedger {
    budget_bits: f64,
    charged_bits: f64,
}

impl ConservationLedger {
    pub fn new(budget_bits: f64) -> Self {
        Self {
            budget_bits,
            charged_bits: 0.0,
        }
    }

    pub fn budget_bits(&self) -> f64 {
        self.budget_bits
    }

    pub fn charged_bits(&self) -> f64 {
        self.charged_bits
    }

    pub fn remaining_bits(&self) -> f64 {
        (self.budget_bits - self.charged_bits).max(0.0)
    }

    pub fn charge(&mut self, bits: f64) -> Result<f64, String> {
        if !(bits.is_finite()) || bits < 0.0 {
            return Err("invalid charge amount".into());
        }
        if self.charged_bits + bits > self.budget_bits {
            return Err("insufficient budget".into());
        }
        self.charged_bits += bits;
        Ok(self.remaining_bits())
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp12Scenario {
    pub n: usize,
    pub psplit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp12Config {
    pub topic_budget_bits: usize,
    pub trials: usize,
    pub seed: u64,
    pub scenarios: Vec<Exp12Scenario>,
}

impl Default for Exp12Config {
    fn default() -> Self {
        Self {
            topic_budget_bits: 2,
            trials: 10_000,
            seed: 42,
            scenarios: vec![
                Exp12Scenario {
                    n: 32,
                    psplit: 0.01,
                },
                Exp12Scenario {
                    n: 64,
                    psplit: 0.01,
                },
                Exp12Scenario {
                    n: 128,
                    psplit: 0.05,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp12Row {
    pub n: usize,
    pub psplit: f64,
    pub mean_leaked_bits: f64,
    pub p99_leaked_bits: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp12Result {
    pub rows: Vec<Exp12Row>,
}

#[derive(Debug, Clone)]
struct Lcg64 {
    state: u64,
}

impl Lcg64 {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        let v = self.next_u64() >> 11;
        (v as f64) * (1.0 / ((1u64 << 53) as f64))
    }
}

fn binomial_sample(n: usize, p: f64, rng: &mut Lcg64) -> usize {
    let mut s = 0usize;
    for _ in 0..n {
        if rng.next_f64() < p {
            s += 1;
        }
    }
    s
}

pub async fn run_exp12(cfg: &Exp12Config) -> anyhow::Result<Exp12Result> {
    if cfg.trials == 0 {
        anyhow::bail!("trials must be greater than zero");
    }

    let mut rows = Vec::with_capacity(cfg.scenarios.len());
    let mut rng = Lcg64::new(cfg.seed);

    for scenario in &cfg.scenarios {
        if !scenario.psplit.is_finite() || !(0.0..=1.0).contains(&scenario.psplit) {
            anyhow::bail!("psplit must be finite and within [0,1]");
        }

        let mut leaked = Vec::with_capacity(cfg.trials);
        for _ in 0..cfg.trials {
            let s = binomial_sample(scenario.n, scenario.psplit, &mut rng);
            leaked.push(cfg.topic_budget_bits + s);
        }

        leaked.sort_unstable();
        let sum: usize = leaked.iter().sum();
        let idx = (((cfg.trials as f64) * 0.99).ceil() as usize)
            .saturating_sub(1)
            .min(cfg.trials - 1);

        rows.push(Exp12Row {
            n: scenario.n,
            psplit: scenario.psplit,
            mean_leaked_bits: (sum as f64) / (cfg.trials as f64),
            p99_leaked_bits: leaked[idx],
        });
    }

    Ok(Exp12Result { rows })
}

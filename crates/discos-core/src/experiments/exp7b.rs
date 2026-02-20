use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp7bConfig {
    pub trials: usize,
    pub seed: u64,
    pub threshold: f64,
}

impl Default for Exp7bConfig {
    fn default() -> Self {
        Self {
            trials: 100_000,
            seed: 7,
            threshold: 2.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp7bCaseResult {
    pub false_positive_rate_product: f64,
    pub false_positive_rate_emerge: f64,
    pub false_positive_count_product: usize,
    pub false_positive_count_emerge: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp7bResult {
    pub schema_version: String,
    pub note: String,
    pub trials: usize,
    pub threshold: f64,
    pub correlated: Exp7bCaseResult,
    pub independent: Exp7bCaseResult,
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

fn sample_evalue_h0(rng: &mut Lcg64) -> f64 {
    if rng.next_f64() < 0.5 {
        0.0
    } else {
        2.0
    }
}

fn fp_rate_from_count(count: usize, trials: usize) -> f64 {
    (count as f64) / (trials as f64)
}

pub async fn run_exp7b(cfg: &Exp7bConfig) -> anyhow::Result<Exp7bResult> {
    if cfg.trials == 0 {
        anyhow::bail!("trials must be greater than zero");
    }
    if !cfg.threshold.is_finite() {
        anyhow::bail!("threshold must be finite");
    }

    let mut correlated_product_fp = 0usize;
    let mut correlated_emerge_fp = 0usize;
    let mut independent_product_fp = 0usize;
    let mut independent_emerge_fp = 0usize;

    let mut rng = Lcg64::new(cfg.seed);

    for _ in 0..cfg.trials {
        let e = sample_evalue_h0(&mut rng);
        let product = e * e;
        let e_merge = (e + e) / 2.0;

        if product > cfg.threshold {
            correlated_product_fp += 1;
        }
        if e_merge > cfg.threshold {
            correlated_emerge_fp += 1;
        }

        let e1 = sample_evalue_h0(&mut rng);
        let e2 = sample_evalue_h0(&mut rng);
        let product = e1 * e2;
        let e_merge = (e1 + e2) / 2.0;

        if product > cfg.threshold {
            independent_product_fp += 1;
        }
        if e_merge > cfg.threshold {
            independent_emerge_fp += 1;
        }
    }

    Ok(Exp7bResult {
        schema_version: "discos.exp7b.v1".to_string(),
        note: "Simulation demonstrating dependence risk under H0; not a claim about real-world oracles."
            .to_string(),
        trials: cfg.trials,
        threshold: cfg.threshold,
        correlated: Exp7bCaseResult {
            false_positive_rate_product: fp_rate_from_count(correlated_product_fp, cfg.trials),
            false_positive_rate_emerge: fp_rate_from_count(correlated_emerge_fp, cfg.trials),
            false_positive_count_product: correlated_product_fp,
            false_positive_count_emerge: correlated_emerge_fp,
        },
        independent: Exp7bCaseResult {
            false_positive_rate_product: fp_rate_from_count(independent_product_fp, cfg.trials),
            false_positive_rate_emerge: fp_rate_from_count(independent_emerge_fp, cfg.trials),
            false_positive_count_product: independent_product_fp,
            false_positive_count_emerge: independent_emerge_fp,
        },
    })
}

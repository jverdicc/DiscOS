use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp3Config {
    pub seed: u64,
    pub n_trials: usize,
    pub intensity: f64,
    pub noise_sigma: f64,
    pub residual_frac_dlc: f64,
    pub residual_frac_pln: f64,
    pub num_bins_mi: usize,
}

impl Default for Exp3Config {
    fn default() -> Self {
        Self {
            seed: 42,
            n_trials: 5000,
            intensity: 10.0,
            noise_sigma: 1.0,
            residual_frac_dlc: 0.05,
            residual_frac_pln: 0.003,
            num_bins_mi: 32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exp3Result {
    pub acc_standard: f64,
    pub mi_standard_bits: f64,
    pub acc_dlc: f64,
    pub mi_dlc_bits: f64,
    pub acc_pln: f64,
    pub mi_pln_bits: f64,
}

const BASE_TIME: f64 = 100.0;

pub async fn run_exp3(cfg: &Exp3Config) -> anyhow::Result<Exp3Result> {
    anyhow::ensure!(cfg.n_trials >= 10, "n_trials must be at least 10");
    anyhow::ensure!(cfg.intensity.is_finite(), "intensity must be finite");
    anyhow::ensure!(cfg.noise_sigma.is_finite(), "noise_sigma must be finite");
    anyhow::ensure!(cfg.noise_sigma >= 0.0, "noise_sigma must be non-negative");
    anyhow::ensure!(
        cfg.residual_frac_dlc.is_finite() && cfg.residual_frac_dlc >= 0.0,
        "residual_frac_dlc must be finite and non-negative"
    );
    anyhow::ensure!(
        cfg.residual_frac_pln.is_finite() && cfg.residual_frac_pln >= 0.0,
        "residual_frac_pln must be finite and non-negative"
    );
    anyhow::ensure!(cfg.num_bins_mi >= 2, "num_bins_mi must be at least 2");

    let mut rng = ChaCha20Rng::seed_from_u64(cfg.seed);
    let mut bits = Vec::with_capacity(cfg.n_trials);
    let mut standard_times = Vec::with_capacity(cfg.n_trials);
    let mut dlc_times = Vec::with_capacity(cfg.n_trials);
    let mut pln_times = Vec::with_capacity(cfg.n_trials);

    for _ in 0..cfg.n_trials {
        let b = if rng.gen_bool(0.5) { 1u8 } else { 0u8 };
        let b_term = b as f64;

        let n_standard = sample_standard_normal(&mut rng) * cfg.noise_sigma;
        let n_dlc = sample_standard_normal(&mut rng) * cfg.noise_sigma;
        let n_pln = sample_standard_normal(&mut rng) * cfg.noise_sigma;

        bits.push(b);
        standard_times.push(BASE_TIME + b_term * cfg.intensity + n_standard);
        dlc_times.push(BASE_TIME + b_term * (cfg.residual_frac_dlc * cfg.intensity) + n_dlc);
        pln_times.push(BASE_TIME + b_term * (cfg.residual_frac_pln * cfg.intensity) + n_pln);
    }

    let acc_standard = train_then_eval_threshold_accuracy(&bits, &standard_times)?;
    let acc_dlc = train_then_eval_threshold_accuracy(&bits, &dlc_times)?;
    let acc_pln = train_then_eval_threshold_accuracy(&bits, &pln_times)?;

    let mi_standard_bits =
        estimate_mutual_information_bits(&bits, &standard_times, cfg.num_bins_mi)?;
    let mi_dlc_bits = estimate_mutual_information_bits(&bits, &dlc_times, cfg.num_bins_mi)?;
    let mi_pln_bits = estimate_mutual_information_bits(&bits, &pln_times, cfg.num_bins_mi)?;

    Ok(Exp3Result {
        acc_standard,
        mi_standard_bits,
        acc_dlc,
        mi_dlc_bits,
        acc_pln,
        mi_pln_bits,
    })
}

fn sample_standard_normal(rng: &mut ChaCha20Rng) -> f64 {
    let u1 = rng.gen::<f64>().max(f64::MIN_POSITIVE);
    let u2 = rng.gen::<f64>();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

fn train_then_eval_threshold_accuracy(bits: &[u8], times: &[f64]) -> anyhow::Result<f64> {
    anyhow::ensure!(bits.len() == times.len(), "bits/times length mismatch");
    anyhow::ensure!(bits.len() >= 2, "need at least two samples");

    let split = bits.len() / 2;
    anyhow::ensure!(split > 0 && split < bits.len(), "invalid train/eval split");

    let train_bits = &bits[..split];
    let train_times = &times[..split];
    let eval_bits = &bits[split..];
    let eval_times = &times[split..];

    let threshold = best_threshold(train_bits, train_times)?;
    Ok(accuracy_with_threshold(eval_bits, eval_times, threshold))
}

fn best_threshold(bits: &[u8], times: &[f64]) -> anyhow::Result<f64> {
    anyhow::ensure!(bits.len() == times.len(), "bits/times length mismatch");
    anyhow::ensure!(!times.is_empty(), "no training samples");

    let mut sorted = times.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let mut candidates = Vec::with_capacity(sorted.len() + 1);
    candidates.push(sorted[0] - 1.0);
    for window in sorted.windows(2) {
        candidates.push((window[0] + window[1]) / 2.0);
    }
    candidates.push(sorted[sorted.len() - 1] + 1.0);

    let mut best = candidates[0];
    let mut best_acc = -1.0f64;

    for threshold in candidates {
        let acc = accuracy_with_threshold(bits, times, threshold);
        if acc > best_acc {
            best_acc = acc;
            best = threshold;
        }
    }

    Ok(best)
}

fn accuracy_with_threshold(bits: &[u8], times: &[f64], threshold: f64) -> f64 {
    let mut correct = 0usize;
    for (b, t) in bits.iter().zip(times) {
        let pred = if *t >= threshold { 1u8 } else { 0u8 };
        if pred == *b {
            correct += 1;
        }
    }
    correct as f64 / (bits.len() as f64)
}

fn estimate_mutual_information_bits(
    bits: &[u8],
    times: &[f64],
    bins: usize,
) -> anyhow::Result<f64> {
    anyhow::ensure!(bits.len() == times.len(), "bits/times length mismatch");
    anyhow::ensure!(!times.is_empty(), "no samples");
    anyhow::ensure!(bins > 0, "bins must be positive");

    let n = bits.len();
    let mut indices = (0..n).collect::<Vec<_>>();
    indices.sort_by(|&i, &j| times[i].total_cmp(&times[j]));

    let mut assigned_bins = vec![0usize; n];
    for (rank, idx) in indices.into_iter().enumerate() {
        assigned_bins[idx] = (rank * bins) / n;
    }

    let mut counts = vec![vec![0usize; bins]; 2];
    let mut count_b = [0usize; 2];
    let mut count_t = vec![0usize; bins];

    for (b, bin) in bits.iter().zip(assigned_bins) {
        let b_idx = (*b as usize).min(1);
        counts[b_idx][bin] += 1;
        count_b[b_idx] += 1;
        count_t[bin] += 1;
    }

    let n_f = n as f64;
    let mut mi = 0.0f64;
    for b in 0..2 {
        for (bin, &joint_count) in counts[b].iter().enumerate() {
            if joint_count == 0 {
                continue;
            }
            let p_bt = (joint_count as f64) / n_f;
            let p_b = (count_b[b] as f64) / n_f;
            let p_t = (count_t[bin] as f64) / n_f;
            mi += p_bt * (p_bt / (p_b * p_t)).log2();
        }
    }
    Ok(mi)
}

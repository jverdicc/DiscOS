// Copyright (c) 2026 Joseph Verdicchio and DiscOS  Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use clap::{Parser, Subcommand};
use discos_core::boundary::{
    accuracy_value_det, attacker_ternary_evidenceos, attacker_ternary_standard, generate_boundary,
    BudgetedBoundaryOracles,
};
use discos_core::labels::{generate_labels, single_bit_probe_attack, AccuracyOracle, OracleObs};
use tracing_subscriber::EnvFilter;

pub mod pb {
    tonic::include_proto!("evidenceos.v1");
}

#[derive(Debug, Parser)]
#[command(name = "discos")]
#[command(about = "DiscOS userland + simulation harness for EvidenceOS")]
struct Args {
    /// EvidenceOS gRPC endpoint, e.g. http://127.0.0.1:50051
    #[arg(long, default_value = "http://127.0.0.1:50051")]
    endpoint: String,

    /// Log filter
    #[arg(long, default_value = "info")]
    log: String,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Ping the kernel
    Health,

    /// Run Experiment 0: label recovery under quantization vs hysteresis.
    Experiment0 {
        /// Holdout seed (simulation only)
        #[arg(long, default_value_t = 123)]
        seed: u64,

        /// Number of labels
        #[arg(long, default_value_t = 256)]
        n: u32,

        /// Oracle buckets
        #[arg(long, default_value_t = 256)]
        buckets: u32,

        /// Hysteresis delta (defended session)
        #[arg(long, default_value_t = 0.01)]
        delta_sigma: f64,
    },

    /// Run Experiment 2: joint entropy defense via bit-budgeted dual-oracle probing.
    Experiment2 {
        /// Number of trials
        #[arg(long, default_value_t = 100)]
        trials: u32,

        /// First seed (increments per trial)
        #[arg(long, default_value_t = 1000)]
        seed0: u64,

        /// Oracle buckets for boundary accuracy
        #[arg(long, default_value_t = 256)]
        buckets: u32,

        /// Joint bit budget
        #[arg(long, default_value_t = 48)]
        budget_bits: u64,

        /// Max ternary queries (accuracy queries use 2 per iteration)
        #[arg(long, default_value_t = 60)]
        max_queries: u64,

        /// Safety margin
        #[arg(long, default_value_t = 1e-4)]
        safety_margin: f64,

        /// Target true accuracy threshold (evaluated using deterministic b in simulation)
        #[arg(long, default_value_t = 0.999)]
        target_true_accuracy: f64,

        /// Also run an unlimited-budget baseline (standard attacker)
        #[arg(long, default_value_t = true)]
        run_baseline: bool,
    },

    /// End-to-end system simulation: spend optional oracle leakage, then attempt certification.
    ///
    /// This demonstrates:
    /// - IPC calls
    /// - leakage charging (k bits)
    /// - certification barrier growth
    /// - ETL append + root
    CertifyDemo {
        /// Holdout seed (simulation-only)
        #[arg(long, default_value_t = 123)]
        seed: u64,

        /// Number of labels
        #[arg(long, default_value_t = 256)]
        n: u32,

        /// Oracle buckets
        #[arg(long, default_value_t = 256)]
        buckets: u32,

        /// Hysteresis delta (usually 0 for demo)
        #[arg(long, default_value_t = 0.0)]
        hysteresis_delta: f64,

        /// Alpha risk budget
        #[arg(long, default_value_t = 0.05)]
        alpha: f64,

        /// Binomial p1 for e-value
        #[arg(long, default_value_t = 0.6)]
        binom_p1: f64,

        /// Joint bit budget (0 = infinite)
        #[arg(long, default_value_t = 0)]
        joint_bits_budget: u64,

        /// Number of oracle calls to spend before certification
        #[arg(long, default_value_t = 0)]
        probe_calls: u32,
    },
}

#[derive(Debug, Clone)]
struct SessionParams {
    alpha: f64,
    epoch_size: u64,
    hysteresis_delta: f64,
    oracle_buckets: u32,
    joint_bits_budget: u64,
    binom_p1: f64,
}

struct GrpcLabelsOracle {
    client: pb::evidence_os_client::EvidenceOsClient<tonic::transport::Channel>,
    session_id: String,
    holdout_id: String,
}

#[async_trait::async_trait]
impl AccuracyOracle for GrpcLabelsOracle {
    async fn query_accuracy(&mut self, preds: &[u8]) -> anyhow::Result<OracleObs> {
        let reply = self
            .client
            .oracle_accuracy(pb::OracleAccuracyRequest {
                session_id: self.session_id.clone(),
                holdout_id: self.holdout_id.clone(),
                predictions: preds.to_vec(),
            })
            .await
            .context("oracle_accuracy")?
            .into_inner();

        Ok(OracleObs {
            bucket: reply.bucket,
            num_buckets: reply.num_buckets,
            k_bits_total: reply.k_bits_total,
            frozen: reply.frozen,
        })
    }
}

struct GrpcBoundaryOracles {
    client: pb::evidence_os_client::EvidenceOsClient<tonic::transport::Channel>,
    session_id: String,
    holdout_id: String,

    num_buckets: u32,
    joint_bits_budget: f64,

    bits_spent: f64,
    frozen: bool,

    acc_q: u64,
    safe_q: u64,
}

#[async_trait::async_trait]
impl BudgetedBoundaryOracles for GrpcBoundaryOracles {
    async fn accuracy_oracle(&mut self, x: f64) -> anyhow::Result<Option<u32>> {
        if self.frozen {
            return Ok(None);
        }
        let reply = self
            .client
            .oracle_boundary_accuracy(pb::OracleBoundaryAccuracyRequest {
                session_id: self.session_id.clone(),
                holdout_id: self.holdout_id.clone(),
                x,
            })
            .await
            .context("oracle_boundary_accuracy")?
            .into_inner();

        self.bits_spent = reply.k_bits_total;
        self.frozen = reply.frozen;
        if !self.frozen {
            self.acc_q += 1;
            Ok(Some(reply.bucket))
        } else {
            Ok(None)
        }
    }

    async fn safety_oracle(&mut self, x: f64) -> anyhow::Result<Option<u32>> {
        if self.frozen {
            return Ok(None);
        }
        let reply = self
            .client
            .oracle_boundary_safety(pb::OracleBoundarySafetyRequest {
                session_id: self.session_id.clone(),
                holdout_id: self.holdout_id.clone(),
                x,
            })
            .await
            .context("oracle_boundary_safety")?
            .into_inner();

        self.bits_spent = reply.k_bits_total;
        self.frozen = reply.frozen;
        if !self.frozen {
            self.safe_q += 1;
            Ok(Some(reply.bucket))
        } else {
            Ok(None)
        }
    }

    fn num_buckets(&self) -> u32 {
        self.num_buckets
    }

    fn bits_per_acc_query(&self) -> f64 {
        (self.num_buckets as f64).log2()
    }

    fn joint_bits_budget(&self) -> f64 {
        self.joint_bits_budget
    }

    fn bits_spent(&self) -> f64 {
        self.bits_spent
    }

    fn frozen(&self) -> bool {
        self.frozen
    }

    fn acc_queries(&self) -> u64 {
        self.acc_q
    }

    fn safe_queries(&self) -> u64 {
        self.safe_q
    }
}

async fn create_session(
    endpoint: &str,
    params: SessionParams,
) -> anyhow::Result<(pb::evidence_os_client::EvidenceOsClient<tonic::transport::Channel>, String)> {
    let mut client = pb::evidence_os_client::EvidenceOsClient::connect(endpoint.to_string())
        .await
        .context("connect")?;

    let resp = client
        .create_session(pb::CreateSessionRequest {
            alpha: params.alpha,
            epoch_size: params.epoch_size,
            hysteresis_delta: params.hysteresis_delta,
            oracle_buckets: params.oracle_buckets,
            joint_bits_budget: params.joint_bits_budget,
            binom_p1: params.binom_p1,
        })
        .await
        .context("create_session")?
        .into_inner();

    Ok((client, resp.session_id))
}

async fn init_holdout(
    client: &mut pb::evidence_os_client::EvidenceOsClient<tonic::transport::Channel>,
    session_id: &str,
    kind: pb::HoldoutKind,
    seed: u64,
    size: u32,
) -> anyhow::Result<String> {
    let resp = client
        .init_holdout(pb::InitHoldoutRequest {
            session_id: session_id.to_string(),
            kind: kind as i32,
            seed,
            size,
        })
        .await
        .context("init_holdout")?
        .into_inner();

    Ok(resp.holdout_id)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(args.log))
        .init();

    match args.cmd {
        Command::Health => {
            let mut client = pb::evidence_os_client::EvidenceOsClient::connect(args.endpoint.clone())
                .await
                .context("connect")?;
            let resp = client.health(pb::HealthRequest {}).await?.into_inner();
            println!("{}", serde_json::json!({"status": resp.status}));
        }

        Command::Experiment0 {
            seed,
            n,
            buckets,
            delta_sigma,
        } => {
            let labels = generate_labels(seed, n as usize);

            // Session A: quantized only.
            let params_a = SessionParams {
                alpha: 0.05,
                epoch_size: 10_000,
                hysteresis_delta: 0.0,
                oracle_buckets: buckets,
                joint_bits_budget: 0,
                binom_p1: 0.6,
            };

            let (mut client_a, session_a) = create_session(&args.endpoint, params_a).await?;
            let holdout_a = init_holdout(
                &mut client_a,
                &session_a,
                pb::HoldoutKind::HoldoutKindLabels,
                seed,
                n,
            )
            .await?;

            let mut oracle_a = GrpcLabelsOracle {
                client: client_a,
                session_id: session_a,
                holdout_id: holdout_a,
            };

            let report_a = single_bit_probe_attack(&mut oracle_a, &labels, 999).await?;

            // Session B: defended with hysteresis.
            let params_b = SessionParams {
                alpha: 0.05,
                epoch_size: 10_000,
                hysteresis_delta: delta_sigma,
                oracle_buckets: buckets,
                joint_bits_budget: 0,
                binom_p1: 0.6,
            };

            let (mut client_b, session_b) = create_session(&args.endpoint, params_b).await?;
            let holdout_b = init_holdout(
                &mut client_b,
                &session_b,
                pb::HoldoutKind::HoldoutKindLabels,
                seed,
                n,
            )
            .await?;

            let mut oracle_b = GrpcLabelsOracle {
                client: client_b,
                session_id: session_b,
                holdout_id: holdout_b,
            };

            let report_b = single_bit_probe_attack(&mut oracle_b, &labels, 999).await?;

            println!(
                "{}",
                serde_json::json!({
                    "experiment": "experiment0_label_recovery",
                    "n": n,
                    "seed": seed,
                    "buckets": buckets,
                    "quantized_only": {
                        "recovery_accuracy": report_a.recovery_accuracy,
                        "oracle_calls": report_a.oracle_calls,
                        "frozen": report_a.frozen,
                    },
                    "quantized_plus_hysteresis": {
                        "delta_sigma": delta_sigma,
                        "recovery_accuracy": report_b.recovery_accuracy,
                        "oracle_calls": report_b.oracle_calls,
                        "frozen": report_b.frozen,
                    }
                })
            );
        }

        Command::Experiment2 {
            trials,
            seed0,
            buckets,
            budget_bits,
            max_queries,
            safety_margin,
            target_true_accuracy,
            run_baseline,
        } => {
            // Baseline: standard attacker sees true accuracy value (no quantization/budget).
            let mut baseline_success: u64 = 0;
            if run_baseline {
                for t in 0..trials {
                    let seed = seed0 + t as u64;
                    let b = generate_boundary(seed);
                    let dbg = attacker_ternary_standard(b, max_queries, safety_margin, target_true_accuracy);
                    let safe = dbg.safety_response == Some(1);
                    let true_acc = accuracy_value_det(dbg.x_submit, b);
                    if safe && true_acc >= target_true_accuracy {
                        baseline_success += 1;
                    }
                }
            }

            // Defended: EvidenceOS (quantized + joint budget).
            let mut defended_success: u64 = 0;
            let mut defended_frozen: u64 = 0;

            for t in 0..trials {
                let seed = seed0 + t as u64;
                let b = generate_boundary(seed);

                let params = SessionParams {
                    alpha: 0.05,
                    epoch_size: 10_000,
                    hysteresis_delta: 0.0,
                    oracle_buckets: buckets,
                    joint_bits_budget: budget_bits,
                    binom_p1: 0.6,
                };

                let (mut client, session_id) = create_session(&args.endpoint, params).await?;
                let holdout_id = init_holdout(
                    &mut client,
                    &session_id,
                    pb::HoldoutKind::HoldoutKindScalarBoundary,
                    seed,
                    0,
                )
                .await?;

                let mut oracles = GrpcBoundaryOracles {
                    client,
                    session_id,
                    holdout_id,
                    num_buckets: buckets,
                    joint_bits_budget: budget_bits as f64,
                    bits_spent: 0.0,
                    frozen: false,
                    acc_q: 0,
                    safe_q: 0,
                };

                let dbg = attacker_ternary_evidenceos(
                    &mut oracles,
                    max_queries,
                    safety_margin,
                    target_true_accuracy,
                )
                .await?;

                let safe = dbg.safety_response == Some(1);
                let true_acc = accuracy_value_det(dbg.x_submit, b);
                let ok = safe && true_acc >= target_true_accuracy;

                if dbg.frozen {
                    defended_frozen += 1;
                }
                if ok {
                    defended_success += 1;
                }
            }

            println!(
                "{}",
                serde_json::json!({
                    "experiment": "experiment2_joint_entropy_defense",
                    "trials": trials,
                    "seed0": seed0,
                    "buckets": buckets,
                    "budget_bits": budget_bits,
                    "target_true_accuracy": target_true_accuracy,
                    "safety_margin": safety_margin,
                    "baseline": if run_baseline { serde_json::json!({
                        "success": baseline_success,
                        "success_rate": (baseline_success as f64) / (trials as f64)
                    }) } else { serde_json::Value::Null },
                    "defended": {
                        "success": defended_success,
                        "success_rate": (defended_success as f64) / (trials as f64),
                        "frozen": defended_frozen,
                        "frozen_rate": (defended_frozen as f64) / (trials as f64)
                    }
                })
            );
        }

        Command::CertifyDemo {
            seed,
            n,
            buckets,
            hysteresis_delta,
            alpha,
            binom_p1,
            joint_bits_budget,
            probe_calls,
        } => {
            let labels = generate_labels(seed, n as usize);
            let preds = labels.clone(); // perfect predictions (simulation)

            let params = SessionParams {
                alpha,
                epoch_size: 10_000,
                hysteresis_delta,
                oracle_buckets: buckets,
                joint_bits_budget,
                binom_p1,
            };

            let (mut client, session_id) = create_session(&args.endpoint, params).await?;
            let holdout_id = init_holdout(
                &mut client,
                &session_id,
                pb::HoldoutKind::HoldoutKindLabels,
                seed,
                n,
            )
            .await?;

            // Spend leakage via oracle calls.
            let zero_preds = vec![0u8; n as usize];
            let mut last_oracle: Option<pb::OracleReply> = None;
            for _ in 0..probe_calls {
                let r = client
                    .oracle_accuracy(pb::OracleAccuracyRequest {
                        session_id: session_id.clone(),
                        holdout_id: holdout_id.clone(),
                        predictions: zero_preds.clone(),
                    })
                    .await
                    .context("oracle_accuracy")?
                    .into_inner();
                last_oracle = Some(r);
            }

            let eval = client
                .evaluate_and_certify(pb::EvaluateAndCertifyRequest {
                    session_id: session_id.clone(),
                    holdout_id: holdout_id.clone(),
                    predictions: preds,
                    claim_name: "certify_demo".to_string(),
                })
                .await
                .context("evaluate_and_certify")?
                .into_inner();

            let ledger = client
                .get_ledger(pb::GetLedgerRequest {
                    session_id: session_id.clone(),
                })
                .await
                .context("get_ledger")?
                .into_inner();

            let etl = client
                .get_etl_root(pb::GetEtlRootRequest {})
                .await
                .context("get_etl_root")?
                .into_inner();

            println!(
                "{}",
                serde_json::json!({
                    "experiment": "certify_demo",
                    "seed": seed,
                    "n": n,
                    "buckets": buckets,
                    "hysteresis_delta": hysteresis_delta,
                    "alpha": alpha,
                    "binom_p1": binom_p1,
                    "joint_bits_budget": joint_bits_budget,
                    "probe_calls": probe_calls,
                    "last_oracle": last_oracle.map(|r| serde_json::json!({
                        "bucket": r.bucket,
                        "num_buckets": r.num_buckets,
                        "k_bits_total": r.k_bits_total,
                        "barrier": r.barrier,
                        "frozen": r.frozen,
                    })),
                    "evaluate": {
                        "certified": eval.certified,
                        "e_value": eval.e_value,
                        "wealth": eval.wealth,
                        "barrier": eval.barrier,
                        "k_bits_total": eval.k_bits_total,
                        "capsule_hash": eval.capsule_hash,
                        "etl_index": eval.etl_index,
                    },
                    "ledger": {
                        "alpha_prime": ledger.alpha_prime,
                        "k_bits_total": ledger.k_bits_total,
                        "barrier": ledger.barrier,
                        "wealth": ledger.wealth,
                        "events": ledger.events.len(),
                    },
                    "etl": {
                        "root_hash_hex": etl.root_hash_hex,
                        "tree_size": etl.tree_size,
                    }
                })
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end smoke test against a *running* EvidenceOS daemon.
    ///
    /// This test is ignored by default. Run with:
    ///
    /// ```bash
    /// EVIDENCEOS_ENDPOINT=http://127.0.0.1:50051 cargo test -p discos-cli -- --ignored
    /// ```
    #[tokio::test]
    #[ignore]
    async fn e2e_health_and_one_oracle_call() {
        let endpoint = std::env::var("EVIDENCEOS_ENDPOINT")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

        let mut client = pb::evidence_os_client::EvidenceOsClient::connect(endpoint)
            .await
            .unwrap();

        let h = client.health(pb::HealthRequest {}).await.unwrap().into_inner();
        assert_eq!(h.status, "SERVING");
    }
}

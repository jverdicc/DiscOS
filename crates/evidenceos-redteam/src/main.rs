use clap::Parser;
use evidenceos_redteam::{run_redteam, Thresholds};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "http://127.0.0.1:50051")]
    endpoint: String,
    #[arg(long, default_value_t = 16)]
    rounds: usize,
    #[arg(long, default_value_t = 0.55)]
    max_arm_auc: f64,
    #[arg(long, default_value_t = 0.0)]
    max_size_variance: f64,
    #[arg(long, default_value_t = true)]
    strict_pln: bool,
    #[arg(long, default_value_t = true)]
    production_mode: bool,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let thresholds = Thresholds {
        max_arm_auc: args.max_arm_auc,
        max_size_variance: args.max_size_variance,
        enforce_strict_pln: args.strict_pln,
        production_mode: args.production_mode,
    };
    let report = run_redteam(&args.endpoint, args.rounds, &thresholds).await?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

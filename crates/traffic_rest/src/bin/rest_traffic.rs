use std::path::PathBuf;

use clap::Parser;

use traffic_rest::metrics::summarize;
use traffic_rest::{TrafficRunConfig, run_profile};

#[derive(Debug, Parser)]
#[command(
    name = "rest_traffic",
    version,
    about = "Configurable REST traffic generator for rate limiting experiments."
)]
struct Args {
    /// Optional path to a TOML config file.
    #[arg(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let run_config = TrafficRunConfig::load(args.config.as_deref())
        .unwrap_or_else(|err| panic!("invalid traffic config: {err}"));
    let (profile, key_mode) = run_config
        .to_profile_and_mode()
        .unwrap_or_else(|err| panic!("invalid traffic config: {err}"));

    println!("REST traffic run config:");
    println!("  target_url: {}", profile.target_url);
    println!(
        "  duration: {:?}, rps: {}, concurrency: {}",
        profile.duration, profile.requests_per_second, profile.concurrency
    );
    println!("  key_mode: {:?}", key_mode);

    let observations = run_profile(profile.clone(), key_mode).await;
    let summary = summarize(&observations);

    println!("REST traffic run complete");
    println!("  samples: {}", summary.total);

    println!("Results:");
    println!("  200 OK:                {}", summary.ok);
    println!("  429 Too Many Requests: {}", summary.too_many);
    println!("  Other:                 {}", summary.other);
    println!("  -------------------------------");
    println!("  Total:                 {}", summary.total);

    println!("Latency:");
    println!("  min: {:?}", summary.latency_min);
    println!("  avg: {:?}", summary.latency_avg);
    println!("  p50: {:?}", summary.latency_p50);
    println!("  p95: {:?}", summary.latency_p95);
    println!("  p99: {:?}", summary.latency_p99);
    println!("  max: {:?}", summary.latency_max);
}

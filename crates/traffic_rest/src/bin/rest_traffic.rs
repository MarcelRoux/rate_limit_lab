use std::time::Duration;

use traffic_rest::metrics::summarize;
use traffic_rest::{KeyMode, TrafficProfile, run_profile};

#[tokio::main]
async fn main() {
    // Assumes your M2.3 Axum app is running locally and rate-limiting "/" route.
    // Adjust the URL to match your server.
    let profile = TrafficProfile {
        target_url: "http://127.0.0.1:3000/".to_string(),
        duration: Duration::from_secs(5),
        requests_per_second: 60_000,
        concurrency: 16,
        key_header: "x-api-key".to_string(),
    };

    // Try these modes:
    // - KeyMode::Keyless
    // - KeyMode::SingleKey("user1".into())
    // - KeyMode::RoundRobin(vec!["u1".into(), "u2".into(), "u3".into()])
    // let key_mode = KeyMode::RoundRobin(vec![
    //     "user1".into(),
    //     "user2".into(),
    //     "user3".into(),
    //     "user4".into(),
    // ]);
    let key_mode = KeyMode::SingleKey("user1".into());

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

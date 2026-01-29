use std::time::Duration;

use http::StatusCode;

use crate::client::Observation;

#[derive(Clone, Debug)]
pub struct Summary {
    pub total: u64,
    pub ok: u64,
    pub too_many: u64,
    pub other: u64,

    pub latency_min: Duration,
    pub latency_avg: Duration,
    pub latency_p50: Duration,
    pub latency_p95: Duration,
    pub latency_p99: Duration,
    pub latency_max: Duration,
}

fn percentile(sorted: &[Duration], p: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let n = sorted.len();
    let idx = ((p * (n as f64 - 1.0)).round() as usize).min(n - 1);
    sorted[idx]
}

pub fn summarize(obs: &[Observation]) -> Summary {
    let total = obs.len() as u64;

    let mut ok = 0u64;
    let mut too_many = 0u64;
    let mut other = 0u64;

    let mut latencies: Vec<Duration> = Vec::with_capacity(obs.len());
    let mut sum = Duration::ZERO;

    for o in obs {
        match o.status {
            StatusCode::OK => ok += 1,
            StatusCode::TOO_MANY_REQUESTS => too_many += 1,
            _ => other += 1,
        }
        latencies.push(o.latency);
        sum += o.latency;
    }

    latencies.sort_unstable();

    let latency_min = *latencies.first().unwrap_or(&Duration::ZERO);
    let latency_max = *latencies.last().unwrap_or(&Duration::ZERO);
    let latency_avg = if total == 0 {
        Duration::ZERO
    } else {
        // integer division on nanos (safe enough for stats printing)
        let avg_nanos = sum.as_nanos() / total as u128;
        Duration::from_nanos(avg_nanos.min(u64::MAX as u128) as u64)
    };

    Summary {
        total,
        ok,
        too_many,
        other,
        latency_min,
        latency_avg,
        latency_p50: percentile(&latencies, 0.50),
        latency_p95: percentile(&latencies, 0.95),
        latency_p99: percentile(&latencies, 0.99),
        latency_max,
    }
}

use std::sync::atomic::{AtomicU64, Ordering};

use axum::{
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};

static RL_REQUESTS_TOTAL: AtomicU64 = AtomicU64::new(0);
static RL_DENIES_TOTAL: AtomicU64 = AtomicU64::new(0);
static RL_REQUEST_LATENCY_MICROS: AtomicU64 = AtomicU64::new(0);

pub fn record_request(is_denied: bool, latency_micros: u64) {
    RL_REQUESTS_TOTAL.fetch_add(1, Ordering::Relaxed);
    if is_denied {
        RL_DENIES_TOTAL.fetch_add(1, Ordering::Relaxed);
    }
    RL_REQUEST_LATENCY_MICROS.store(latency_micros, Ordering::Relaxed);
}

pub fn metrics_payload() -> String {
    let requests_total = RL_REQUESTS_TOTAL.load(Ordering::Relaxed);
    let denies_total = RL_DENIES_TOTAL.load(Ordering::Relaxed);
    let latency_ms = RL_REQUEST_LATENCY_MICROS.load(Ordering::Relaxed) as f64 / 1000.0;

    format!(
        concat!(
            "# HELP rl_requests_total Total requests observed by REST adapter.\n",
            "# TYPE rl_requests_total counter\n",
            "rl_requests_total {requests_total}\n",
            "# HELP rl_denies_total Total denied requests by REST adapter.\n",
            "# TYPE rl_denies_total counter\n",
            "rl_denies_total {denies_total}\n",
            "# HELP rl_request_latency_ms Request latency in milliseconds.\n",
            "# TYPE rl_request_latency_ms gauge\n",
            "rl_request_latency_ms {latency_ms:.3}\n"
        ),
        requests_total = requests_total,
        denies_total = denies_total,
        latency_ms = latency_ms
    )
}

pub async fn metrics_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "content-type",
        HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    (headers, metrics_payload())
}

#[cfg(test)]
mod tests {
    use super::{
        RL_DENIES_TOTAL, RL_REQUEST_LATENCY_MICROS, RL_REQUESTS_TOTAL, metrics_payload,
        record_request,
    };
    use std::sync::atomic::Ordering;

    fn reset_metrics() {
        RL_REQUESTS_TOTAL.store(0, Ordering::Relaxed);
        RL_DENIES_TOTAL.store(0, Ordering::Relaxed);
        RL_REQUEST_LATENCY_MICROS.store(0, Ordering::Relaxed);
    }

    #[test]
    fn metrics_payload_reflects_recorded_values() {
        reset_metrics();
        record_request(false, 2500);
        record_request(true, 1500);

        let text = metrics_payload();
        assert!(text.contains("rl_requests_total 2"));
        assert!(text.contains("rl_denies_total 1"));
        assert!(text.contains("rl_request_latency_ms 1.500"));
    }
}

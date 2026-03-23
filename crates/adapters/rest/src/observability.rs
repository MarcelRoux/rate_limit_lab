use axum::{
    http::{HeaderMap, HeaderValue},
    response::IntoResponse,
};

const METRICS_PAYLOAD: &str = concat!(
    "# HELP rl_requests_total Total requests observed by REST adapter.\n",
    "# TYPE rl_requests_total counter\n",
    "rl_requests_total 0\n",
    "# HELP rl_denies_total Total denied requests by REST adapter.\n",
    "# TYPE rl_denies_total counter\n",
    "rl_denies_total 0\n",
    "# HELP rl_request_latency_ms Request latency in milliseconds.\n",
    "# TYPE rl_request_latency_ms gauge\n",
    "rl_request_latency_ms 0\n"
);

pub fn metrics_payload() -> &'static str {
    METRICS_PAYLOAD
}

pub async fn metrics_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "content-type",
        HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    (headers, metrics_payload())
}

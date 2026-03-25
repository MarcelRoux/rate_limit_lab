#![cfg(feature = "observability_ui")]

use axum::{
    Router,
    body::{Body, to_bytes},
    routing::get,
};
use hyper::{Request, StatusCode};
use tower::ServiceExt;

use rate_limit::{
    event_sink::NoopEventSink,
    factory::hierarchical_limiter,
    models::{InstrumentationLevel, RateLimit, RateLimitKey},
};
use rest::{
    config::RestServerConfig,
    layer::RateLimitLayer,
    observability::{metrics_handler, metrics_payload},
};
use std::sync::Arc;

fn parse_metric_value(payload: &str, metric: &str) -> f64 {
    payload
        .lines()
        .find(|line| line.starts_with(metric))
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0)
}

#[tokio::test]
async fn metrics_endpoint_returns_prometheus_metric_families_when_enabled() {
    let cfg_toml = r#"
bind_address = "127.0.0.1:3000"
key_mode = "header_or_anonymous"
key_header = "x-api-key"
anonymous_key = "anonymous"
instrumentation = "off"

[limits]
global_per_second = 1000
per_key_per_second = 1000

[observability]
enabled = true
"#;

    let cfg: RestServerConfig = toml::from_str(cfg_toml).expect("parse config");
    assert!(cfg.observability_enabled());

    let app: Router = Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/metrics", get(metrics_handler));
    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let text = String::from_utf8(body.to_vec()).expect("utf8");
    assert!(text.contains("# HELP rl_requests_total"));
    assert!(text.contains("# TYPE rl_requests_total counter"));
}

#[test]
fn metrics_payload_contains_required_metric_families() {
    let text = metrics_payload();
    assert!(text.contains("# HELP rl_requests_total"));
    assert!(text.contains("# HELP rl_denies_total"));
    assert!(text.contains("# HELP rl_request_latency_ms"));
}

#[tokio::test]
async fn metrics_payload_changes_after_allow_and_deny_requests() {
    let sink = Arc::new(NoopEventSink);
    let global_limit = RateLimit::per_second(10).expect("global limit");
    let key_limit = RateLimit::per_second(1).expect("key limit");
    let limiter = hierarchical_limiter(
        global_limit.to_quota(),
        key_limit.to_quota(),
        sink,
        InstrumentationLevel::Off,
    );
    let _ = limiter.check(&RateLimitKey("anonymous".to_string()));

    let app: Router = Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/metrics", get(metrics_handler))
        .layer(RateLimitLayer::new(Arc::new(limiter)));

    let before = metrics_payload();
    let before_requests = parse_metric_value(&before, "rl_requests_total");
    let before_denies = parse_metric_value(&before, "rl_denies_total");

    let allow_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/")
                .header("x-api-key", "user_allow")
                .body(Body::empty())
                .expect("allow request"),
        )
        .await
        .expect("allow response");
    assert_eq!(allow_response.status(), StatusCode::OK);

    let deny_response = app
        .oneshot(
            Request::builder()
                .uri("/")
                .header("x-api-key", "anonymous")
                .body(Body::empty())
                .expect("deny request"),
        )
        .await
        .expect("deny response");
    assert_eq!(deny_response.status(), StatusCode::TOO_MANY_REQUESTS);

    let after = metrics_payload();
    let after_requests = parse_metric_value(&after, "rl_requests_total");
    let after_denies = parse_metric_value(&after, "rl_denies_total");
    let after_latency_ms = parse_metric_value(&after, "rl_request_latency_ms");

    assert!(after_requests >= before_requests + 2.0);
    assert!(after_denies >= before_denies + 1.0);
    assert!(after_latency_ms >= 0.0);
}

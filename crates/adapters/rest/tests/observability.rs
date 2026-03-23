#![cfg(feature = "observability_ui")]

use axum::{
    Router,
    body::{Body, to_bytes},
    routing::get,
};
use hyper::{Request, StatusCode};
use tower::ServiceExt;

use rest::{
    config::RestServerConfig,
    observability::{metrics_handler, metrics_payload},
};

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

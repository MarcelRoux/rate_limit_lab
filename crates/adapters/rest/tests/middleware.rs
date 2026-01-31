use std::sync::Arc;

use axum::{Router, body::Body, routing::get};
use hyper::{Request, StatusCode};
use tokio;
use tower::ServiceExt;

use rate_limit::{
    event_sink::NoopEventSink,
    factory::hierarchical_limiter,
    hierarchical_limiter::HierarchicalLimiter,
    models::{InstrumentationLevel, RateLimit, RateLimitKey},
};
use rest::layer::RateLimitLayer;

/// Helper: construct a HierarchicalLimiter that always denies.
fn deny_hierarchical_limiter() -> HierarchicalLimiter<RateLimitKey, Arc<NoopEventSink>> {
    let sink = Arc::new(NoopEventSink);
    let global_limit = RateLimit::per_second(1).unwrap();
    let key_limit = RateLimit::per_second(1).unwrap();
    hierarchical_limiter(
        global_limit.to_quota(),
        key_limit.to_quota(),
        sink.clone(),
        InstrumentationLevel::Off,
    )
}

#[tokio::test]
async fn rest_middleware_denies_request() {
    let limiter = deny_hierarchical_limiter();

    // Pre-consume the single token so the next check is denied.
    let _ = limiter.check(&RateLimitKey("anonymous".to_string()));

    let app: Router = Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(RateLimitLayer::new(Arc::new(limiter)));

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

/// Helper: construct a HierarchicalLimiter that always allows.
fn allow_hierarchical_limiter() -> HierarchicalLimiter<RateLimitKey, Arc<NoopEventSink>> {
    let sink = Arc::new(NoopEventSink);
    let global_limit = RateLimit::per_second(1).unwrap();
    let key_limit = RateLimit::per_second(1).unwrap();
    hierarchical_limiter(
        global_limit.to_quota(),
        key_limit.to_quota(),
        sink.clone(),
        InstrumentationLevel::Off,
    )
}

#[tokio::test]
async fn rest_middleware_allows_request() {
    let limiter = allow_hierarchical_limiter();

    let app: Router = Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(RateLimitLayer::new(Arc::new(limiter)));

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

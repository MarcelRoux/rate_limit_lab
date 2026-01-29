use std::net::SocketAddr;
use std::sync::Arc;

use axum::{Router, routing::get};
use tokio::signal;

use rate_limit::{
    event_sink::NoopEventSink,
    factory::hierarchical_limiter,
    models::{InstrumentationLevel, RateLimit},
};

use rest::layer::RateLimitLayer;

#[tokio::main]
async fn main() {
    // ---- Rate limit configuration (M2.3 scope) ----
    let sink = Arc::new(NoopEventSink);
    let global_limit = RateLimit::per_second(1_000).expect("non-zero");
    let per_key_limit = RateLimit::per_second(1_000).expect("non-zero");

    let limiter = hierarchical_limiter(
        global_limit.to_quota(),
        per_key_limit.to_quota(),
        sink.clone(),
        InstrumentationLevel::Basic,
    );

    // ---- Axum app ----
    let app: Router = Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(RateLimitLayer::new(limiter));

    // ---- Server ----
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("REST rate-limited server listening on {}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    println!("shutdown signal received");
}

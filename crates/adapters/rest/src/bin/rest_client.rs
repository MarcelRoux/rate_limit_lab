use std::{net::SocketAddr, sync::Arc};

use axum::{Router, routing::get};
use tokio::signal;

use rate_limit::{event_sink::NoopEventSink, models::RateLimitKey};

use rest::{config::RestServerConfig, layer::RateLimitLayer};

#[cfg(feature = "in_memory_limiter")]
use rate_limit::{
    factory::hierarchical_limiter, hierarchical_limiter::HierarchicalLimiter, models::RateLimit,
};
#[cfg(feature = "in_memory_limiter")]
async fn get_limiter(cfg: &RestServerConfig) -> HierarchicalLimiter<RateLimitKey, NoopEventSink> {
    log::info!("Configuring in-memory hierarchical limiter.");
    let sink = NoopEventSink;
    let global_limit = RateLimit::per_second(cfg.limits.global_per_second)
        .expect("global_per_second must be non-zero");
    let per_key_limit = RateLimit::per_second(cfg.limits.per_key_per_second)
        .expect("per_key_per_second must be non-zero");

    hierarchical_limiter(
        global_limit.to_quota(),
        per_key_limit.to_quota(),
        sink,
        cfg.instrumentation_level(),
    )
}

#[cfg(feature = "distributed_limiter")]
use {
    rate_limit_distributed::DistributedKeyedLimiter,
    state_backend::{LimitSpec, RedisBackend},
    std::time::Duration,
};
#[cfg(feature = "distributed_limiter")]
async fn get_limiter(
    cfg: &RestServerConfig,
) -> DistributedKeyedLimiter<RateLimitKey, RedisBackend, NoopEventSink> {
    log::info!("Configuring distributed limiter with Redis backend.");
    let sink = NoopEventSink;
    let backend = RedisBackend::connect_from_env().await.expect("redis");

    let (namespace, window_ms, max) = cfg
        .distributed
        .as_ref()
        .map(|d| (d.namespace.as_str(), d.window_ms, d.max))
        .unwrap_or(("rest", 1_000, 1_000));

    DistributedKeyedLimiter::new(
        Arc::new(backend),
        namespace,
        LimitSpec {
            window: Duration::from_millis(window_ms),
            max,
        },
        sink,
        cfg.instrumentation_level(),
    )
}
#[cfg(feature = "hybrid_limiter")]
compile_error!("'hybrid_limiter' feature not yet implemented.");

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cfg = RestServerConfig::load();
    log::info!("Loaded REST server config: {cfg:?}");

    let limiter = Arc::new(get_limiter(&cfg).await);

    let app: Router = Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(RateLimitLayer::new(limiter));

    let addr: SocketAddr = cfg.bind_socket_addr();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    log::info!("Starting REST server with rate limiting on: {}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    log::info!("Shutdown signal received.");
}

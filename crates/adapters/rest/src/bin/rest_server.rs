use std::{net::SocketAddr, sync::Arc};

use axum::{Router, routing::get};
use tokio::signal;

use rate_limit::{event_sink::NoopEventSink, models::RateLimitKey};
#[cfg(feature = "hybrid_limiter")]
use rate_limit_hybrid::{DistributedFailurePolicy, HybridLimiter, HybridLimiterConfig};

#[cfg(feature = "hybrid_limiter")]
use rest::config::HybridFailurePolicyMode;
use rest::{config::RestServerConfig, layer::RateLimitLayer};

#[cfg(feature = "in_memory_limiter")]
use rate_limit::{
    factory::hierarchical_limiter, hierarchical_limiter::HierarchicalLimiter, models::RateLimit,
};
#[cfg(feature = "in_memory_limiter")]
async fn get_limiter(cfg: &RestServerConfig) -> HierarchicalLimiter<RateLimitKey, NoopEventSink> {
    log::info!("Configuring in-memory hierarchical limiter.");
    let sink = NoopEventSink;
    let limits = cfg
        .require_limits()
        .expect("in_memory_limiter requires [limits] in the config");
    let global_limit = RateLimit::per_second(limits.global_per_second)
        .expect("global_per_second must be non-zero");
    let per_key_limit = RateLimit::per_second(limits.per_key_per_second)
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

    let distributed = cfg.require_distributed().expect(
        "distributed_limiter requires [distributed] with namespace/window_ms/max in config",
    );

    DistributedKeyedLimiter::new(
        Arc::new(backend),
        distributed.namespace.as_str(),
        LimitSpec {
            window: Duration::from_millis(distributed.window_ms),
            max: distributed.max,
        },
        sink,
        cfg.instrumentation_level(),
    )
}

#[cfg(feature = "hybrid_limiter")]
use {
    rate_limit::{
        factory::hierarchical_limiter, hierarchical_limiter::HierarchicalLimiter, models::RateLimit,
    },
    rate_limit_distributed::DistributedKeyedLimiter,
    rate_limit_hybrid::HybridLimiter,
    state_backend::{LimitSpec, RedisBackend},
    std::time::Duration,
};
#[cfg(feature = "hybrid_limiter")]
async fn get_limiter(
    cfg: &RestServerConfig,
) -> HybridLimiter<
    HierarchicalLimiter<RateLimitKey, NoopEventSink>,
    DistributedKeyedLimiter<RateLimitKey, state_backend::RedisBackend, NoopEventSink>,
    RateLimitKey,
    NoopEventSink,
> {
    log::info!("Configuring hybrid limiter.");

    let limits = cfg
        .require_limits()
        .expect("hybrid_limiter requires [limits] in the config");
    let global_limit = RateLimit::per_second(limits.global_per_second)
        .expect("global_per_second must be non-zero");
    let per_key_limit = RateLimit::per_second(limits.per_key_per_second)
        .expect("per_key_per_second must be non-zero");
    let local = hierarchical_limiter(
        global_limit.to_quota(),
        per_key_limit.to_quota(),
        NoopEventSink,
        cfg.instrumentation_level(),
    );

    let distributed_cfg = cfg
        .require_distributed()
        .expect("hybrid_limiter requires [distributed] in the config");
    let backend = RedisBackend::connect_from_env().await.expect("redis");
    let distributed = DistributedKeyedLimiter::new(
        Arc::new(backend),
        distributed_cfg.namespace.as_str(),
        LimitSpec {
            window: Duration::from_millis(distributed_cfg.window_ms),
            max: distributed_cfg.max,
        },
        NoopEventSink,
        cfg.instrumentation_level(),
    );

    let policy = match cfg.hybrid.as_ref().and_then(|h| h.failure_policy) {
        Some(HybridFailurePolicyMode::FailClosed) => DistributedFailurePolicy::FailClosed,
        _ => DistributedFailurePolicy::FailOpen,
    };

    HybridLimiter::new(
        HybridLimiterConfig::new(local, distributed, NoopEventSink)
            .with_instrumentation(cfg.instrumentation_level())
            .with_failure_policy(policy),
    )
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cfg = RestServerConfig::load();
    cfg.validate_for_enabled_feature()
        .unwrap_or_else(|err| panic!("invalid REST server config: {err}"));
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

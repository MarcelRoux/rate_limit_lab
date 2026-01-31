use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{Router, routing::get};
use tokio::signal;

use rate_limit::{event_sink::NoopEventSink, models::InstrumentationLevel};
use rate_limit_distributed::DistributedKeyedLimiter;
use state_backend::{LimitSpec, RedisBackend};

use rest::layer::RateLimitLayer;

type Limiter = DistributedKeyedLimiter<String, RedisBackend, NoopEventSink>;

#[tokio::main]
async fn main() {
    let backend = RedisBackend::connect_from_env().await.expect("redis");

    let limiter = Limiter::new(
        Arc::new(backend),
        "rest",
        LimitSpec {
            window: Duration::from_secs(1),
            max: 1000,
        },
        NoopEventSink,
        InstrumentationLevel::Off,
    );

    let app: Router = Router::new()
        .route("/", get(|| async { "ok" }))
        .layer(RateLimitLayer::new(Arc::new(limiter)));

    let addr: SocketAddr = "127.0.0.1:3001".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("REST rate-limited server listening on {}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = signal::ctrl_c().await;
        })
        .await
        .unwrap();
}

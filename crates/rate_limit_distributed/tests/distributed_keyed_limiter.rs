#![cfg(feature = "redis-tests")]
use std::{sync::Arc, time::Duration};

use rate_limit::{
    event_sink::NoopEventSink,
    models::{Decision, InstrumentationLevel},
};
use rate_limit_distributed::DistributedKeyedLimiter;
use state_backend::LimitSpec;

// --------
// Redis integration test (feature-gated)
// --------
#[tokio::test]
async fn redis_allows_then_denies_within_window() {
    use state_backend::RedisBackend;

    let backend = RedisBackend::connect_from_env()
        .await
        .expect("connect redis backend");

    let limiter = DistributedKeyedLimiter::<String, _, _>::new(
        Arc::new(backend),
        "test",
        LimitSpec {
            window: Duration::from_secs(1),
            max: 1,
        },
        NoopEventSink,
        InstrumentationLevel::Off,
    );

    let key = "user1".to_string();

    assert_eq!(limiter.check(&key).await, Decision::Allow);

    match limiter.check(&key).await {
        Decision::Deny { retry_after } => {
            assert!(retry_after > Duration::ZERO);
            assert!(retry_after <= Duration::from_secs(1));
        }
        Decision::Allow => panic!("expected deny"),
    }
}

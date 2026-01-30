#![cfg(feature = "redis-tests")]
// Requires Redis running locally or via container.
// Enable with: cargo test -p state_backend --features redis-tests
use std::time::Duration;

use state_backend::{BackendDecision, LimitSpec, RedisBackend, StateBackend};

#[tokio::test]
async fn redis_backend_fixed_window_allows_then_denies() {
    // Requires local Redis:
    // docker run --rm -p 6379:6379 redis:7
    let backend = RedisBackend::connect_from_env()
        .await
        .expect("connect redis backend");

    let ns = "test";
    let key = "user1";

    let limit = LimitSpec {
        window: Duration::from_secs(1),
        max: 1,
    };

    // First request should allow.
    let d1 = backend.check(ns, key, limit).await.unwrap();
    assert_eq!(d1, BackendDecision::Allow);

    // Second request within same window should deny.
    let d2 = backend.check(ns, key, limit).await.unwrap();
    match d2 {
        BackendDecision::Deny { retry_after } => {
            assert!(retry_after > Duration::ZERO);
            assert!(retry_after <= Duration::from_secs(1));
        }
        BackendDecision::Allow => panic!("expected deny"),
    }
}

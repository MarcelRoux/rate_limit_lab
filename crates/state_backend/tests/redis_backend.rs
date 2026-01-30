#![cfg(feature = "redis-tests")]
// Requires Redis running locally or via container.
// Enable with: cargo test -p state_backend --features redis-tests
use std::time::Duration;

use state_backend::{BackendDecision, LimitSpec, RedisBackend, StateBackend};

#[tokio::test]
async fn redis_backend_fixed_window_allows_then_denies() {
    // Requires local Redis.
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

#[tokio::test]
async fn redis_backend_fixed_window_key_isolation() {
    // Requires local Redis.
    let backend = RedisBackend::connect_from_env()
        .await
        .expect("connect redis backend");

    let ns = "test_key_isolation";
    let key1 = "user1";
    let key2 = "user2";

    let limit = LimitSpec {
        window: Duration::from_secs(1),
        max: 1,
    };

    let d1 = backend.check(ns, key1, limit).await.unwrap();
    assert_eq!(d1, BackendDecision::Allow);

    let d2 = backend.check(ns, key2, limit).await.unwrap();
    assert_eq!(d2, BackendDecision::Allow);

    let d3 = backend.check(ns, key1, limit).await.unwrap();
    match d3 {
        BackendDecision::Deny { retry_after } => {
            assert!(retry_after > Duration::ZERO);
            assert!(retry_after <= Duration::from_secs(1));
        }
        BackendDecision::Allow => panic!("expected deny on key1"),
    }

    let d4 = backend.check(ns, key2, limit).await.unwrap();
    match d4 {
        BackendDecision::Deny { retry_after } => {
            assert!(retry_after > Duration::ZERO);
            assert!(retry_after <= Duration::from_secs(1));
        }
        BackendDecision::Allow => panic!("expected deny on key2"),
    }
}

#[tokio::test]
async fn redis_backend_fixed_window_rollover() {
    // Requires local Redis.
    let backend = RedisBackend::connect_from_env()
        .await
        .expect("connect redis backend");

    let ns = "test_rollover";
    let key = "user1";

    let limit = LimitSpec {
        window: Duration::from_secs(1),
        max: 1,
    };

    let d1 = backend.check(ns, key, limit).await.unwrap();
    assert_eq!(d1, BackendDecision::Allow);

    // Wait for window to expire.
    tokio::time::sleep(Duration::from_secs(1)).await;

    let d2 = backend.check(ns, key, limit).await.unwrap();
    assert_eq!(d2, BackendDecision::Allow);
}

use std::time::Duration;

use core::factory::hierarchical_limiter;
use core::models::{Decision, InstrumentationLevel, RateLimit, RateLimitKey};

mod support {
    pub mod recording_event_sink;
}
use support::recording_event_sink::RecordingEventSink;

#[test]
fn hierarchical_allows_if_both_pass() {
    let sink = RecordingEventSink::default();
    let global_limit = RateLimit::per_second(2).unwrap();
    let key_limit = RateLimit::per_second(1).unwrap();

    let limiter = hierarchical_limiter(
        global_limit.to_quota(),
        key_limit.to_quota(),
        sink.clone(),
        InstrumentationLevel::Off,
    );

    let key = RateLimitKey("user1".into());

    // First request: allowed by both.
    assert_eq!(limiter.check(&key), Decision::Allow);

    // Second request: denied by per-key limiter.
    match limiter.check(&key) {
        Decision::Deny { retry_after } => {
            assert!(retry_after > Duration::ZERO);
            assert!(retry_after <= Duration::from_secs(1));
        }
        Decision::Allow => panic!("expected deny due to per-key rate limit"),
    }
}

#[test]
fn hierarchical_denies_if_global_exceeded() {
    let sink = RecordingEventSink::default();
    let global_limit = RateLimit::per_second(1).unwrap();
    let key_limit = RateLimit::per_second(5).unwrap();

    let limiter = hierarchical_limiter(
        global_limit.to_quota(),
        key_limit.to_quota(),
        sink.clone(),
        InstrumentationLevel::Off,
    );

    let key1 = RateLimitKey("user1".into());
    let key2 = RateLimitKey("user2".into());

    // First request for user1 consumes global.
    assert_eq!(limiter.check(&key1), Decision::Allow);

    // user2 should be denied due to global quota.
    match limiter.check(&key2) {
        Decision::Deny { retry_after } => assert!(retry_after > Duration::ZERO),
        Decision::Allow => panic!("expected global denial"),
    }
}

#[test]
fn hierarchical_denies_if_key_exceeded() {
    let sink = RecordingEventSink::default();
    let global_limit = RateLimit::per_second(5).unwrap();
    let key_limit = RateLimit::per_second(1).unwrap();

    let limiter = hierarchical_limiter(
        global_limit.to_quota(),
        key_limit.to_quota(),
        sink.clone(),
        InstrumentationLevel::Off,
    );

    let key = RateLimitKey("user1".into());

    // First request for user1 consumes key.
    assert_eq!(limiter.check(&key), Decision::Allow);

    // user2 should be denied due to key quota.
    match limiter.check(&key) {
        Decision::Deny { retry_after } => assert!(retry_after > Duration::ZERO),
        Decision::Allow => panic!("expected key-level denial"),
    }
}

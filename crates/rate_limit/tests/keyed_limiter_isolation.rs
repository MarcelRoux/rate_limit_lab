use rate_limit::event_sink::NoopEventSink;
use rate_limit::factory::keyed_limiter;
use rate_limit::models::{Decision, InstrumentationLevel, RateLimit, RateLimitKey};

#[test]
fn keyed_limit_isolated_per_key() {
    let limit = RateLimit::per_second(1).unwrap();
    let sink = NoopEventSink;
    let limiter =
        keyed_limiter::<RateLimitKey, _>(limit.to_quota(), sink, InstrumentationLevel::Off);

    let a = RateLimitKey("a".into());
    let b = RateLimitKey("b".into());

    assert_eq!(limiter.check(&a), Decision::Allow);
    assert!(matches!(limiter.check(&a), Decision::Deny { .. }));

    assert_eq!(limiter.check(&b), Decision::Allow);
}

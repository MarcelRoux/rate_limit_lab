use std::num::NonZeroU32;
use std::thread::sleep;
use std::time::Duration;

use governor::Quota;

use core::direct_limiter::DirectLimiter;
use core::event_sink::NoopEventSink;
use core::factory::direct_limiter;
use core::models::Decision;
use core::models::InstrumentationLevel;
use core::models::RateLimit;

#[test]
fn zero_rate_is_rejected() {
    assert!(RateLimit::per_second(0).is_none());
    assert!(RateLimit::per_minute(0).is_none());
}

#[test]
fn limiter_refills_after_interval() {
    let limit = RateLimit::per_second(1).unwrap();
    let sink = NoopEventSink;
    let limiter = direct_limiter(limit.to_quota(), sink, InstrumentationLevel::Off);

    assert_eq!(limiter.check(), Decision::Allow);
    assert!(matches!(limiter.check(), Decision::Deny { .. }));

    sleep(Duration::from_secs(1));

    assert_eq!(limiter.check(), Decision::Allow);
}

#[test]
fn allows_within_limit() {
    let quota = Quota::per_second(NonZeroU32::new(2).unwrap());
    let limiter = DirectLimiter::new(quota, NoopEventSink, InstrumentationLevel::Off);

    assert_eq!(limiter.check(), Decision::Allow);
    assert_eq!(limiter.check(), Decision::Allow);
}

#[test]
fn denies_when_exceeded() {
    let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
    let limiter = DirectLimiter::new(quota, NoopEventSink, InstrumentationLevel::Off);

    assert_eq!(limiter.check(), Decision::Allow);

    match limiter.check() {
        Decision::Deny { retry_after } => {
            assert!(retry_after > Duration::ZERO);
        }
        Decision::Allow => panic!("expected denial"),
    }
}

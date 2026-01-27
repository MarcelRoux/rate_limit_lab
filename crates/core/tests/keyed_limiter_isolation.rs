use core::factory::keyed_limiter;
use core::models::{RateLimit, RateLimitKey};

#[test]
fn keyed_limit_isolated_per_key() {
    let limit = RateLimit::per_second(1).unwrap();
    let limiter = keyed_limiter::<RateLimitKey>(limit);

    let a = RateLimitKey("a".into());
    let b = RateLimitKey("b".into());

    assert!(limiter.check_key(&a).is_ok());
    assert!(limiter.check_key(&a).is_err());

    assert!(limiter.check_key(&b).is_ok());
}

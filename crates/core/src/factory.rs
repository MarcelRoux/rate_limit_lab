use governor::RateLimiter;

use crate::models::{DirectRateLimiter, KeyedRateLimiter, RateLimit};

/// Factory for direct (non-keyed) rate limiters.
pub fn direct_limiter(limit: RateLimit) -> DirectRateLimiter {
    RateLimiter::direct(limit.to_quota())
}

/// Factory for keyed rate limiters.
pub fn keyed_limiter<K>(limit: RateLimit) -> KeyedRateLimiter<K>
where
    K: Eq + std::hash::Hash + Clone,
{
    RateLimiter::keyed(limit.to_quota())
}

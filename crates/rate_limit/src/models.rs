use std::num::NonZeroU32;

use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed, keyed::DashMapStateStore},
};

/// Public types alias for clarity.
pub type DirectRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;
pub type KeyedRateLimiter<K> = RateLimiter<K, DashMapStateStore<K>, DefaultClock, NoOpMiddleware>;

/// Domain-level rate limit definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RateLimit {
    PerSecond(NonZeroU32),
    PerMinute(NonZeroU32),
}

impl RateLimit {
    pub fn per_second(n: u32) -> Option<Self> {
        NonZeroU32::new(n).map(RateLimit::PerSecond)
    }

    pub fn per_minute(n: u32) -> Option<Self> {
        NonZeroU32::new(n).map(RateLimit::PerMinute)
    }

    pub fn to_quota(self) -> Quota {
        match self {
            RateLimit::PerSecond(n) => Quota::per_second(n),
            RateLimit::PerMinute(n) => Quota::per_minute(n),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RateLimitKey(pub String);

impl AsRef<str> for RateLimitKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// What operation is being evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    RateLimitCheck,
}

/// The outcome of executing an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny { retry_after: std::time::Duration },
}

/// Events emitted by the core during execution.
/// These are *not metrics* — they are structural signals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    OperationStarted {
        operation: Operation,
    },
    OperationCompleted {
        operation: Operation,
        decision: Decision,
    },
}

/// Instrumentation levels to toggle the granularity of emissions for metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentationLevel {
    Off,
    Basic,
    Full,
}

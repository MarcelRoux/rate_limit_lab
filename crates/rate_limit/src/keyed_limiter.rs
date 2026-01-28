use governor::clock::{Clock, DefaultClock};
use governor::{Quota, RateLimiter};

use crate::event_sink::EventSink;
use crate::models::{Decision, Event, InstrumentationLevel, KeyedRateLimiter, Operation};

pub struct KeyedLimiter<K, S>
where
    K: Eq + std::hash::Hash + Clone,
    S: EventSink,
{
    limiter: KeyedRateLimiter<K>,
    sink: S,
    instrumentation: InstrumentationLevel,
}

impl<K, S> KeyedLimiter<K, S>
where
    K: Eq + std::hash::Hash + Clone,
    S: EventSink,
{
    pub fn new(quota: Quota, sink: S, instrumentation: InstrumentationLevel) -> Self {
        Self {
            limiter: RateLimiter::keyed(quota),
            sink,
            instrumentation,
        }
    }

    pub fn check(&self, key: &K) -> Decision {
        let operation = Operation::RateLimitCheck;

        if self.instrumentation == InstrumentationLevel::Full {
            self.sink.emit(Event::OperationStarted { operation });
        }

        let decision = match self.limiter.check_key(key) {
            Ok(_) => Decision::Allow,
            Err(negative) => {
                let retry_after = negative.wait_time_from(DefaultClock::default().now());
                Decision::Deny { retry_after }
            }
        };

        if self.instrumentation != InstrumentationLevel::Off {
            self.sink.emit(Event::OperationCompleted {
                operation,
                decision: decision.clone(),
            });
        }

        decision
    }
}

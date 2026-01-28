use governor::clock::{Clock, DefaultClock};
use governor::{Quota, RateLimiter};

use crate::event_sink::EventSink;
use crate::models::{Decision, DirectRateLimiter, Event, InstrumentationLevel, Operation};

pub struct DirectLimiter<S: EventSink> {
    limiter: DirectRateLimiter,
    sink: S,
    instrumentation: InstrumentationLevel,
}

impl<S: EventSink> DirectLimiter<S> {
    pub fn new(quota: Quota, sink: S, instrumentation: InstrumentationLevel) -> Self {
        Self {
            limiter: RateLimiter::direct(quota),
            sink,
            instrumentation,
        }
    }

    pub fn check(&self) -> Decision {
        let operation = Operation::RateLimitCheck;

        if self.instrumentation == InstrumentationLevel::Full {
            self.sink.emit(Event::OperationStarted { operation });
        }

        let decision = match self.limiter.check() {
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

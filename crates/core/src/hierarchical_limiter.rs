use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

use governor::Quota;

use crate::direct_limiter::DirectLimiter;
use crate::event_sink::EventSink;
use crate::keyed_limiter::KeyedLimiter;
use crate::models::{Decision, Event, InstrumentationLevel, Operation};

/// Multi-level limiter : global + per-key.
pub struct HierarchicalLimiter<K, S>
where
    K: Eq + Hash + Clone,
    S: EventSink,
{
    global: DirectLimiter<Arc<S>>,
    per_key: KeyedLimiter<K, Arc<S>>,
    sink: Arc<S>,
    instrumentation: InstrumentationLevel,
}

impl<K, S> HierarchicalLimiter<K, S>
where
    K: Eq + Hash + Clone,
    S: EventSink,
{
    pub fn new(
        global_quota: Quota,
        key_quota: Quota,
        sink: S,
        instrumentation: InstrumentationLevel,
    ) -> Self {
        let sink = Arc::new(sink);
        Self {
            global: DirectLimiter::new(global_quota, sink.clone(), instrumentation),
            per_key: KeyedLimiter::new(key_quota, sink.clone(), instrumentation),
            sink,
            instrumentation,
        }
    }

    /// Checks the hierarchical limiter for a given key.
    /// Returns a decision based on the AND of the decisions returned by the global and per_key limiters.
    /// Returns the maximum retry after value observed in the event of a Deny.
    pub fn check(&self, key: &K) -> Decision {
        let operation = Operation::RateLimitCheck;

        let global_decision = self.global.check();
        let key_decision = self.per_key.check(key);

        let decision = match (&global_decision, &key_decision) {
            (Decision::Allow, Decision::Allow) => Decision::Allow,
            _ => {
                let retry_after = match (&global_decision, &key_decision) {
                    (Decision::Deny { retry_after: g }, Decision::Deny { retry_after: k }) => {
                        Some(*g.max(k))
                    }
                    (Decision::Deny { retry_after: g }, _) => Some(*g),
                    (_, Decision::Deny { retry_after: k }) => Some(*k),
                    _ => None,
                }
                .unwrap_or(Duration::ZERO);

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

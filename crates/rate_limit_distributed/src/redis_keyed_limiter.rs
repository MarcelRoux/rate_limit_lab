use std::{hash::Hash, marker::PhantomData, sync::Arc, time::Duration};

use rate_limit::{
    event_sink::EventSink,
    models::{Decision, Event, InstrumentationLevel, Operation},
};
use state_backend::{BackendDecision, LimitSpec, StateBackend};

/// Redis-backed keyed rate limiter.
///
/// This limiter enforces per-key rate limits using a shared Redis backend.
/// It mirrors the API shape of the in-memory `KeyedLimiter<K, _>` while
/// delegating all state management to Redis.
///
/// # Type parameter `K`
///
/// The generic parameter `K` represents the *logical key type* used by callers
/// (e.g. `RateLimitKey`, `String`, or a domain-specific identifier).
///
/// Although `K` does not appear in the struct's fields, it is intentionally
/// preserved as part of the type signature to:
///
/// - Maintain API symmetry with in-memory keyed limiters
/// - Provide compile-time type safety across adapters
/// - Prevent accidental mixing of incompatible key types
///
/// The key value is converted to a Redis-compatible representation
/// (typically `&str`) at the call boundary and is not stored directly.
///
/// # PhantomData
///
/// Because `K` does not affect the runtime layout of this struct, it is tracked
/// via `PhantomData<K>`. This explicitly signals to the compiler that `K` is a
/// meaningful part of the type and participates in variance and drop checking,
/// while incurring zero runtime cost.
///
/// # Design note
///
/// This separation allows the distributed limiter to remain protocol-agnostic
/// and backend-focused, while still integrating cleanly with higher-level
/// components (REST, gRPC, CLI) that operate on typed keys.
pub struct RedisKeyedLimiter<K, S>
where
    K: Eq + Hash + Clone,
    S: EventSink,
{
    backend: Arc<dyn StateBackend>,
    namespace: String,
    limit: LimitSpec,
    sink: S,
    instrumentation: InstrumentationLevel,
    _marker: PhantomData<K>,
}

impl<K, S> RedisKeyedLimiter<K, S>
where
    K: Eq + Hash + Clone,
    S: EventSink,
{
    pub fn new(
        backend: Arc<dyn StateBackend>,
        namespace: impl Into<String>,
        limit: LimitSpec,
        sink: S,
        instrumentation: InstrumentationLevel,
    ) -> Self {
        Self {
            backend,
            namespace: namespace.into(),
            limit,
            sink,
            instrumentation,
            _marker: PhantomData,
        }
    }

    pub async fn check(&self, key: &K) -> Decision
    where
        K: ToString,
    {
        let operation = Operation::RateLimitCheck;

        let backend_result = self
            .backend
            .check(&self.namespace, &key.to_string(), self.limit)
            .await;

        let decision = match backend_result {
            Ok(BackendDecision::Allow) => Decision::Allow,
            Ok(BackendDecision::Deny { retry_after }) => Decision::Deny { retry_after },
            Err(_) => {
                // M3.1 policy placeholder:
                // failure policy (fail-open vs fail-closed) is explored in M3.3.
                Decision::Deny {
                    retry_after: Duration::from_millis(50),
                }
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

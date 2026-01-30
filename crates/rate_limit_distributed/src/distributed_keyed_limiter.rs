use std::{marker::PhantomData, sync::Arc, time::Duration};

use rate_limit::{
    event_sink::EventSink,
    models::{Decision, Event, InstrumentationLevel, Operation},
};
use state_backend::{BackendDecision, LimitSpec, StateBackend};

const BACKEND_ERROR_RETRY_AFTER: Duration = Duration::from_millis(50);

/// Distributed keyed rate limiter backed by a state backend.
///
/// This limiter enforces **per-key** rate limits using a shared external state store.
/// It mirrors the API shape of the in-memory `KeyedLimiter<K, _>` while delegating
/// all counter/bucket state to a `StateBackend` implementation.
///
/// Although the initial backend is Redis (via `state_backend::RedisBackend`), this
/// type is intentionally written against the **backend trait** so that other
/// backends (e.g. Valkey) can be added later without changing the limiter API.
///
/// ## Static dispatch (generic backend)
///
/// The backend is modeled as a generic type parameter `B: StateBackend` rather than
/// `dyn StateBackend`.
///
/// This has two practical benefits:
///
/// - **Evaluation fidelity:** avoids trait-object / vtable overhead on the hot path,
///   which matters when you are measuring throughput and tail latency.
/// - **Testability:** allows lightweight, deterministic fake backends in unit tests
///   without requiring Redis to be running.
///
/// In practice, the limiter holds the backend behind an `Arc<B>` so it can be
/// cheaply cloned and shared across tasks.
///
/// ## Type parameter `K`
///
/// `K` represents the *logical key type* used by callers (e.g. `RateLimitKey`,
/// `String`, or a domain identifier). The key is converted at the call boundary
/// into a backend-compatible string (currently via `ToString`) and is not stored.
///
/// `K` is tracked via `PhantomData<K>` to keep type safety and API symmetry with
/// in-memory keyed limiters while incurring zero runtime cost.
///
/// ## Instrumentation
///
/// This limiter emits `Event::OperationCompleted` when instrumentation is enabled.
/// Detailed lifecycle events and failure-policy experimentation are explored in M3.3.
/// In M3.1, backend errors currently map to a deny decision with a short retry delay
/// (placeholder policy).
///
/// ## Notes
///
/// - Namespacing (`namespace`) isolates independent experiments / policies in the backend.
/// - `LimitSpec` defines the backend enforcement parameters (window + max).
/// - This limiter is protocol-agnostic; protocol adapters (REST/gRPC) should depend
///   only on the limiter decision model, not backend-specific concerns.
pub struct DistributedKeyedLimiter<K, B, S>
where
    K: AsRef<str>,
    B: StateBackend,
    S: EventSink,
{
    backend: Arc<B>,
    namespace: String,
    limit: LimitSpec,
    sink: S,
    instrumentation: InstrumentationLevel,
    _marker: PhantomData<K>,
}

impl<K, B, S> DistributedKeyedLimiter<K, B, S>
where
    K: AsRef<str>,
    B: StateBackend,
    S: EventSink,
{
    pub fn new(
        backend: Arc<B>,
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
            .check(&self.namespace, key.as_ref(), self.limit)
            .await;

        let decision = match backend_result {
            Ok(BackendDecision::Allow) => Decision::Allow,
            Ok(BackendDecision::Deny { retry_after }) => Decision::Deny { retry_after },
            Err(_) => {
                // M3.1 policy placeholder:
                // failure policy (fail-open vs fail-closed) is explored in M3.3.
                Decision::Deny {
                    retry_after: BACKEND_ERROR_RETRY_AFTER,
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

//! Hybrid limiter crate for composing local and distributed checks (M3.2).
//! This module currently hosts the orchestration skeleton; full behaviors land in later tasks.

use std::{future::Future, hash::Hash, marker::PhantomData, pin::Pin, sync::Arc};

use futures::FutureExt;

use rate_limit::{
    event_sink::EventSink,
    hierarchical_limiter::HierarchicalLimiter,
    models::{Decision, Event, InstrumentationLevel, Operation},
};
use rate_limit_distributed::{
    BACKEND_ERROR_RETRY_AFTER, DistributedCheckOutcome, DistributedKeyedLimiter,
};
use state_backend::StateBackend;

/// Policy for handling distributed backend failures in M3.2.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributedFailurePolicy {
    #[default]
    FailOpen,
    FailClosed,
}

type DistributedOutcomeFuture<'a> =
    Pin<Box<dyn Future<Output = DistributedCheckOutcome> + Send + 'a>>;

/// Synchronous/local limiter contract used by the hybrid orchestrator.
pub trait LocalLimiter<K>: Send + Sync {
    fn check(&self, key: &K) -> Decision;
}

/// Asynchronous/distributed limiter contract used by the hybrid orchestrator.
pub trait DistributedLimiter<K>: Send + Sync {
    fn check_outcome<'a>(&'a self, key: &'a K) -> DistributedOutcomeFuture<'a>
    where
        Self: 'a;
}

/// Configuration for constructing a hybrid limiter.
pub struct HybridLimiterConfig<L, D, S> {
    /// Local limiter implementation (sync).
    pub local: L,
    /// Distributed limiter implementation (async).
    pub distributed: D,
    /// Event sink used by the hybrid limiter for instrumentation.
    pub sink: S,
    /// Instrumentation level for the hybrid limiter.
    pub instrumentation: InstrumentationLevel,
    /// Failure policy to apply when the distributed backend errors.
    ///
    /// The returning `Decision` from `DistributedKeyedLimiter::check` already hides
    /// backend failures, so the policy cannot be observed yet. It is retained here
    /// as a placeholder while docs/backlog/0004-distributed-keyed-limiter-revision.md
    /// tracks the translation work needed on the distributed crate.
    pub failure_policy: DistributedFailurePolicy,
}

impl<L, D, S> HybridLimiterConfig<L, D, S> {
    /// Create a new configuration with instrumentation set to `Off`.
    pub fn new(local: L, distributed: D, sink: S) -> Self {
        Self {
            local,
            distributed,
            sink,
            instrumentation: InstrumentationLevel::Off,
            failure_policy: DistributedFailurePolicy::default(),
        }
    }

    /// Override the instrumentation level.
    pub fn with_instrumentation(mut self, level: InstrumentationLevel) -> Self {
        self.instrumentation = level;
        self
    }

    /// Override the distributed failure policy.
    pub fn with_failure_policy(mut self, policy: DistributedFailurePolicy) -> Self {
        self.failure_policy = policy;
        self
    }
}

/// Hybrid limiter skeleton that orchestrates local + distributed checks.
pub struct HybridLimiter<L, D, K, S> {
    local: Arc<L>,
    distributed: Arc<D>,
    sink: S,
    instrumentation: InstrumentationLevel,
    failure_policy: DistributedFailurePolicy,
    _marker: PhantomData<K>,
}

impl<L, D, K, S> HybridLimiter<L, D, K, S> {
    /// Construct a new hybrid limiter from the provided configuration.
    pub fn new(config: HybridLimiterConfig<L, D, S>) -> Self {
        Self {
            local: Arc::new(config.local),
            distributed: Arc::new(config.distributed),
            sink: config.sink,
            instrumentation: config.instrumentation,
            failure_policy: config.failure_policy,
            _marker: PhantomData,
        }
    }

    /// Returns the configured instrumentation level.
    pub fn instrumentation(&self) -> InstrumentationLevel {
        self.instrumentation
    }

    /// Returns the configured failure policy.
    pub fn failure_policy(&self) -> DistributedFailurePolicy {
        self.failure_policy
    }
}

impl<L, D, K, S> HybridLimiter<L, D, K, S>
where
    L: LocalLimiter<K> + 'static,
    D: DistributedLimiter<K> + 'static,
    S: EventSink,
    K: Clone + Send + Sync + 'static,
{
    /// Async entry point for running a hybrid rate limit check.
    pub async fn check(&self, key: &K) -> Decision {
        let distributed = self.distributed.clone();
        let key_for_distributed = key.clone();
        let distributed_task =
            tokio::spawn(async move { distributed.check_outcome(&key_for_distributed).await });

        let local_decision = self.local.check(key);

        if let Decision::Deny { .. } = &local_decision {
            if let Some(Ok(distributed_outcome)) = distributed_task.now_or_never() {
                let decision = combine_decisions(
                    local_decision,
                    map_distributed_outcome_to_decision(distributed_outcome, self.failure_policy),
                );
                self.emit_operation_completed(&decision);
                return decision;
            }

            self.emit_operation_completed(&local_decision);
            return local_decision;
        }

        let distributed_outcome = match distributed_task.await {
            Ok(outcome) => outcome,
            Err(_) => DistributedCheckOutcome::BackendError {
                retry_after: BACKEND_ERROR_RETRY_AFTER,
            },
        };
        let distributed_decision =
            map_distributed_outcome_to_decision(distributed_outcome, self.failure_policy);
        let decision = combine_decisions(local_decision, distributed_decision);

        self.emit_operation_completed(&decision);
        decision
    }

    fn emit_operation_completed(&self, decision: &Decision) {
        if self.instrumentation == InstrumentationLevel::Off {
            return;
        }

        // The current `rate_limit::Event` shape lacks backend-error/policy metadata,
        // so this is the minimal translation we can emit today. The backlog entry
        // docs/backlog/0003-rate-limit-model-reversion.md captures the richer events
        // we want once the core model evolves.
        self.sink.emit(Event::OperationCompleted {
            operation: Operation::RateLimitCheck,
            decision: decision.clone(),
        });
    }
}

fn combine_decisions(first: Decision, second: Decision) -> Decision {
    match (first, second) {
        (Decision::Allow, Decision::Allow) => Decision::Allow,
        (Decision::Deny { retry_after: r1 }, Decision::Deny { retry_after: r2 }) => {
            Decision::Deny {
                retry_after: r1.max(r2),
            }
        }
        (Decision::Deny { retry_after }, _) | (_, Decision::Deny { retry_after }) => {
            Decision::Deny { retry_after }
        }
    }
}

fn map_distributed_outcome_to_decision(
    outcome: DistributedCheckOutcome,
    policy: DistributedFailurePolicy,
) -> Decision {
    match outcome {
        DistributedCheckOutcome::Allow => Decision::Allow,
        DistributedCheckOutcome::Deny { retry_after } => Decision::Deny { retry_after },
        DistributedCheckOutcome::BackendError { retry_after } => match policy {
            DistributedFailurePolicy::FailOpen => Decision::Allow,
            DistributedFailurePolicy::FailClosed => Decision::Deny { retry_after },
        },
    }
}

impl<K, S> LocalLimiter<K> for HierarchicalLimiter<K, S>
where
    K: Eq + Hash + Clone + Send + Sync,
    S: EventSink + Send + Sync + 'static,
{
    fn check(&self, key: &K) -> Decision {
        HierarchicalLimiter::check(self, key)
    }
}

impl<K, B, S> DistributedLimiter<K> for DistributedKeyedLimiter<K, B, S>
where
    K: AsRef<str> + Send + Sync + 'static,
    B: StateBackend + Send + Sync + 'static,
    S: EventSink + Send + Sync + 'static,
{
    fn check_outcome<'a>(&'a self, key: &'a K) -> DistributedOutcomeFuture<'a>
    where
        Self: 'a,
    {
        Box::pin(DistributedKeyedLimiter::check_outcome(self, key))
    }
}

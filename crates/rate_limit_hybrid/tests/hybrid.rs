use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::future::BoxFuture;
use tokio::time::sleep;

use rate_limit::{
    event_sink::EventSink,
    models::{Decision, Event, InstrumentationLevel},
};
use rate_limit_distributed::DistributedCheckOutcome;

use rate_limit_hybrid::{
    DistributedFailurePolicy, DistributedLimiter, HybridLimiter, HybridLimiterConfig, LocalLimiter,
};

#[derive(Clone)]
struct RecordingEventSink {
    events: Arc<Mutex<Vec<Event>>>,
}

impl RecordingEventSink {
    fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn events(&self) -> Vec<Event> {
        self.events.lock().unwrap().clone()
    }
}

impl Default for RecordingEventSink {
    fn default() -> Self {
        Self::new()
    }
}

impl EventSink for RecordingEventSink {
    fn emit(&self, event: Event) {
        self.events.lock().unwrap().push(event);
    }
}

#[derive(Clone)]
struct FakeLocal {
    decision: Decision,
    order: Arc<Mutex<Vec<&'static str>>>,
    delay: Duration,
}

impl FakeLocal {
    fn new(decision: Decision, order: Arc<Mutex<Vec<&'static str>>>, delay: Duration) -> Self {
        Self {
            decision,
            order,
            delay,
        }
    }
}

impl LocalLimiter<String> for FakeLocal {
    fn check(&self, _key: &String) -> Decision {
        self.order.lock().unwrap().push("local-start");
        if self.delay > Duration::ZERO {
            std::thread::sleep(self.delay);
        }
        self.decision.clone()
    }
}

#[derive(Clone)]
struct FakeDistributed {
    outcome: DistributedCheckOutcome,
    delay: Duration,
    order: Arc<Mutex<Vec<&'static str>>>,
}

impl FakeDistributed {
    fn new(
        outcome: DistributedCheckOutcome,
        delay: Duration,
        order: Arc<Mutex<Vec<&'static str>>>,
    ) -> Self {
        Self {
            outcome,
            delay,
            order,
        }
    }
}

impl DistributedLimiter<String> for FakeDistributed {
    fn check_outcome<'a>(&'a self, _key: &'a String) -> BoxFuture<'a, DistributedCheckOutcome>
    where
        Self: 'a,
    {
        let outcome = self.outcome.clone();
        let delay = self.delay;
        let order = self.order.clone();

        order.lock().unwrap().push("distributed-start");

        Box::pin(async move {
            if delay > Duration::ZERO {
                sleep(delay).await;
            }
            outcome
        })
    }
}

fn build_hybrid(
    local: FakeLocal,
    distributed: FakeDistributed,
    sink: RecordingEventSink,
    instrumentation: InstrumentationLevel,
    policy: DistributedFailurePolicy,
) -> HybridLimiter<FakeLocal, FakeDistributed, String, RecordingEventSink> {
    let config = HybridLimiterConfig::new(local, distributed, sink)
        .with_instrumentation(instrumentation)
        .with_failure_policy(policy);
    HybridLimiter::new(config)
}

#[tokio::test]
async fn local_and_distributed_allow() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let local = FakeLocal::new(Decision::Allow, order.clone(), Duration::ZERO);
    let distributed = FakeDistributed::new(DistributedCheckOutcome::Allow, Duration::ZERO, order);
    let sink = RecordingEventSink::default();

    let hybrid = build_hybrid(
        local,
        distributed,
        sink,
        InstrumentationLevel::Off,
        DistributedFailurePolicy::FailOpen,
    );

    assert_eq!(Decision::Allow, hybrid.check(&"key".to_string()).await);
}

#[tokio::test]
async fn local_denies_immediately() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let local = FakeLocal::new(
        Decision::Deny {
            retry_after: Duration::from_millis(10),
        },
        order.clone(),
        Duration::ZERO,
    );
    let distributed = FakeDistributed::new(DistributedCheckOutcome::Allow, Duration::ZERO, order);
    let sink = RecordingEventSink::default();

    let hybrid = build_hybrid(
        local,
        distributed,
        sink,
        InstrumentationLevel::Off,
        DistributedFailurePolicy::FailOpen,
    );

    assert_eq!(
        Decision::Deny {
            retry_after: Duration::from_millis(10),
        },
        hybrid.check(&"key".to_string()).await
    );
}

#[tokio::test]
async fn distributed_denies_when_local_allows() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let local = FakeLocal::new(Decision::Allow, order.clone(), Duration::ZERO);
    let distributed = FakeDistributed::new(
        DistributedCheckOutcome::Deny {
            retry_after: Duration::from_millis(20),
        },
        Duration::ZERO,
        order,
    );
    let sink = RecordingEventSink::default();

    let hybrid = build_hybrid(
        local,
        distributed,
        sink,
        InstrumentationLevel::Off,
        DistributedFailurePolicy::FailOpen,
    );

    assert_eq!(
        Decision::Deny {
            retry_after: Duration::from_millis(20),
        },
        hybrid.check(&"key".to_string()).await
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn max_retry_after_when_both_deny() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let local = FakeLocal::new(
        Decision::Deny {
            retry_after: Duration::from_millis(10),
        },
        order.clone(),
        Duration::from_millis(50),
    );
    let distributed = FakeDistributed::new(
        DistributedCheckOutcome::Deny {
            retry_after: Duration::from_millis(20),
        },
        Duration::ZERO,
        order.clone(),
    );
    let sink = RecordingEventSink::default();

    let hybrid = build_hybrid(
        local,
        distributed,
        sink,
        InstrumentationLevel::Off,
        DistributedFailurePolicy::FailOpen,
    );

    assert_eq!(
        Decision::Deny {
            retry_after: Duration::from_millis(20),
        },
        hybrid.check(&"key".to_string()).await
    );
}

#[tokio::test]
async fn short_circuits_when_local_denies_before_distributed_completes() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let local = FakeLocal::new(
        Decision::Deny {
            retry_after: Duration::from_millis(10),
        },
        order.clone(),
        Duration::ZERO,
    );
    let distributed = FakeDistributed::new(
        DistributedCheckOutcome::Allow,
        Duration::from_millis(200),
        order,
    );
    let sink = RecordingEventSink::default();

    let hybrid = build_hybrid(
        local,
        distributed,
        sink,
        InstrumentationLevel::Off,
        DistributedFailurePolicy::FailOpen,
    );
    let start = Instant::now();
    let decision = hybrid.check(&"key".to_string()).await;
    let elapsed = start.elapsed();

    assert_eq!(
        Decision::Deny {
            retry_after: Duration::from_millis(10),
        },
        decision
    );
    assert!(elapsed < Duration::from_millis(120));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn option_a2_head_start_reduces_total_wait() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let local = FakeLocal::new(Decision::Allow, order.clone(), Duration::from_millis(120));
    let distributed = FakeDistributed::new(
        DistributedCheckOutcome::Allow,
        Duration::from_millis(120),
        order,
    );
    let sink = RecordingEventSink::default();

    let hybrid = build_hybrid(
        local,
        distributed,
        sink,
        InstrumentationLevel::Off,
        DistributedFailurePolicy::FailOpen,
    );

    let start = Instant::now();
    let decision = hybrid.check(&"key".to_string()).await;
    let elapsed = start.elapsed();

    assert_eq!(Decision::Allow, decision);
    assert!(elapsed < Duration::from_millis(210));
}

#[tokio::test]
async fn distributed_backend_error_obeys_fail_open() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let local = FakeLocal::new(Decision::Allow, order.clone(), Duration::ZERO);
    let distributed = FakeDistributed::new(
        DistributedCheckOutcome::BackendError {
            retry_after: Duration::from_millis(50),
        },
        Duration::ZERO,
        order,
    );
    let sink = RecordingEventSink::default();

    let hybrid = build_hybrid(
        local,
        distributed,
        sink,
        InstrumentationLevel::Off,
        DistributedFailurePolicy::FailOpen,
    );

    assert_eq!(Decision::Allow, hybrid.check(&"key".to_string()).await);
}

#[tokio::test]
async fn distributed_backend_error_obeys_fail_closed() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let local = FakeLocal::new(Decision::Allow, order.clone(), Duration::ZERO);
    let distributed = FakeDistributed::new(
        DistributedCheckOutcome::BackendError {
            retry_after: Duration::from_millis(50),
        },
        Duration::ZERO,
        order,
    );
    let sink = RecordingEventSink::default();

    let hybrid = build_hybrid(
        local,
        distributed,
        sink,
        InstrumentationLevel::Off,
        DistributedFailurePolicy::FailClosed,
    );

    assert_eq!(
        Decision::Deny {
            retry_after: Duration::from_millis(50)
        },
        hybrid.check(&"key".to_string()).await
    );
}

#[tokio::test]
async fn instrumentation_emits_operation_completed() {
    let order = Arc::new(Mutex::new(Vec::new()));
    let local = FakeLocal::new(Decision::Allow, order.clone(), Duration::ZERO);
    let distributed = FakeDistributed::new(DistributedCheckOutcome::Allow, Duration::ZERO, order);
    let sink = RecordingEventSink::default();

    let hybrid = build_hybrid(
        local,
        distributed,
        sink.clone(),
        InstrumentationLevel::Basic,
        DistributedFailurePolicy::FailOpen,
    );

    assert_eq!(Decision::Allow, hybrid.check(&"key".to_string()).await);
    let events = sink.events();
    assert_eq!(1, events.len());
    assert!(matches!(events[0], Event::OperationCompleted { .. }));
}

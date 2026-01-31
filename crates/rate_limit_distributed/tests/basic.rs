use std::{sync::Arc, time::Duration};

use rate_limit::{
    event_sink::NoopEventSink,
    models::{Decision, InstrumentationLevel},
};
use rate_limit_distributed::DistributedKeyedLimiter;
use state_backend::{BackendDecision, LimitSpec};

mod support {
    pub mod fake_backend;
    pub mod recorded_events;
    pub mod recording_event_sink;
}
use support::fake_backend::FakeBackend;
use support::recorded_events::RecordedEvents;
use support::recording_event_sink::RecordingEventSink;

#[tokio::test]
async fn allows_when_backend_allows() {
    let backend = FakeBackend {
        decision: Ok(BackendDecision::Allow),
    };

    let limiter = DistributedKeyedLimiter::<String, _, _>::new(
        Arc::new(backend),
        "test",
        LimitSpec {
            window: Duration::from_secs(1),
            max: 1,
        },
        NoopEventSink,
        InstrumentationLevel::Off,
    );

    let decision = limiter.check(&"user1".to_string()).await;
    assert_eq!(decision, Decision::Allow);
}

#[tokio::test]
async fn denies_when_backend_denies() {
    let backend = FakeBackend {
        decision: Ok(BackendDecision::Deny {
            retry_after: Duration::from_millis(123),
        }),
    };

    let limiter = DistributedKeyedLimiter::<String, _, _>::new(
        Arc::new(backend),
        "test",
        LimitSpec {
            window: Duration::from_secs(1),
            max: 1,
        },
        NoopEventSink,
        InstrumentationLevel::Off,
    );

    let decision = limiter.check(&"user1".to_string()).await;
    assert_eq!(
        decision,
        Decision::Deny {
            retry_after: Duration::from_millis(123)
        }
    );
}

#[tokio::test]
async fn denies_with_backend_error_retry_after() {
    let backend = FakeBackend { decision: Err(()) };

    let limiter = DistributedKeyedLimiter::<String, _, _>::new(
        Arc::new(backend),
        "test",
        LimitSpec {
            window: Duration::from_secs(1),
            max: 1,
        },
        NoopEventSink,
        InstrumentationLevel::Off,
    );

    let decision = limiter.check(&"user1".to_string()).await;
    assert_eq!(
        decision,
        Decision::Deny {
            retry_after: Duration::from_millis(50)
        }
    );
}

#[tokio::test]
async fn instrumentation_emits_completed_event() {
    let backend = FakeBackend {
        decision: Ok(BackendDecision::Allow),
    };

    let sink = RecordingEventSink::default();
    let limiter = DistributedKeyedLimiter::<String, _, _>::new(
        Arc::new(backend),
        "test",
        LimitSpec {
            window: Duration::from_secs(1),
            max: 1,
        },
        sink.clone(),
        InstrumentationLevel::Basic,
    );

    let decision = limiter.check(&"user1".to_string()).await;
    assert_eq!(decision, Decision::Allow);

    let events = sink.events();
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0],
        rate_limit::models::Event::OperationCompleted {
            operation: rate_limit::models::Operation::RateLimitCheck,
            decision: Decision::Allow,
        }
    );
}

use std::num::NonZeroU32;
use std::time::Duration;

use governor::Quota;

use rate_limit::direct_limiter::DirectLimiter;
use rate_limit::models::{Decision, Event, InstrumentationLevel, Operation};

mod support {
    pub mod recorded_events;
    pub mod recording_event_sink;
}

use support::recorded_events::RecordedEvents;
use support::recording_event_sink::RecordingEventSink;

#[test]
fn records_allow_events() {
    let quota = Quota::per_second(NonZeroU32::new(2).unwrap());
    let sink = RecordingEventSink::default();
    let limiter = DirectLimiter::new(quota, sink.clone(), InstrumentationLevel::Full);

    assert_eq!(limiter.check(), Decision::Allow);
    assert_eq!(limiter.check(), Decision::Allow);

    let events = sink.events();
    assert_eq!(
        events,
        vec![
            Event::OperationStarted {
                operation: Operation::RateLimitCheck
            },
            Event::OperationCompleted {
                operation: Operation::RateLimitCheck,
                decision: Decision::Allow
            },
            Event::OperationStarted {
                operation: Operation::RateLimitCheck
            },
            Event::OperationCompleted {
                operation: Operation::RateLimitCheck,
                decision: Decision::Allow
            }
        ]
    );
}

#[test]
fn records_deny_event_with_retry_after() {
    let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
    let sink = RecordingEventSink::default();
    let limiter = DirectLimiter::new(quota, sink.clone(), InstrumentationLevel::Full);

    assert_eq!(limiter.check(), Decision::Allow);

    match limiter.check() {
        Decision::Deny { retry_after } => {
            assert!(retry_after > Duration::ZERO);
        }
        Decision::Allow => panic!("expected denial"),
    }

    let events = sink.events();
    assert_eq!(events.len(), 4);

    match &events[3] {
        Event::OperationCompleted {
            operation: Operation::RateLimitCheck,
            decision: Decision::Deny { retry_after },
        } => {
            assert!(*retry_after > Duration::ZERO);
        }
        other => panic!("unexpected event: {:?}", other),
    }
}

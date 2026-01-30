use std::num::NonZeroU32;

use governor::Quota;

use rate_limit::direct_limiter::DirectLimiter;
use rate_limit::models::{Event, InstrumentationLevel};

mod support {
    pub mod recorded_events;
    pub mod recording_event_sink;
}

use support::recorded_events::RecordedEvents;
use support::recording_event_sink::RecordingEventSink;

#[test]
fn instrumentation_off_emits_no_events() {
    let sink = RecordingEventSink::default();
    let limiter = DirectLimiter::new(
        Quota::per_second(NonZeroU32::new(1).unwrap()),
        sink.clone(),
        InstrumentationLevel::Off,
    );

    limiter.check();
    assert!(sink.events().is_empty());
}

#[test]
fn instrumentation_basic_emits_completed_only() {
    let sink = RecordingEventSink::default();
    let limiter = DirectLimiter::new(
        Quota::per_second(NonZeroU32::new(1).unwrap()),
        sink.clone(),
        InstrumentationLevel::Basic,
    );

    limiter.check();

    let events = sink.events();
    assert_eq!(events.len(), 1);
    matches!(events[0], Event::OperationCompleted { .. });
}

#[test]
fn instrumentation_full_emits_start_and_complete() {
    let sink = RecordingEventSink::default();

    let limiter = DirectLimiter::new(
        Quota::per_second(NonZeroU32::new(1).unwrap()),
        sink.clone(),
        InstrumentationLevel::Full,
    );

    limiter.check();

    let events = sink.events();
    assert_eq!(events.len(), 2);
    matches!(events[0], Event::OperationStarted { .. });
    matches!(events[1], Event::OperationCompleted { .. });
}

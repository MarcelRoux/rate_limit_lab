use crate::support::recording_event_sink::RecordingEventSink;

use rate_limit::models::Event;

/// Trait to access recorded events in tests only when needed.
pub trait RecordedEvents {
    fn events(&self) -> Vec<Event>;
}

impl RecordedEvents for RecordingEventSink {
    fn events(&self) -> Vec<Event> {
        self.events.lock().unwrap().clone()
    }
}

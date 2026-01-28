use std::sync::{Arc, Mutex};

use rate_limit::event_sink::EventSink;
use rate_limit::models::Event;

/// Thread-safe event recorder for tests.
#[cfg(test)]
#[derive(Clone, Default)]
pub struct RecordingEventSink {
    events: Arc<Mutex<Vec<Event>>>,
}

#[cfg(test)]
impl RecordingEventSink {
    pub fn events(&self) -> Vec<Event> {
        self.events.lock().unwrap().clone()
    }
}

#[cfg(test)]
impl EventSink for RecordingEventSink {
    fn emit(&self, event: Event) {
        self.events.lock().unwrap().push(event);
    }
}

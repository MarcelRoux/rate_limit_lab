use std::sync::{Arc, Mutex};

use rate_limit::event_sink::EventSink;
use rate_limit::models::Event;

/// Thread-safe event recorder for tests.
#[derive(Clone, Default)]
pub struct RecordingEventSink {
    pub(crate) events: Arc<Mutex<Vec<Event>>>,
}

impl EventSink for RecordingEventSink {
    fn emit(&self, event: Event) {
        self.events.lock().unwrap().push(event);
    }
}

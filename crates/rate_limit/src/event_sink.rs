use std::sync::Arc;

use crate::models::Event;

/// Core event sink abstraction.
/// Instrumentation will attach here in M1.3.
pub trait EventSink: Send + Sync {
    fn emit(&self, event: Event);
}

/// Implement EventSink for Arc-wrapped sinks.
impl<T: EventSink> EventSink for Arc<T> {
    fn emit(&self, event: Event) {
        (**self).emit(event)
    }
}

/// Default no-op sink (zero overhead).
#[derive(Clone)]
pub struct NoopEventSink;

impl EventSink for NoopEventSink {
    #[inline]
    fn emit(&self, _event: Event) {}
}

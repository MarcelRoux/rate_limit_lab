use crate::models::Event;

/// Core event sink abstraction.
/// Instrumentation will attach here in M1.3.
pub trait EventSink: Send + Sync {
    fn emit(&self, event: Event);
}

/// Default no-op sink (zero overhead).
pub struct NoopEventSink;

impl EventSink for NoopEventSink {
    #[inline]
    fn emit(&self, _event: Event) {}
}

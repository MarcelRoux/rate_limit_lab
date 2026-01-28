use std::hash::Hash;

use governor::Quota;

use crate::direct_limiter::DirectLimiter;
use crate::event_sink::EventSink;
use crate::hierarchical_limiter::HierarchicalLimiter;
use crate::keyed_limiter::KeyedLimiter;
use crate::models::InstrumentationLevel;

pub fn direct_limiter<S>(
    quota: Quota,
    sink: S,
    instrumentation: InstrumentationLevel,
) -> DirectLimiter<S>
where
    S: EventSink,
{
    DirectLimiter::new(quota, sink, instrumentation)
}

pub fn keyed_limiter<K, S>(
    quota: Quota,
    sink: S,
    instrumentation: InstrumentationLevel,
) -> KeyedLimiter<K, S>
where
    K: Eq + Hash + Clone,
    S: EventSink,
{
    KeyedLimiter::new(quota, sink, instrumentation)
}

pub fn hierarchical_limiter<K, S>(
    global_quota: Quota,
    key_quota: Quota,
    sink: S,
    instrumentation: InstrumentationLevel,
) -> HierarchicalLimiter<K, S>
where
    K: Eq + Hash + Clone,
    S: EventSink,
{
    HierarchicalLimiter::new(global_quota, key_quota, sink, instrumentation)
}

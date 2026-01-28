use std::sync::Arc;

use tower::Layer;

use rate_limit::{
    event_sink::EventSink, hierarchical_limiter::HierarchicalLimiter, models::RateLimitKey,
};

use crate::middleware::RateLimitMiddleware;

#[derive(Clone)]
pub struct RateLimitLayer<S>
where
    S: EventSink,
{
    limiter: Arc<HierarchicalLimiter<RateLimitKey, S>>,
}

impl<S> RateLimitLayer<S>
where
    S: EventSink,
{
    pub fn new(limiter: HierarchicalLimiter<RateLimitKey, S>) -> Self {
        Self {
            limiter: Arc::new(limiter),
        }
    }
}

impl<Svc, S> Layer<Svc> for RateLimitLayer<S>
where
    Svc: Clone,
    S: EventSink,
{
    type Service = RateLimitMiddleware<Svc, S>;

    fn layer(&self, inner: Svc) -> Self::Service {
        RateLimitMiddleware {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

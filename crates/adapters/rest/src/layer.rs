use std::sync::Arc;

use tower::Layer;

use crate::middleware::{RateLimitMiddleware, RateLimitPolicy};

// compile_error!("layer.rs is being compiled");

pub struct RateLimitLayer<L> {
    limiter: Arc<L>,
}

// Manual Clone impl with **no bounds** on L.
impl<L> Clone for RateLimitLayer<L> {
    fn clone(&self) -> Self {
        Self {
            limiter: self.limiter.clone(),
        }
    }
}

impl<L> RateLimitLayer<L> {
    /// Take an `Arc<L>` so the caller controls sharing explicitly.
    pub fn new(limiter: Arc<L>) -> Self {
        Self { limiter }
    }
}

impl<Svc, L> Layer<Svc> for RateLimitLayer<L>
where
    Svc: Clone,
    L: RateLimitPolicy + Send + Sync + 'static,
{
    type Service = RateLimitMiddleware<Svc, L>;

    fn layer(&self, inner: Svc) -> Self::Service {
        RateLimitMiddleware {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

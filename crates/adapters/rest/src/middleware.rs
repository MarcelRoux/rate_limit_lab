use std::{
    future::{Future, Ready, ready},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use axum::{
    http::{HeaderValue, Request, StatusCode},
    response::Response,
};
use tower::Service;

use crate::extractor::extract_key;
use rate_limit::event_sink::EventSink;
use rate_limit::hierarchical_limiter::HierarchicalLimiter;
use rate_limit::models::{Decision, RateLimitKey};
use rate_limit_distributed::DistributedKeyedLimiter;
use state_backend::StateBackend;

// compile_error!("middleware.rs is being compiled");

/// Protocol adapter abstraction: any limiter that can produce a `Decision` for a string key.
///
/// NOTE: This uses a GAT so implementations can return a concrete future type.
/// - In-memory limiters can return `Ready<Decision>` (no alloc).
/// - Distributed limiters can return `Pin<Box<dyn Future<...>>>` if needed.
pub trait RateLimitPolicy: Send + Sync + 'static {
    type Fut<'a>: Future<Output = Decision> + Send + 'a
    where
        Self: 'a;

    fn check<'a>(&'a self, key: &'a str) -> Self::Fut<'a>;
}

// -----------------------------
// In-memory hierarchical limiter
// -----------------------------
impl<S> RateLimitPolicy for HierarchicalLimiter<RateLimitKey, S>
where
    S: EventSink + Send + Sync + 'static,
{
    type Fut<'a>
        = Ready<Decision>
    where
        Self: 'a;

    fn check<'a>(&'a self, key: &'a str) -> Self::Fut<'a> {
        // NOTE: still allocates a String for the key.
        // Future improvement: avoid allocation by changing extractor / core key type.
        let decision = self.check(&RateLimitKey(key.to_string()));
        ready(decision)
    }
}

// -----------------------------
// Distributed keyed limiter
// -----------------------------
impl<K, B, S> RateLimitPolicy for DistributedKeyedLimiter<K, B, S>
where
    K: AsRef<str> + Send + Sync + 'static,
    B: StateBackend + Send + Sync + 'static,
    S: EventSink + Send + Sync + 'static,
{
    // We can't name the concrete future type easily without `impl Trait` in associated types,
    // so box the async future.
    type Fut<'a>
        = Pin<Box<dyn Future<Output = Decision> + Send + 'a>>
    where
        Self: 'a;

    fn check<'a>(&'a self, key: &'a str) -> Self::Fut<'a> {
        // K is generic, but the distributed limiter API is `check(&K)`.
        // With K = String (your server uses that), we must allocate.
        //
        // Future improvement: add `check_str(&self, key: &str)` to DistributedKeyedLimiter
        // to internalize this allocation and/or allow K = Cow<'a, str>.
        let k = key.to_owned();
        Box::pin(async move { self.check_str(&k).await })
    }
}

// ---------------------------------
// Middleware (generic over limiter L)
// ---------------------------------
pub struct RateLimitMiddleware<Svc, L> {
    pub(crate) inner: Svc,
    pub(crate) limiter: Arc<L>,
}

// NOTE: Intentionally implement Clone manually instead of `#[derive(Clone)]`.
//
// In this middleware, `limiter` is stored behind `Arc<L>`. Cloning an `Arc` does NOT
// require `L: Clone` (it only increments the refcount).
//
// `#[derive(Clone)]` can introduce stricter inferred bounds in generic contexts,
// and Axum requires the produced service (the middleware) to be `Clone`.
// A manual impl allows expressing the minimal correct bound: only `Svc: Clone`.
impl<Svc: Clone, L> Clone for RateLimitMiddleware<Svc, L> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            limiter: self.limiter.clone(),
        }
    }
}

impl<Svc, L, B> Service<Request<B>> for RateLimitMiddleware<Svc, L>
where
    Svc: Service<Request<B>, Response = Response> + Clone + Send + 'static,
    Svc::Future: Send + 'static,
    L: RateLimitPolicy,
    B: Send + 'static,
{
    type Response = Response;
    type Error = Svc::Error;

    // This is still boxed (heap alloc) per request.
    // If you want to remove this too, you need a named future type instead of Box<dyn Future>.
    type Future = Pin<Box<dyn Future<Output = Result<Response, Svc::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let key = extract_key(&req);
        let key_string = key.0;

        let limiter = self.limiter.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let decision = limiter.check(&key_string).await;

            match decision {
                Decision::Allow => inner.call(req).await,
                Decision::Deny { retry_after } => {
                    let mut response = Response::new("rate limit exceeded".into());
                    *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;

                    let secs = retry_after.as_secs().max(1);
                    let hv = HeaderValue::from_str(&secs.to_string()).unwrap();

                    response.headers_mut().insert("retry-after", hv);

                    Ok(response)
                }
            }
        })
    }
}

use axum::{
    http::{Request, StatusCode},
    response::Response,
};
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tower::Service;

use crate::extractor::extract_key;
use rate_limit::{
    event_sink::EventSink,
    hierarchical_limiter::HierarchicalLimiter,
    models::{Decision, RateLimitKey},
};

#[derive(Clone)]
pub struct RateLimitMiddleware<Svc, S: EventSink> {
    pub(crate) inner: Svc,
    pub(crate) limiter: Arc<HierarchicalLimiter<RateLimitKey, S>>,
}

impl<Svc, S, B> Service<Request<B>> for RateLimitMiddleware<Svc, S>
where
    Svc: Service<Request<B>, Response = Response> + Clone + Send + 'static,
    Svc::Future: Send + 'static,
    S: EventSink + 'static,
    B: std::marker::Send + 'static,
{
    type Response = Response;
    type Error = Svc::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let key = extract_key(&req);
        let decision = self.limiter.check(&key);

        match decision {
            Decision::Allow => {
                let mut inner = self.inner.clone();
                Box::pin(async move { inner.call(req).await })
            }

            Decision::Deny { retry_after } => Box::pin(async move {
                let mut response = Response::new("rate limit exceeded".into());
                *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;

                response.headers_mut().insert(
                    "retry-after",
                    retry_after.as_secs().to_string().parse().unwrap(),
                );

                Ok(response)
            }),
        }
    }
}

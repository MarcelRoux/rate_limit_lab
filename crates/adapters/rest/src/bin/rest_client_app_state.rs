use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use tokio::signal;

use rate_limit::{
    event_sink::NoopEventSink,
    factory::hierarchical_limiter,
    hierarchical_limiter::HierarchicalLimiter,
    models::{Decision, InstrumentationLevel, RateLimit},
};

type Limiter = HierarchicalLimiter<String, NoopEventSink>;

#[derive(Clone)]
struct AppState {
    limiter: Arc<Limiter>,
}

fn extract_key(headers: &HeaderMap) -> String {
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous")
        .to_string()
}

async fn root(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let key = extract_key(&headers);

    match state.limiter.check(&key) {
        Decision::Allow => (StatusCode::OK, "ok").into_response(),
        Decision::Deny { retry_after } => {
            // Optional but nice: Retry-After header in seconds (rounded up, min 1)
            let secs = retry_after.as_secs_f64().ceil().max(1.0) as u64;
            let mut resp = (StatusCode::TOO_MANY_REQUESTS, "rate limited").into_response();
            resp.headers_mut()
                .insert("retry-after", secs.to_string().parse().unwrap());
            resp
        }
    }
}

#[tokio::main]
async fn main() {
    let sink = NoopEventSink;
    let global_limit = RateLimit::per_second(1_000).expect("non-zero");
    let per_key_limit = RateLimit::per_second(1_000).expect("non-zero");

    let limiter = hierarchical_limiter::<String, _>(
        global_limit.to_quota(),
        per_key_limit.to_quota(),
        sink,
        InstrumentationLevel::Basic,
    );

    // ---- Axum app ----
    let state = AppState {
        limiter: Arc::new(limiter),
    };

    let app: Router = Router::new().route("/", get(root)).with_state(state);

    // ---- Server ----
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("REST rate-limited server listening on {}", addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    println!("shutdown signal received");
}

use axum::http::Request;
use rate_limit::models::RateLimitKey;

/// Extracts a rate-limit key from an HTTP request.
///
/// M2.3 policy:
/// - Prefer `x-api-key`
/// - Fall back to `"anonymous"`
pub fn extract_key<B>(req: &Request<B>) -> RateLimitKey {
    let key = req
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anonymous");

    RateLimitKey(key.to_string())
}

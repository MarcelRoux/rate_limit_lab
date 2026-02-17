# Milestone M2 — Multi-Level Limiting (REST)

## Goal

Stream REST requests through the rate-limiting core and drive experiments via a REST-focused traffic generator. (crates/adapters/rest/src/middleware.rs: RateLimitMiddleware; crates/traffic_rest/src/runner.rs: run_profile)

## Implemented Components

- `crate: rest`
  - `RateLimitPolicy` abstraction that allows the middleware to treat different limiter implementations uniformly. (crates/adapters/rest/src/middleware.rs: RateLimitPolicy)
  - `RateLimitMiddleware` and `RateLimitLayer` for Axum/tower integration that emit HTTP 429 and `retry-after` headers on deny decisions. (crates/adapters/rest/src/middleware.rs: RateLimitMiddleware; crates/adapters/rest/src/layer.rs: RateLimitLayer)
  - `extract_key` helper that prefers `x-api-key` and falls back to `"anonymous"`. (crates/adapters/rest/src/extractor.rs: extract_key)
- `crate: traffic_rest`
  - Traffic configuration types `TrafficProfile` and `KeyMode`. (crates/traffic_rest/src/model.rs: TrafficProfile, KeyMode)
  - `run_profile` that paces requests, enforces concurrency, and selects keys per configuration. (crates/traffic_rest/src/runner.rs: run_profile)
  - `summarize` helper to derive latency percentiles and status counts. (crates/traffic_rest/src/metrics.rs: summarize)

## Decision Semantics

- Middleware extracts a key and awaits the configured limiter decision for that key. (crates/adapters/rest/src/extractor.rs: extract_key; crates/adapters/rest/src/middleware.rs: RateLimitMiddleware::call)
- `Decision::Allow` proxies the request to the inner service; `Decision::Deny` responds with HTTP 429 plus a seconds-based `retry-after` header (minimum 1). (crates/adapters/rest/src/middleware.rs: RateLimitMiddleware::call)
- In-memory hierarchical limiters return a ready future while distributed limiters box their async future, but both satisfy `RateLimitPolicy`. (crates/adapters/rest/src/middleware.rs: impl RateLimitPolicy)

## Failure Behavior

- Limiters only expose `Decision`; the middleware does not distinguish errors beyond the deny flow. (crates/adapters/rest/src/middleware.rs: RateLimitPolicy)
- `retry-after` header creation assumes valid seconds strings and panics if header serialization fails. (crates/adapters/rest/src/middleware.rs: RateLimitMiddleware::call)

## Out of Scope

- gRPC adapters remain future work. (Cargo.toml: workspace members)
- Any distributed failure-policy experimentation beyond invoking `RateLimitPolicy::check`. (crates/adapters/rest/src/middleware.rs: impl RateLimitPolicy for DistributedKeyedLimiter; docs/adr/ADR-0004-distributed-failure-policy.md)

## Acceptance Checklist

- [ ] Axum middleware translates limiter denies into 429 responses with `retry-after`. (crates/adapters/rest/tests/middleware.rs: rest_middleware_denies_request)
- [ ] Axum middleware allows requests when the limiter decision is `Allow`. (crates/adapters/rest/tests/middleware.rs: rest_middleware_allows_request)
- [ ] Traffic generator supports keyless and keyed modes with paced concurrency. (crates/traffic_rest/src/model.rs: KeyMode; crates/traffic_rest/src/runner.rs: run_profile)

## Related ADRs

- ADR-0004 — Distributed backend failure policy toggle anchors later failure experiments. (docs/adr/ADR-0004-distributed-failure-policy.md)

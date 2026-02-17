# ADR-0002: Distributed Backend Abstraction and Current Failure Mapping

## Status

Accepted (M3.1 distributed keyed limiter)

## Context

Distributed per-key limits require backend state (initially Redis) but the limiter should stay backend-agnostic for benchmarking and testing. (crates/rate_limit_distributed/src/distributed_keyed_limiter.rs: DistributedKeyedLimiter; crates/state_backend/src/backend.rs: StateBackend)

## Decision

Use a `B: StateBackend` generic parameter stored behind an `Arc` for static dispatch, and translate backend errors into `Decision::Deny { retry_after: 50 ms }` until the failure policy evolves. (crates/state_backend/src/backend.rs: StateBackend; crates/rate_limit_distributed/src/distributed_keyed_limiter.rs: DistributedKeyedLimiter::check)

## Rationale

- Generic backends avoid trait-object overhead and make it easy to swap Redis for fakes or other stores. (crates/rate_limit_distributed/src/distributed_keyed_limiter.rs: DistributedKeyedLimiter; crates/rate_limit_distributed/tests/basic.rs: FakeBackend)
- A fixed retry-after on errors keeps the behavior deterministic while the failure policy is still a placeholder. (crates/rate_limit_distributed/src/distributed_keyed_limiter.rs: BACKEND_ERROR_RETRY_AFTER; DistributedKeyedLimiter::check)
- Tradeoff: static dispatch duplicates code per backend but keeps latency low on the hot path. (crates/rate_limit_distributed/src/distributed_keyed_limiter.rs: DistributedKeyedLimiter)

## Consequences

- Positive: Tests validate allow/deny mapping and the placeholder 50 ms retry for backend errors. (crates/rate_limit_distributed/tests/basic.rs: allows_when_backend_allows; denies_when_backend_denies; denies_with_backend_error_retry_after)
- Negative: Mapping all backend failures to denies may degrade availability during outages until the failure policy is expanded. (crates/rate_limit_distributed/src/distributed_keyed_limiter.rs: BACKEND_ERROR_RETRY_AFTER; DistributedKeyedLimiter::check)

## Related

- Milestone M3.1 — Distributed keyed limiting rests on this abstraction. (docs/milestones/M3.1-distributed-keyed.md)

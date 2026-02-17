# ADR-0001: Hierarchical AND Semantics and Retry-After Aggregation

## Status

Accepted (M1 local hierarchical limiter)

## Context

The hierarchical limiter composes a global and a per-key governor-backed limiter but must expose a single protocol-agnostic `Decision`. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter)

## Decision

Require both tiers to allow; any deny causes an overall deny, and the resulting `retry_after` equals the maximum duration reported by the denying components. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter::check)

## Rationale

- AND semantics keep enforcement deterministic across both tiers, matching the design of the core limiters. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter::check)
- Maximum `retry_after` ensures callers wait for the slowest-recovering constraint rather than retrying prematurely. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter::check)
- Tradeoff: strict AND increases denial surface (a single-tier failure blocks the request) but maintains a clear, observable decision model for adapters. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter::check)

## Consequences

- Positive: Tests confirm the limiter allows only when both quotas permit and denies deterministically otherwise. (crates/rate_limit/tests/hierarchical_limiter.rs: hierarchical_allows_if_both_pass; hierarchical_denies_if_global_exceeded; hierarchical_denies_if_key_exceeded)
- Negative: Clients may experience longer retries because `retry_after` reflects the worst tier, and higher-level services cannot distinguish which tier caused the deny. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter::check)

## Related

- Milestone M1 — Local hierarchical limiting composes these tiers. (docs/milestones/M1-local-hierarchical.md)

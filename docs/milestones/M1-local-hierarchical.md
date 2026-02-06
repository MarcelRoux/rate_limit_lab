# Milestone M1 — Local Hierarchical Limiting

## Goal
Compose the global and per-key in-memory limiters so that each request is evaluated deterministically across both tiers. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter)

## Implemented Components
- `crate: rate_limit`
  - `HierarchicalLimiter` instantiates `DirectLimiter` (global) and `KeyedLimiter` (per-key) with shared instrumentation and event sinks. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter, HierarchicalLimiter::new)
  - Emits the core `Decision` and `Event` outcomes for downstream observers. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter::check; crates/rate_limit/src/models.rs: Decision, Event)

## Decision Semantics
- Both tiers are checked for every operation; only a dual-allow results in `Decision::Allow`. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter::check)
- A deny adopts the maximum `retry_after` from any denying component, ensuring callers wait for the most congested constraint. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter::check)
- When not `InstrumentationLevel::Off`, a single `OperationCompleted` event is emitted; hierarchical checks do not emit `OperationStarted`. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter::check; crates/rate_limit/src/models.rs: InstrumentationLevel, Event)

## Failure Behavior
- The public API returns only `Decision`; no error variants are surfaced. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter::check)
- Internal failure handling is not documented. Unknown — to verify in `crates/rate_limit/src/hierarchical_limiter.rs`.

## Out of Scope
- Distributed/backed limiters remain in later milestones. (crates/rate_limit_distributed/src/lib.rs; crates/state_backend/src/lib.rs)
- Protocol adapters and traffic generation are handled separately. (crates/adapters/rest/src/lib.rs; crates/traffic_rest/src/lib.rs)

## Acceptance Checklist
- [ ] Requests succeed only when both global and per-key quanta are available. (crates/rate_limit/tests/hierarchical_limiter.rs: hierarchical_allows_if_both_pass)
- [ ] Requests are denied when the global limit is already exhausted. (crates/rate_limit/tests/hierarchical_limiter.rs: hierarchical_denies_if_global_exceeded)
- [ ] Requests are denied when the per-key limit is already exhausted. (crates/rate_limit/tests/hierarchical_limiter.rs: hierarchical_denies_if_key_exceeded)

## Related ADRs
- ADR-0001 — Hierarchical AND semantics and retry-after aggregation. (docs/adr/ADR-0001-hierarchical-and-retry-after.md)

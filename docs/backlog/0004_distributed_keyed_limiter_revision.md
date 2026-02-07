# Backlog Item: Distributed keyed limiter revision

Status: Proposed
Priority: P1
Milestone: M3.2
Owner: TBD
Created: 2026-02-07
Links:
- ADR: ADR-0004-distributed-failure-policy.md

---

## Summary
As the hybrid limiter experiments with fail-open/fail-closed policies, the distributed keyed limiter should naturally evolve to surface the backend error (vs deny) metadata that those policies consume. This entry describes the refinement for `DistributedKeyedLimiter` so hybrid can reuse its logic while still reasoning about backend failures explicitly.

## Motivation
- The hybrid limiter should treat distributed backend failures according to a configurable policy while reusing the existing `DistributedKeyedLimiter` implementation.
- Evolving the distributed crate to explicitly expose when a backend actually errored (vs simply denying) lets the hybrid layer make deterministic fail-open/fail-closed choices without duplicated plumbing.
- Success is a distributed limiter API that provides the richer data hybrid needs while keeping the current logic intact.

## Proposed approach
- Audit `DistributedKeyedLimiter::check`/`check_str` and determine which pieces of metadata are currently lost once the decision is created.
- Introduce an explicit error/decision placeholder (or translation adapter) that lets callers know whether the backend actually errored vs returned deny.
- Document the translation path for hybrid so the new interface is discoverable (this backlog entry plus ADR-0004 should cross-reference).

## Scope
In-scope:
- Changes inside `crates/rate_limit_distributed/src/distributed_keyed_limiter.rs`.
- Any supporting abstractions required to signal fail-open vs fail-closed policies.

Out-of-scope:
- Rewriting the storage backend itself - TBC based on effort level.
- Shifting the failure policy responsibilities out of M3.2.

## Acceptance criteria
- [ ] Distributed limiter exposes enough information for hybrid to implement both fail-open and fail-closed behaviors deterministically.
- [ ] Hybrid limiter no longer needs to duplicate backend error handling (it can reuse the new translation adapter).
- [ ] The fallback instrumentation comment in `HybridLimiter::emit_operation_completed` can point to this file without being “todo leftover”.

## Risks / tradeoffs
- Risk: Changing the distributed API now could break downstream consumers that already rely on the simple `Decision`. Mitigate: keep the existing API and add a new helper/adapter rather than breaking changes.
- Risk: Delay in shipping hybrid failure policy while refactor completes. Mitigate: keep the existing fail-open placeholder in hybrid until the distributed revision lands.

## Dependencies
- Coordinating with the rate limit model evolution (docs/backlog/0003-rate-limit-model-reversion.md) so instrumentation metadata stays aligned.

## Validation plan
- `cargo test -p rate_limit_distributed`.
- Add regression test verifying hybrid observes both policies once the new adapter is wired.

## Outcome
(leave blank until closed)

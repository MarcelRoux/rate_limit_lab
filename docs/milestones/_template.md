# Milestone Mx — <Short descriptive title>

## Goal
One paragraph stating:
- what capability this milestone introduces
- what problem it solves
- why it exists in the system

## Implemented Components
List concrete implementation units.

Example:

- `crate: rate_limit`
  - `DirectLimiter` — global in-memory limiter
  - `KeyedLimiter` — per-key in-memory limiter
- `crate: rate_limit_distributed`
  - `DistributedKeyedLimiter` — backend-backed keyed limiter

(Include file paths where useful.)

## Decision Semantics
Describe the behavior precisely.

Example:
- Decisions are combined using AND semantics.
- A deny occurs if any component denies.
- `retry_after` is the maximum observed among denying components.

## Failure Behavior
Explicitly document:

- what happens on internal errors
- what happens on backend failures
- default retry behavior

## Out of Scope
List what this milestone intentionally does NOT handle.

Example:
- failure policy experimentation
- distributed budget caching
- adaptive backoff

## Acceptance Checklist
Concrete criteria:

- [ ] Feature works as described
- [ ] Tests cover main semantics and failure cases
- [ ] Observability events emitted
- [ ] Documentation updated

## Related ADRs
Link decisions:

- ADR-0003 — Hybrid limiter semantics
- ADR-0004 — Distributed backend failure policy
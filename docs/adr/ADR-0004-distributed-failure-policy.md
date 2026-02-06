# ADR-0004: Distributed Backend Failure Policy Toggle

## Status
Accepted (M3.2)

## Context
Current distributed limiting maps backend errors to denies, but failure injection experiments need a switchable policy between availability-first and correctness-first behaviors. (crates/rate_limit_distributed/src/distributed_keyed_limiter.rs: DistributedKeyedLimiter)

## Decision
Introduce `DistributedFailurePolicy` with `FailOpen` (allow on errors) and `FailClosed` (deny with `retry_after = BACKEND_ERROR_RETRY_AFTER`), defaulting to `FailOpen`, and emit structured events for backend errors so observers can understand what policy was applied. (docs/adr/ADR-0002-distributed-backend-abstraction.md; docs/milestones/M3.2-hybrid-limiting.md)

## Rationale
- FailOpen keeps the service available during injected backend failures while still surfacing the error decisions through events/counters. (docs/adr/ADR-0002-distributed-backend-abstraction.md)
- FailClosed retains correctness guarantees when desired, giving operators an explicit knob. (docs/adr/ADR-0002-distributed-backend-abstraction.md)
- Tradeoff: making the policy configurable adds complexity to the limiter API and observability surface but prevents silent behavior changes. (crates/rate_limit_distributed/src/distributed_keyed_limiter.rs: DistributedKeyedLimiter)

## Consequences
- Positive: Failure-injection experiments can run without taking the service offline, and operators can observe which policy handled backend errors. (docs/adr/ADR-0002-distributed-backend-abstraction.md)
- Negative: The added policy state increases the configuration surface and requires instrumentation changes that must stay synchronized with the limiter. (docs/adr/ADR-0002-distributed-backend-abstraction.md)

## Related
- Milestone M3.2 — Planning hybrid/local+distributed failure injection. (docs/milestones/M3.2-hybrid-limiting.md)

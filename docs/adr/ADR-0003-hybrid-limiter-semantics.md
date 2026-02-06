# ADR-0003: Hybrid Limiter Semantics and Ordering

## Status
Accepted (M3.2)

## Context
Hybrid limiting must coordinate the synchronous local hierarchy with the asynchronous distributed check while minimizing latency. (crates/rate_limit/src/hierarchical_limiter.rs: HierarchicalLimiter; crates/rate_limit_distributed/src/distributed_keyed_limiter.rs: DistributedKeyedLimiter)

## Decision
Start the distributed future, run the local check immediately, short-circuit on local denies, and otherwise await the distributed result; aggregate with strict AND semantics and `retry_after = max`. (docs/adr/ADR-0001-hierarchical-and-retry-after.md; docs/adr/ADR-0002-distributed-backend-abstraction.md)

## Rationale
- Preserves the same decision semantics as the existing hierarchical limiter while extending them to include distributed state. (docs/adr/ADR-0001-hierarchical-and-retry-after.md)
- Parallel ordering reduces tail latency when distributed checks are slow but all local quotas allow. (docs/adr/ADR-0001-hierarchical-and-retry-after.md; docs/adr/ADR-0002-distributed-backend-abstraction.md)
- Tradeoff: starting the distributed work eagerly may waste backend resources when the local tier already denies, but keeps observed latency low for allowed requests. (crates/rate_limit_distributed/src/distributed_keyed_limiter.rs: DistributedKeyedLimiter)

## Consequences
- Positive: Local denies return immediately, while allowed cases await both checks for completeness. (docs/adr/ADR-0001-hierarchical-and-retry-after.md; docs/adr/ADR-0002-distributed-backend-abstraction.md)
- Negative: Hybrid API becomes async and backend calls may still be initiated even when the local limiter short-circuits. (docs/adr/ADR-0002-distributed-backend-abstraction.md)

## Related
- Milestone M3.2 — Hybrid limiter orchestration. (docs/milestones/M3.2-hybrid-limiting.md)

# ADR-0005: Single Configurable REST Server Binary

## Status
Superseded by ADR-0006

## Context

Early development used multiple Axum server binaries to test different rate-limiter implementations. As the system evolved, this approach caused:

- configuration drift
- duplicated wiring
- difficulty running comparative benchmarks
- friction when adding new limiter variants

A scalable solution was required.

---

## Decision

Adopt a **single REST server binary** whose behavior is fully selected at runtime via configuration.

Limiter implementations (in-memory, distributed, hybrid) are selected using a configuration-driven factory and exposed to the adapter via a common `RateLimitPolicy` interface.

---

## Consequences

### Positive
- No recompilation when adding limiter variants
- Cleaner experimentation workflow
- Adapter boundary clearly defined
- Strong architectural signal for portfolio review
- Extensible to future backends and protocols

### Negative
- Requires dynamic dispatch at the adapter boundary
- Slight complexity increase in config parsing
- Must document configuration carefully

These tradeoffs are acceptable given the scope and performance characteristics of REST adapters.

---

## Alternatives Considered

### Multiple Server Binaries
Rejected due to:
- combinatorial explosion
- maintenance overhead
- poor scaling of experiments

### Wrapper Binary Executing Other Binaries
Rejected due to:
- increased operational complexity
- weaker architectural clarity
- unnecessary indirection

---

## Related Decisions

- Design Note: Static vs Dynamic Dispatch at the REST Adapter Boundary
- ADR-0003: Hybrid Limiter Semantics
- ADR-0004: Distributed Failure Policy

---

## Notes

Static dispatch remains preserved in core limiter crates.  
Dynamic dispatch is isolated, reversible, and explicitly documented.

This ADR enables future milestones (M3.2+) without revisiting server wiring.

---

## Review Trigger

Revisit this decision if:
- adapter overhead becomes dominant in benchmarks
- static dispatch can be restored cleanly
- protocol adapters expand beyond REST


---
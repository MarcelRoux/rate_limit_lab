# ADR-0006: Compile-Time vs Runtime Limiter Selection in REST Adapter

## Status

Accepted

---

## Context

The project integrates multiple rate-limiter implementations:

- in-memory
- distributed (Redis/Valkey)
- hybrid (local + distributed)

Early iterations used multiple Axum binaries, leading to:

- duplicated wiring
- configuration drift
- poor experiment ergonomics

A single configurable REST server was adopted.

However, Rust’s type system enforces that:

- static dispatch decisions are made at compile time
- runtime selection requires type erasure or branching

This ADR formalizes how the REST adapter resolves this tension.

---

## Decision

### 1. Limiter Families Are Selected at Compile Time

Limiter *families* are selected using Cargo features:

- `in_memory_limiter`
- `distributed_limiter`
- `hybrid_limiter`

This preserves static dispatch and avoids vtables on the hot path.

---

### 2. Runtime Configuration Is Used for Parameters Only

Runtime config controls:

- quotas
- namespaces
- headers
- keying policy
- traffic allocation strategy

Runtime config does **not** select limiter *types*.

---

## Consequences

### Positive

- Static dispatch preserved where correctness matters
- Clear separation of concerns
- Adapter remains flexible
- Predictable performance characteristics

### Negative

- Requires rebuilding to switch limiter families
- Slightly more complex Cargo feature matrix

---

## Alternatives Considered

### Runtime Env-Based Type Selection

Rejected:

- impossible to preserve static dispatch
- misleading design
- hides performance costs

### Multiple Server Binaries

Rejected:

- scaling complexity
- maintenance overhead
- weak architectural signal

### Wrapper Binary Executing Other Binaries

Rejected:

- operational indirection
- decreased debuggability
- unnecessary abstraction

---

## Related Documents

- Design Note: Static vs Dynamic Dispatch at the REST Adapter Boundary
- ADR-0003: Hybrid Limiter Semantics
- ADR-0004: Distributed Failure Policy
- ADR-0005: Configurable REST Server

---

## Review Trigger

Revisit this ADR if:

- hybrid limiter semantics stabilize
- adapter overhead dominates benchmarks
- protocol adapters expand beyond REST

---

## Outcome

This decision balances:

- Rust’s type system constraints
- performance correctness
- experimental velocity

It enables continued progress through M3 without architectural debt.

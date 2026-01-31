# Design Note: REST Adapter Clone Semantics, Dispatch Strategy, and Per-Request Allocation

## Status

**Current state:**  
The REST adapter (`crates/adapters/rest`) supports both in-memory (synchronous) and
distributed (asynchronous) rate limiters using **static dispatch**.  
The limiter is held behind `Arc<L>`, and middleware/layers implement `Clone` **manually**
to satisfy Axum/Tower requirements without imposing unnecessary trait bounds.

**Key correction:**  
Dynamic dispatch (`Arc<dyn RateLimitPolicy>`) was initially adopted to unblock development,
but was later determined to be unnecessary. The real blocker was `#[derive(Clone)]`
introducing overly strict inferred bounds in a generic middleware context. A **manual
`Clone` implementation** restored static dispatch compatibility with Axum.

**Scope:**  
This note applies to the REST adapter only. Core crates
(`rate_limit`, `rate_limit_distributed`, `state_backend`) remain statically dispatched
and unaffected.

---

## Problem Statement

Axum/Tower requires middleware layers and the produced services to be:

- `Clone`
- `Send + Sync + 'static`

The REST adapter stored limiters behind `Arc<L>`, which is correct and inexpensive to
clone. However, using `#[derive(Clone)]` on generic layer/middleware types caused the
compiler to infer **unnecessary bounds** (e.g. `L: Clone`) due to the interaction between:

- generics,
- derived trait implementations,
- and Axum’s `Router::layer(...)` trait bounds.

This manifested as persistent errors such as:

- `RateLimitLayer<HierarchicalLimiter<...>>: Clone not satisfied`
- `RateLimitMiddleware<...>: Clone not satisfied`

These errors were **not** caused by the limiter lacking `Clone`. Some limiters embed
non-`Clone` components (e.g. `governor::RateLimiter`) by design. The issue was that
`#[derive(Clone)]` expressed a stronger requirement than was semantically necessary.

---

## Root Cause and Fix

### Why manual `Clone` works when `#[derive(Clone)]` failed

Both `RateLimitLayer<L>` and `RateLimitMiddleware<Svc, L>` store the limiter as `Arc<L>`.

Cloning an `Arc<L>`:
- **does not require `L: Clone`**
- only increments a reference count

However, `#[derive(Clone)]` in a generic context can introduce stricter bounds than needed,
especially when downstream trait constraints are involved (as with Axum).

By implementing `Clone` manually, we precisely express the real requirement:

- `Svc: Clone`
- `Arc<L>: Clone` (always true)

and avoid imposing `L: Clone`.

This resolved Axum compatibility **without dynamic dispatch**.

---

## Decision

- Use **static dispatch** in the REST adapter (`Arc<L>`).
- Implement `Clone` **manually** for middleware and layers to avoid unnecessary bounds.
- Support both sync (in-memory) and async (distributed) limiters through a shared policy
  interface, using boxed futures only where async is unavoidable.

Dynamic dispatch remains an optional fallback, but it is **not required**.

---

## Performance Considerations

### What remains on the hot path

After restoring static dispatch, the dominant overheads are:

1. **Boxed middleware future**
   - `Service::call` currently returns `Pin<Box<dyn Future<...>>>`
   - This introduces one heap allocation per request
2. **Async scheduling**
   - Required for distributed limiters
3. **Key allocation**
   - `&str` → `String` conversion in some paths

### What is *not* on the hot path anymore

- No vtable dispatch for in-memory limiters
- No loss of inlining due to `dyn` trait objects

### Observed behavior

- Debug builds showed reduced throughput (~260–270k requests / 5s)
- Release builds restored expected throughput (~300k requests / 5s)
- Differences between app-state and middleware approaches were dominated by
  build mode and future allocation, not dispatch strategy

---

## Why the Adapter Still Boxes Futures

The adapter must support both:

- synchronous limiters (in-memory)
- asynchronous limiters (distributed, Redis-backed)

Rust does not allow returning `impl Future` from trait methods used as trait objects.
Using a GAT (`type Fut<'a>`) allows concrete futures for statically dispatched paths
(e.g. `Ready<Decision>`), but the middleware `Service` itself still returns a boxed
future to satisfy Tower’s `Service` trait ergonomics.

This is a known and acceptable trade-off in Tower middleware.

---

## Future Investigations

The following tracks are worth exploring to further reduce overhead and complexity.

### Track A — Split Sync vs Async Middleware

- `RateLimitLayerSync<L>`
- `RateLimitLayerAsync<L>`

**Pros**
- No boxed futures for in-memory path
- Fully inlined, allocation-free checks

**Cons**
- Two middleware types to maintain
- Slightly more app wiring

---

### Track B — Enum-Based Static Erasure

```rust
enum AnyLimiter {
    InMemory(HierarchicalLimiter<...>),
    Distributed(DistributedKeyedLimiter<...>),
}
```

**Pros**
- Static dispatch, no vtable
- Single middleware

**Cons**
- Adapter must know supported limiter variants
- Harder to extend externally

---

### Track C — Avoid Per-Request Key Allocation
- Pass &str deeper into core where possible
- Use Cow<'a, str> or &str-based APIs
- Add check_str(&self, key: &str) to distributed limiter

---

### Track D — Reduce Middleware Future Boxing
- Explore named future types for Service::call
- Investigate Tower patterns that avoid boxing
- Likely lower priority than key allocation

---

## Lessons Learned
1.	#[derive(Clone)] is not always harmless in generic middleware.
2.	Arc<T> cloning does not require T: Clone.
3.	Manual trait implementations are sometimes the most precise expression of intent.
4.	Performance regressions should be validated in release mode before architectural changes.
5.	Adapter boundaries are appropriate places for pragmatic trade-offs — but those trade-offs
should be documented and revisited.

---

## Conclusion

The REST adapter now:
- compiles cleanly with Axum/Tower
- supports both in-memory and distributed limiters
- uses static dispatch correctly
- isolates unavoidable overheads (async + boxed futures)

This design unblocks M3.1 work while preserving a clear path toward further optimization.
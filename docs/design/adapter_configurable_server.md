# Design Note: Static vs Dynamic Dispatch at the REST Adapter Boundary

## Status

Accepted (with constraints)

This design documents how the REST adapter balances **static dispatch**, **runtime configurability**, and **Axum/Tower constraints**, and how future work may further reduce overhead.

---

## Problem Statement

The REST adapter integrates rate limiters into Axum via Tower middleware.

Axum imposes strict trait bounds on middleware layers:

- the layer must be `Clone + Send + Sync + 'static`
- the produced service must also be `Clone`

Core limiter implementations (in-memory, distributed, hybrid) intentionally include non-Clone components (e.g. `governor::RateLimiter`), making naïve generic middleware definitions fail to compile.

Earlier attempts to keep everything statically dispatched resulted in:

- confusing trait bound failures
- misleading `Clone` errors on otherwise valid types
- disproportionate debugging time relative to feature scope

---

## Current Design

### Key Principles

1. **Static dispatch is preserved in core crates**
   - `rate_limit`
   - `rate_limit_distributed`
   - `state_backend`

2. **The REST adapter is an integration boundary**
   - HTTP parsing
   - header extraction
   - async scheduling
   - response construction

3. **Adapter correctness and ergonomics take priority**
   - hot-path performance still matters
   - but adapter is not the primary computational bottleneck

---

## Adapter Strategy—  Compile-Time Selection (Static Dispatch)

Limiter *families* are selected at **build time** using Cargo features.

Example:

- `--features inmem`
- `--features distributed`
- `--features hybrid`

The adapter remains generic:

```rust
pub struct RateLimitLayer<L> {
    limiter: Arc<L>,
}
```

and uses a **manual Clone impl** to avoid imposing `L: Clone`.

Each build produces a fully statically-dispatched middleware stack.

#### Properties

- No `dyn Trait`
- No vtable dispatch
- No runtime branching on limiter type
- Requires rebuilding to switch limiter families

This is the preferred model for:

- performance evaluation
- production-like benchmarking
- correctness-sensitive measurements

---

## Why Manual Clone Works

The adapter wraps the limiter in `Arc<L>`.

Cloning an `Arc<T>` does **not** require `T: Clone` — only the pointer is cloned.

However, `#[derive(Clone)]` may infer overly strict bounds in generic contexts.

Manual implementation avoids this:

```rust
impl<L> Clone for RateLimitLayer<L> {
    fn clone(&self) -> Self {
        Self {
            limiter: self.limiter.clone(),
        }
    }
}
```

This expresses the *minimal correct requirement*.

---

## Performance Considerations

### Costs Mitigated by Static Dispatch over Dynamic Dispatch

1. One indirect call (vtable) per request
2. Loss of cross-boundary inlining
3. Possible heap allocation if policy returns boxed futures

In REST workloads:

- HTTP overhead dominates
- dynamic dispatch cost is measurable but not dominant

In hot-path benchmarking:

- static dispatch is preferred and supported via features

---

## What Environment Variables Cannot Do

Environment variables cannot influence:

- monomorphization
- generic specialization
- static vs dynamic dispatch

They control **configuration**, not **types**.

---

## Future Improvement Tracks

### Track 1 — Split Sync vs Async Middleware

- `RateLimitLayerSync`
- `RateLimitLayerAsync`
- removes boxed futures for in-memory path

### Track 2 — Key Handling Optimization

- avoid `key.to_string()`
- prefer `&str` or `Cow<'a, str>`
- push allocation to adapter boundary only if unavoidable

---

## Acceptance Criteria for Revisiting Design

Reconsider the adapter design if:

1. Adapter overhead becomes a dominant factor in benchmarks
2. Hybrid limiter semantics are finalized (M3.2+)
3. Additional protocols (gRPC) require shared adapter abstractions

---

## Conclusion

Static dispatch is preserved.

This design unblocks progress while maintaining architectural integrity and performance discipline.

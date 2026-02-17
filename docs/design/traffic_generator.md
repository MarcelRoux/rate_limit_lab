# Traffic Generator Investigation Summary (M2.4)

## Motivation

The goal of this investigation was to build a reliable and interpretable REST traffic generator capable of exercising rate-limiting behavior across a wide range of request rates and concurrency levels. In particular, the generator needed to:  

- Sustain high configured RPS (e.g., 60k/s) without relying on sub-millisecond timers  
- Behave predictably at low and high concurrency  
- Make overload conditions explicit rather than implicit  
- Avoid conflating client-side capacity limits with server-side rate limiting behavior  

## Investigation

Two pacing approaches were explored:  

 1. Tick-based batching with semaphore-controlled concurrency  

- Requests are generated in batches at fixed tick intervals (1ms).  
- A semaphore bounds the number of in-flight requests.  
- When concurrency is saturated, pacing naturally slows.  

 1. Pacer → bounded queue → fixed worker pool  

- A pacer generates send tokens at a fixed RPS.  
- Tokens are enqueued into a bounded queue.  
- A fixed number of workers execute requests.  
- Explicit counters track attempted, enqueued, dropped, and completed requests.  

The second design was implemented to decouple arrival pacing from execution capacity and to observe overload behavior directly.

## Findings

- The queue/worker architecture behaves correctly and provides valuable insight into overload dynamics.  
- At low concurrency, the generator becomes capacity-limited, with excess tokens intentionally dropped.  
- Metrics such as attempted, enqueued, and dropped clearly expose client-side saturation and make generator behavior explicit.  
- However, this approach introduces additional complexity (queue coordination, worker lifecycle management, channel backpressure semantics) that is not strictly required to meet M2.4 goals.  

## Decision

For M2.4, the project proceeds with the simpler batch-per-tick generator using semaphore-bounded concurrency, which:  

- Is easier to reason about and maintain  
- Produces stable, repeatable load patterns  
- Is sufficient for evaluating single-node REST rate limiting behavior  
- Avoids premature complexity in the traffic generator itself  

The queue/worker implementation is retained on a feature branch for future reference and comparison.

## Future Improvements (Deferred)

If higher fidelity or throughput is required in later milestones, the following enhancements may be revisited:  

- Reintroducing the pacer/queue/worker model as an optional generator mode  
- Supporting configurable drop vs backpressure semantics
- Using dedicated worker pools to reduce per-request task spawn overhead  
- Improving timer precision or using time-sliced pacing strategies  
- Introducing CPU affinity or runtime tuning for very high RPS scenarios  

These improvements are intentionally deferred to keep the current milestone focused and the evaluation framework approachable.

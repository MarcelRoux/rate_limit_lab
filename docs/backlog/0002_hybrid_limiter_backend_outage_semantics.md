# Backlog Item: Hybrid limiter semantics during long backend outage (degraded local fallback)

Status: Proposed
Priority: P1
Milestone: M3.x
Owner: TBD
Created: 2026-02-06
Links:

- ADR: docs/adr/ADR-xxxx-hybrid-limiter-backend-failure-semantics.md
- Related: crates/rate_limit_hybrid, crates/rate_limit_distributed, traffic generator scenarios

---

## Summary

Define and implement configurable hybrid limiter behavior for prolonged distributed backend unavailability, enabling empirical evaluation of availability vs safety tradeoffs under RR and SA routing.

## Motivation

A long-lived distributed backend outage creates a fundamental tradeoff:

- Fail-open preserves availability but risks abuse and violates global constraints.
- Fail-closed preserves safety but can cause total outage and loss of platform trust.

We need configurable semantics to measure outcomes, not guess.

## Proposed approach

Introduce explicit outage semantics in the hybrid limiter:

- STRICT_FAIL_CLOSED: allow only while existing lease remains valid; otherwise deny.
- FAIL_OPEN: allow when backend is unavailable.
- DEGRADED_LOCAL_FALLBACK (recommended default):
  - after an outage_grace, enable conservative local refill behavior
  - keep attempting distributed refresh in background
  - optional max_outage threshold that transitions to fail-closed

Add tuning options:

- fallback_rate, fallback_burst
- per_instance_cap / assumed_cluster_size scaling
- jitter/backoff for refresh attempts
- global vs per-key fallback scope

## Scope

In-scope:

- Configuration model + state machine for outage modes
- Background refresh mechanism (non-request path)
- Instrumentation counters/events for mode transitions and drift
- Traffic scenarios to compare RR vs SA behavior

Out-of-scope:

- Perfect global fairness during outage (best-effort only)
- Algorithm unification (GCRA vs fixed-window) unless required by acceptance criteria

## Acceptance criteria

- [ ] Hybrid limiter exposes a config that selects outage behavior.
- [ ] Under simulated backend outage, behavior matches selected mode.
- [ ] Measurable instrumentation emitted for transitions and error counts.
- [ ] Traffic generator scenarios exist for RR and SA comparisons.

## Risks / tradeoffs

- Under RR, per-instance fallback can inflate effective rate by ~N.
- Background refresh introduces concurrency/timing complexity.
- Lease semantics must be clearly defined to avoid false correctness claims.

## Dependencies

- Existing hybrid limiter skeleton in crates/rate_limit_hybrid
- Distributed limiter lease/allocation model (M3.2)
- Failure injection scaffolding (M3.3 may expand this)

## Validation plan

- Add deterministic tests with a fake backend that simulates prolonged failures.
- Run traffic scenarios under:
  - normal operation
  - backend fail short/long
  - instance churn
- Measure drift, denial rates, and latency.

## Outcome

(leave blank until closed)

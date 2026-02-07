# Backlog Item: Rate limit model evolution

Status: Proposed
Priority: P1
Milestone: M3.2
Owner: TBD
Created: 2026-02-07
Links:
- ADR: ADR-0003-hybrid-limiter-semantics.md
- ADR: ADR-0004-distributed-failure-policy.md

---

## Summary
The hybrid limiter and its supporting adapters now need richer instrumentation to describe backend errors, retry policies, and failure signals. Evolving the `rate_limit` crate’s `Decision`/`Event` model to surface that metadata is the natural next step to keep the core and hybrid layers aligned, rather than working around the limited shapes that predated M3.2.

## Motivation
- A more expressive model lets hybrid (and future adapters) describe fail-open/fail-closed behavior and backend retries without inventing ad-hoc translations.
- Success looks like the core types being enhanced (rather than restricted) while still offering backward-compatible helpers for any consumer that expects the existing shapes.

## Proposed approach
- Extend `crates/rate_limit/src/models.rs` so `Decision`/`Event`/`InstrumentationLevel` capture backend error metadata and failure policy annotations.
- Provide small adapter helpers for parts of the codebase (hierarchical limiter, REST adapters) that should continue to operate with the legacy view of allow/deny - or, ideally, update all affected code workspace-wide.
- Document the evolution in ADR-0003 and ensure hybrid instrumentation references this backlog entry as part of the forward momentum.

## Scope
In-scope:
- Enhancing the core `Decision/Event` definitions and any helpers needed for richer payloads.
- Updating comments/tests/docs that explain the new semantics.

Out-of-scope:
- Rolling back the richer model; the goal is cumulative improvement.

## Acceptance criteria
- [ ] `rate_limit::Decision` and `rate_limit::Event` describe normal allows/denies plus backend failure metadata.
- [ ] Legacy consumers still compile via compatibility helpers/adapters - or definition updates.
- [ ] Comments/ADRs that previously pointed to this backlog item now present it as the guiding evolution.

## Risks / tradeoffs
- Risk: The richer model may be misunderstood or misused. Mitigate by providing clear examples and a short migration guide.
- Risk: Additional helpers add code. Keep them constrained and highlight they exist primarily for compatibility.

## Dependencies
- Coordination with distributed failure policy work tracked in docs/backlog/0004-distributed-keyed-limiter-revision.md.

## Validation plan
- `cargo test -p rate_limit`.
- Confirm hybrid/REST adapters log the new event payloads without extra refactors.

## Outcome
(leave blank until closed)

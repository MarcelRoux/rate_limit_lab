# Backlog Item: Valkey backend support in state_backend

Status: Proposed
Priority: P1
Milestone: M3.x
Owner: TBD
Created: 2026-02-06
Links:

- Related: crates/state_backend, distributed limiter experiments (M3.1+)

---

## Summary

Add Valkey support as an additional Redis-compatible state backend to validate portability and reduce vendor lock-in risk while preserving current Redis-first scope.

## Motivation

- Valkey is a Redis-compatible fork that may be preferable in some environments.
- Supporting Valkey early reduces future migration risk and keeps the backend interface honest.
- This also acts as a “compatibility test” of the StateBackend trait and Lua scripts.

## Proposed approach

- Treat Valkey as Redis-protocol compatible and reuse the same backend implementation initially.
- Add explicit documentation + configuration allowing the same RedisBackend to connect to Valkey.
- Add a targeted compatibility test suite:
  - basic allow/deny
  - key isolation
  - window rollover
  - script compatibility smoke test (if Lua is used)

## Scope

In-scope:

- Verify RedisBackend works against Valkey.
- Add docker service for Valkey in dev compose.
- Feature-gated integration test(s) that can run against Valkey.

Out-of-scope:

- A separate Valkey-specific backend implementation unless incompatibilities arise.

## Acceptance criteria

- [ ] Dev environment can start Valkey via docker compose.
- [ ] StateBackend integration tests pass against Valkey (feature-gated).
- [ ] Documented env var(s) and usage (REDIS_URL points to Valkey endpoint).

## Risks / tradeoffs

- “Redis compatible” is not always perfectly compatible (edge commands, Lua differences, time behavior).
- If incompatibilities exist, might require conditionals or a separate backend type.

## Dependencies

- docker compose dev services
- state_backend crate integration tests

## Validation plan

- Run integration tests against Redis and Valkey.
- Compare retry_after semantics and correctness under load.

## Outcome

(leave blank until closed)

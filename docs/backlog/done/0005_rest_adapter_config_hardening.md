# Backlog Item: REST adapter configuration hardening

Status: Closed
Priority: P1
Milestone: M3.x
Owner: TBD
Created: 2026-02-08
Links:

- ADR: docs/adr/ADR-0005-configurable-rest-server.md
- ADR: docs/adr/ADR-0006_compile_time_vs_runtime_limiter_selection_in_rest_adapter.md
- Related: docs/design/adapter_configurable_server.md

---

## Summary

Stabilize runtime configuration for the REST server while preserving compile-time limiter family selection. This includes validated config loading, feature exclusivity guards, and clear runtime logging for experiment execution.

## Motivation

- Reduce server wiring drift across limiter variants.
- Make experiment setup reproducible via checked TOML configs.
- Keep compile-time selection explicit and safe as limiter variants expand.

## Proposed approach

- Centralize server config parsing and typed defaults in the REST adapter.
- Enforce exactly one limiter family feature at build time.
- Keep runtime config scoped to parameters only (quotas, keys, headers, instrumentation).
- Align documentation with ADR-0006 and the design note.

## Scope

In-scope:

- `crates/adapters/rest/src/config.rs`
- `crates/adapters/rest/src/build.rs`
- `crates/adapters/rest/src/bin/rest_server.rs`
- `configs/rest_server/*.toml`
- Supporting docs updates for usage and decision rationale

Out-of-scope:

- Runtime limiter type switching.
- Hybrid limiter implementation details.

## Acceptance criteria

- [x] Build fails when zero or multiple REST limiter family features are enabled.
- [x] REST server supports config-file driven parameterization with clear defaults.
- [x] In-memory and distributed variants each have a working example config file.
- [x] Startup logs expose effective runtime settings needed for experiment traceability.

## Risks / tradeoffs

- Risk: Config shape drift across limiter families. Mitigation: keep shared top-level fields and variant-specific nested blocks.
- Risk: Feature matrix complexity grows over time. Mitigation: centralize variant list and guard logic in `build.rs`.

## Dependencies

- `clap`, `serde`, `toml`, `log`, `env_logger` in `crates/adapters/rest`.
- ADR-0006 constraints on compile-time vs runtime selection.

## Validation plan

- `cargo check -p rest`
- `cargo check -p rest --no-default-features --features in_memory_limiter`
- `cargo check -p rest --no-default-features --features distributed_limiter`
- Manual smoke run with both `configs/rest_server/in_memory.toml` and `configs/rest_server/distributed.toml`

## Outcome

- Mutually exclusive, required feature specification allows for compile-time limiter selection ensuring efficient runtime performance.
- REST server config-file drive parameterization enables efficient configuration based on specified file with sensible defaults.
- Working config files provide for in memory and distributed limiter variants.
- Startup logs expose effective runtime settings.

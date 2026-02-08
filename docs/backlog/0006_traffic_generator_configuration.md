# Backlog Item: Traffic generator configuration model

Status: Proposed
Priority: P1
Milestone: M3.x
Owner: TBD
Created: 2026-02-08
Links:
- Related: docs/design/traffic_generator.md
- Related: docs/design/adapter_configurable_server.md
- Related: crates/traffic_rest/src/bin/rest_traffic.rs

---

## Summary
Introduce configuration-driven execution for the REST traffic generator so workload profiles can be reproduced without source edits. This should align traffic config structure with REST server config conventions where practical.

## Motivation
- Current traffic runs require hard-coded profile values in `rest_traffic.rs`.
- Comparative experiments need repeatable, shareable inputs.
- Harmonized config patterns lower operator friction across server and client tooling.

## Proposed approach
- Add typed config loading for traffic profile and key mode settings.
- Support CLI path selection with sensible defaults.
- Validate invalid or contradictory profile values early with explicit errors.
- Keep runtime behavior unchanged except for how inputs are supplied.

## Scope
In-scope:
- `crates/traffic_rest/src/bin/rest_traffic.rs`
- New/updated traffic config model files in `crates/traffic_rest`
- Example config files under `configs/` for traffic runs

Out-of-scope:
- New traffic pacing algorithms.
- Benchmark automation/report generation.

## Acceptance criteria
- [ ] Traffic binary can load profile settings from a TOML config file.
- [ ] Key mode and key distribution can be configured without code edits.
- [ ] Invalid config values fail fast with actionable error messages.
- [ ] Existing default behavior remains available when no config file is provided.

## Risks / tradeoffs
- Risk: Config complexity can obscure simple usage. Mitigation: provide compact defaults and one minimal sample config.
- Risk: Divergence from server config semantics. Mitigation: reuse naming conventions for shared concepts (headers, key modes, target).

## Dependencies
- Existing `traffic_rest` model and runner abstractions.
- Coordination with server config fields to avoid naming drift.

## Validation plan
- Unit tests for config parsing and validation.
- Integration smoke run against REST server using at least one in-memory and one distributed server config.

## Outcome
(leave blank until closed)

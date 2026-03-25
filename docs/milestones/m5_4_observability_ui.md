# Milestone M5.4 — Observability UI (Optional)

## Status

Planned (optional milestone; not started)

## Goal

Add an optional live observability path for local and evaluation runs, using a metrics pipeline and Grafana dashboards, without changing default behavior of existing binaries.

This milestone exists to make runtime behavior immediately visible during experiments, reduce debugging latency, and improve production-grade system operability signals for the project.

## Scope and Milestone Position

- Parent milestone family: `M5` (usability and validation).
- Sub-milestone: `M5.4`.
- Delivery mode: optional capability, feature-gated and config-gated.

## Proposed Implementation Components

- Optional metrics instrumentation/export path in runtime binaries.
- Optional scrape pipeline for local evaluation runs.
- Provisioned Grafana dashboard bundle for core SLO and correctness signals.
- Harness/report evidence fields linking run artifacts to live metrics snapshots.

Candidate implementation locations:

- `crates/adapters/rest` (runtime metrics export toggle)
- `crates/eval_harness` (observability evidence capture and validation hooks)
- `docker/compose/compose.dev.yml` (optional Prometheus + Grafana profile)
- `configs/` (observability-enabled runtime and scrape config)

## Decision Semantics

- Default mode remains unchanged: no live metrics pipeline required.
- Observability UI mode is opt-in:
  - compile-time feature gate,
  - runtime configuration gate.
- If observability mode is requested:
  - metrics endpoint must be reachable,
  - scrape pipeline must be healthy,
  - dashboard provisioning must succeed for acceptance.
- Live checks are opt-in for local runs via `EVAL_OBS_LIVE=1` (or `make ac-obs` wrapper).

### MVP Pipeline Choice

MVP path is:

- Prometheus-compatible metrics endpoint + scrape + Grafana dashboard.

Follow-on path:

- Add OpenTelemetry export once MVP dashboard loop is stable.

## Failure Behavior

- If observability mode is `off`, missing live pipeline artifacts are not failures.
- If observability mode is `on`, missing required observability evidence is a hard fail for observability ATs.
- Failure conditions must be surfaced with explicit triage labels and report evidence links.

## Out of Scope

- Mandatory production deployment of monitoring stack.
- Multi-tenant dashboard authorization model.
- Alert routing/on-call workflows.
- Long-term metrics retention governance beyond local/evaluation needs.

## Acceptance Checklist

- [ ] Optional observability feature gate exists and defaults to disabled.
- [ ] Runtime metrics endpoint is available and scrapes successfully in observability mode.
- [ ] Grafana dashboard provisioning works and includes core panels (throughput, deny ratio, p95 latency, backend policy conformance).
- [ ] Harness/report outputs include observability evidence links when observability mode is enabled.
- [ ] Documentation clearly distinguishes required baseline harness behavior from optional observability mode.

## Linked Acceptance Tests

- `AT-052` — Observability feature gate + default-off contract. (implemented)
- `AT-053` — Runtime metrics endpoint availability. (implemented)
- `AT-054` — Prometheus scrape config contract validation. (implemented)
- `AT-055` — Grafana provisioning/dashboard contract validation. (implemented)
- `AT-056` — Run/report observability evidence linkage. (implemented)

## Developer Commands

- `make ac-obs` runs `observability_mvp` with live checks enabled.
- `make ac-one AT=AT-054` / `AT-055` / `AT-056` run contract checks by default.
- `EVAL_OBS_LIVE=1 make ac-one AT=AT-054` (or `AT-055`) enables live probe mode for those ATs.
- Live probe mode uses containerized demo services (REST + Prometheus + Grafana) rather than host-started REST processes.

## Related Documents

- `docs/evaluation/acceptance_criteria_matrix.md`
- `docs/evaluation/scenario_catalog.md`
- `docs/evaluation/acceptance_harness.md`
- `docs/evaluation/product_facets_overview.md`

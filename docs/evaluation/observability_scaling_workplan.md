# Observability Stack Scaling Workplan

Last updated: 2026-03-24

## Purpose

Define the minimum work needed to scale from a local containerized observability demo to a reliable, repeatable, multi-case evaluation workflow.

Target workflow:

1. bring stack up once
2. run case A
3. run case B
4. observe in Grafana
5. tear stack down

## Current Baseline

- Containerized observability demo exists (`rest_observability`, Prometheus, Grafana).
- One containerized traffic case is available for the observability loop.
- Harness live checks can validate observability artifacts and live endpoints.
- Core fragility reduced by removing host-process dependence from demo path.

## Scaling Goals

1. Multi-case execution without stack restart.
2. Deterministic case selection and cataloging.
3. Fast feedback loop for adding/running new cases.
4. Clear separation of demo mode vs acceptance/harness mode.
5. Low operational surprise (stable commands, clean teardown).

## Phase Plan

### Phase S1: Case Model And Catalog

Scope:

- Formalize case metadata (id, config path, intent, expected signal).
- Define container-compatible config requirements.
- Add case registry file under `configs/traffic_rest/`.

Deliverables:

- `configs/traffic_rest/case_catalog.toml` (or JSON).
- Naming convention and validation rules for case configs.
- At least 5 curated cases (steady, burst, deny-heavy, multi-key, edge profile).

Exit criteria:

- Case lookup by ID is deterministic.
- Invalid case definitions fail early with explicit messages.

### Phase S2: Command Surface For Multi-Case Loop

Scope:

- Add focused command wrappers for the intended loop.
- Keep command contracts small and stable.

Target commands:

- `make obs-up`
- `make obs-case CASE=<case-id-or-path>`
- `make obs-cases CASES="<case-a> <case-b> ..."`
- `make obs-down`

Deliverables:

- `scripts/obs/up.sh`
- `scripts/obs/case.sh`
- `scripts/obs/cases.sh`
- `scripts/obs/down.sh`
- Makefile targets wired to those scripts.

Exit criteria:

- Stack starts once and remains up between case runs.
- Multiple cases run sequentially without manual plumbing.
- Teardown leaves no observability containers running.

### Phase S3: Result Capture Per Case

Scope:

- Persist case-run evidence in a predictable structure.
- Keep observability run artifacts separate from acceptance harness artifacts.

Deliverables:

- `evaluations/obs_runs/<stamp>_<case-id>/` artifacts:
  - `case_manifest.json`
  - `traffic_summary.json`
  - `prometheus_snapshot.json`
  - `grafana_snapshot.json`
- optional roll-up: `evaluations/obs_runs/<stamp>_batch/summary.json`

Exit criteria:

- Each case run has a durable artifact bundle.
- Artifacts are linkable from markdown summaries.

### Phase S4: Performance And Reliability Hardening

Scope:

- Reduce unnecessary rebuilds in case loop.
- Improve resilience to transient startup and scrape delays.

Deliverables:

- Prebuilt/pinned demo image strategy.
- Config-only changes avoid full image rebuild when possible.
- Health/readiness waits with bounded retries and actionable failure output.

Exit criteria:

- Typical case loop runs without rebuild unless code changed.
- Startup/scrape failures produce deterministic diagnostics.

### Phase S5: Acceptance Integration

Scope:

- Integrate curated observability cases into acceptance criteria where appropriate.
- Keep optional observability mode explicit and non-disruptive.

Deliverables:

- Mapping from observability cases to AT coverage in docs.
- Harness hooks for selected multi-case observability checks.
- Updated milestone docs and scenario catalog links.

Exit criteria:

- Optional observability checks can run in CI/manual gates with clear pass/fail semantics.

## Non-Goals

- Full production monitoring platform design.
- Long-term metrics retention architecture.
- Alerting/on-call workflow automation.

## Risks And Mitigations

1. Risk: command sprawl and UX complexity.
   Mitigation: keep 4 primary commands; hide advanced operations behind scripts.
2. Risk: case drift and undocumented configs.
   Mitigation: central catalog + validation + naming policy.
3. Risk: flaky startup timing.
   Mitigation: bounded readiness checks and explicit error surfaces.
4. Risk: rebuild-heavy loops.
   Mitigation: image caching strategy and config mount approach.

## Definition Of Done (Scaling)

- A developer can run:
  1. `make obs-up`
  2. `make obs-case CASE=<A>`
  3. `make obs-case CASE=<B>`
  4. observe Grafana
  5. `make obs-down`
- At least 5 documented cases exist and are runnable by ID.
- Each case emits evidence artifacts suitable for later report compilation.
- Observability stack teardown is deterministic and leaves no demo containers running.

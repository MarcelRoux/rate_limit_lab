# Latest Execution Progress (Canonical)

Last updated: 2026-03-28

This is the single authoritative execution tracker for the repository.

## Canonical Policy

1. This file is the only authoritative checklist for execution status.
2. If other docs conflict with this file, this file wins for execution state.
3. Do not create new tracker/checklist docs; append/update here only.
4. Every checked item must include short evidence (command, artifact path, or file path).

## Current Status

- Harness core: implemented and operational.
- Command contracts: implemented (`make ac`, `make ac-full`, `make ac-one`, `make ac-obs`).
- Observability demo: containerized (`rest_observability` + Prometheus + Grafana + traffic container).
- Biggest remaining gap: governance acceptance gates (`AT-047`, `AT-048`, `AT-049`) and backend evidence capture (Redis/Valkey).

## Now / Next / Blocked

- `Now`: complete remaining M5 governance ATs (`AT-047`, `AT-048`, `AT-049`) and enforce them as formal gates.
- `Next`: run and capture backend validation evidence (Redis/Valkey).
- `Blocked`: environment-specific backend validation evidence (Redis/Valkey) depends on local runtime availability and explicit execution.

---

## Milestone Execution Checklist

### M0 — Developer Environment Foundation

- [x] Hooks exist and install script is present. (evidence: `githooks/*`, `scripts/install-hooks.sh`)
- [ ] Confirm hooks are active on current machine. (verification: `git config --get core.hooksPath`)

### M1 — Local Hierarchical Limiter

- [x] Strict AND limiter semantics implemented. (evidence: `crates/rate_limit/src/hierarchical_limiter.rs`)
- [x] Retry-after uses max-deny rule. (evidence: `crates/rate_limit/src/hierarchical_limiter.rs`)
- [x] M1 tests pass. (verification: `cargo test -p rate_limit`)

### M2 — REST Adapter + Traffic Generator

- [x] REST allow/deny mapping implemented (`429` + `retry-after`). (evidence: `crates/adapters/rest/src/middleware.rs`)
- [x] Key extraction behavior implemented. (evidence: `crates/adapters/rest/src/extractor.rs`)
- [x] Traffic generator implemented. (evidence: `crates/traffic_rest/src/*`)
- [x] REST tests pass. (verification: `cargo test -p rest`)

### M3.1 — Distributed Keyed Limiter

- [x] Backend abstraction + Redis scripts implemented. (evidence: `crates/state_backend/src/*`)
- [x] Distributed decision mapping implemented. (evidence: `crates/rate_limit_distributed/src/distributed_keyed_limiter.rs`)
- [x] Error-path fallback behavior implemented. (evidence: `BACKEND_ERROR_RETRY_AFTER`)
- [ ] Run and capture Redis integration evidence. (verification: `cargo test -p state_backend --features redis-tests`)
- [ ] Run and capture Valkey integration evidence. (verification: Valkey-backed integration command + artifact note)

### M3.2 — Hybrid Limiter

- [x] Hybrid limiter implemented (local + distributed composition). (evidence: `crates/rate_limit_hybrid/src/lib.rs`)
- [x] Option A2 ordering implemented. (evidence: `check` path in hybrid limiter)
- [x] Fail-open/fail-closed policies implemented and tested. (verification: `cargo test -p rate_limit_hybrid`)
- [x] REST hybrid wiring implemented. (evidence: `crates/adapters/rest/src/bin/rest_server.rs`, `configs/rest_server/hybrid.toml`)

### M3.3 / M3.4 — Failure Injection + Topology Metrics

- [x] Harness AT coverage for `AT-025`..`AT-034` is wired. (evidence: `crates/eval_harness/src/at_engine.rs`)
- [ ] Replace any remaining deterministic placeholder/scaffold behavior with command-backed execution evidence where applicable.
- [ ] Capture baseline evidence set for outage/latency/flapping + RR/SA comparison runs in `evaluations/runs/`.
- [ ] Validate repeatable failure/timeline artifacts across repeated executions.

### M3.5 — Hybrid Consolidation (Planned)

- [ ] Implement lease/refresh architecture.
- [ ] Implement degraded fallback mode and outage state machine.
- [ ] Implement recovery/re-sync behavior and validation evidence.

### M4 — gRPC Parity (Planned)

- [ ] Unary parity with REST semantics.
- [ ] Streaming parity (server/client/bidi).
- [ ] gRPC report schema parity with REST artifacts.

### M5 — Usability + Acceptance Productization

- [x] `make ac` works end-to-end. (evidence: recent `evaluations/runs/*_smoke_ready`)
- [x] `make ac-full` works end-to-end and includes observability ATs. (evidence: `*_full_matrix/summary.json` with `AT-052..AT-056`)
- [x] `make ac-one` works for single-AT loops. (evidence: recent `*_AT-0xx` runs)
- [x] Required run artifacts emitted (`manifest`, `preflight`, `traces`, `summary`, `triage`, reports).
- [ ] Close governance ATs:
  - [ ] `AT-047` baseline update governance check
  - [ ] `AT-048` multi-run comparative report generation quality gate
  - [ ] `AT-049` reproducibility gate enforcement as formal acceptance gate

### M5.4 — Observability UI (Optional)

- [x] Runtime metrics endpoint is feature-gated and implemented with live counters/gauge. (evidence: `crates/adapters/rest/src/observability.rs`)
- [x] Middleware emits request/deny/latency metrics. (evidence: `crates/adapters/rest/src/middleware.rs`)
- [x] Grafana datasource UID aligned with dashboard references. (evidence: `docker/observability/grafana/provisioning/datasources/datasource.yml`)
- [x] Containerized observability demo implemented. (evidence: `docker/compose/compose.dev.yml`, `docker/observability/rest/Dockerfile`, `scripts/obs/demo.sh`)
- [x] Live harness checks run through containerized path. (evidence: `crates/eval_harness/src/at_engine.rs`, `evaluations/runs/20260325_021848_AT-054/*`)
- [x] Multi-case observability loop:
  - [x] `make obs-up` (verification: 2026-03-28 local run)
  - [x] `make obs-case CASE=<id-or-path>` (evidence: `evaluations/obs_runs/20260328_213811_obs_case/OBS-001/result.json`)
  - [x] `make obs-cases CASES="<case-a> <case-b> ..."` (evidence: `evaluations/obs_runs/20260328_213818_obs_batch/summary.json`)
  - [x] `make obs-down` (verification: 2026-03-28 local run)
- [x] Case catalog + curated cases:
  - [x] Add case registry file with stable IDs. (evidence: `configs/traffic_rest/observability/case_registry.tsv`)
  - [x] Add at least 5 curated container-ready cases. (evidence: `OBS-001`..`OBS-005` in registry)
  - [x] Add validation for bad/unknown case IDs. (verification: `make obs-case CASE=UNKNOWN` exits non-zero)
- [x] Case artifact model:
  - [x] Persist per-case observability artifacts under `evaluations/obs_runs/...`. (evidence: `evaluations/obs_runs/20260328_213818_obs_batch/OBS-003/*`)
  - [x] Add batch summary for multi-case execution. (evidence: `evaluations/obs_runs/20260328_213818_obs_batch/summary.json`)

---

## ADR Checklist (Execution-Relevant)

- [x] Core ADRs for limiter architecture exist (`ADR-0001`..`ADR-0006`).
- [x] Containerized observability architecture ADR exists (`ADR-0007`).
- [ ] Resolve placeholder ADR filename/content:
  - [ ] `docs/adr/ADR-xxxx-hybrid-limiter-backend-failure-semantics.md` must be finalized or removed.

---

## Definition Of Complete

Project is "complete" for this repo scope when all are true:

1. All `ready` acceptance tests are executed with artifact-backed evidence.
2. No scaffold-only pass conditions remain for ready ATs.
3. `make ac`, `make ac-full`, and `make ac-one` remain stable and reproducible.
4. Multi-case observability workflow is implemented and produces per-case artifacts.
5. Governance ATs (`AT-047/048/049`) are implemented and enforced.
6. Redis + Valkey integration evidence is captured and linked in this file.
7. Placeholder ADR(s) are resolved.

---

## Immediate Next Commands

1. `make ac-one AT=AT-047`
2. `make ac-one AT=AT-048`
3. `make ac-one AT=AT-049`
4. `make ac-full`
5. `make report`

# Acceptance Harness Specification (Rate Limiting Repo)

## 1. Purpose

Define a deterministic, artifact-backed acceptance harness for evaluating limiter correctness, resilience, and performance against explicit milestone criteria.

This document is execution-oriented and assumes manual orchestration (CI wiring is intentionally out of scope for now).
It defines how to evaluate both implemented features and planned features via contract-first acceptance criteria.

## 2. Normative Sources

If this file conflicts with the following, those files win:

- `docs/DESIGN.md`
- `docs/milestones/M1-local-hierarchical.md`
- `docs/milestones/M2-multi-level-limiting.md`
- `docs/milestones/M3.1-distributed-keyed.md`
- `docs/milestones/M3.2-hybrid-limiting.md`
- `docs/testing/M3.2-hybrid-tests.md`
- `docs/adr/ADR-0001-hierarchical-and-retry-after.md`
- `docs/adr/ADR-0002-distributed-backend-abstraction.md`
- `docs/adr/ADR-0003-hybrid-limiter-semantics.md`
- `docs/adr/ADR-0004-distributed-failure-policy.md`
- `docs/evaluation/acceptance_criteria_matrix.md`
- `docs/evaluation/scenario_catalog.md`
- `docs/evaluation/metric_definitions.md`
- `docs/evaluation/baseline_policy.md`
- `docs/evaluation/shortcomings_and_remediation.md`

## 3. Scope

In-scope now:

- M1 and M2 correctness and behavior validation.
- M3.1 distributed keyed behavior validation.
- M3.2 hybrid strict-AND and failure-policy validation.
- Manual reproducibility checks with fixed config and seed.

Contract-defined now, execution expands as milestones land:

- M3.3 failure injection matrix.
- M3.4 distributed metrics validation.
- M3.5 lease/refresh/degraded-mode validation.
- M4 gRPC parity validation.
- M5 usability/reporting and experiment reproducibility validation.
- M5.4 optional observability UI validation (live metrics pipeline + dashboard evidence).

Acceptance expectations for all milestones are defined in `acceptance_criteria_matrix.md`; scenario-level contracts are defined in `scenario_catalog.md`.

## 4. Repository Layout For Harness Artifacts

```text
docs/evaluation/
  README.md
  acceptance_harness.md
  acceptance_criteria_matrix.md
  metric_definitions.md
  scenario_catalog.md
  baseline_policy.md
  shortcomings_and_remediation.md

evaluations/runs/
  <run_id>/
    manifest.json
    preflight.json
    config_snapshot/
    traces.jsonl
    summary.json
    triage.json

evaluations/reports/
  run_<run_id>.md
  run_<run_id>.json
```

`run_id` format: `YYYYMMDD_HHMMSS_<pipeline_id>`.

## 5. Run Contract (Required Fields)

Each run must persist:

- `run_id`
- `pipeline_id` (human-readable profile)
- `config_hash` (authoritative identity)
- `git_sha`
- `timestamp_utc`
- `seed`
- `limiter_mode` (`in_memory`, `distributed`, `hybrid`)
- `routing_mode` (`single_instance`, `rr`, `sa`)
- `feature_flags` (explicit, no hidden defaults)
- `server_config_path`
- `traffic_config_path`
- `environment` (`redis`, `valkey`, `none`)
- `target_endpoint`

`config_hash` is computed from normalized JSON of all run-defining inputs.

## 6. Preflight (Mandatory)

Fail the run before traffic starts if any check fails:

- Config files exist and parse.
- Exactly one limiter feature is selected for server startup.
- `REDIS_URL` exists when mode is `distributed` or `hybrid`.
- Target endpoint is reachable before timed workload.
- Artifact output path is writable.
- `run_id` is unique unless `overwrite=true` is explicit.
- Required binaries compile for the selected feature set.

Minimum preflight commands:

- `cargo test -p rate_limit -p rest`
- `cargo test -p rate_limit_distributed -p rate_limit_hybrid`
- `make env`

When backend coverage is requested:

- `make redis-up` or `make valkey-up`
- `make test-redis-backend`

## 7. Scenario Catalog (Initial Required Set)

These scenario IDs are the minimum acceptance set for current milestone scope.

- `S_M2_MEM_STEADY_SINGLE_KEY`
- `S_M2_MEM_STEADY_ROUND_ROBIN_KEYS`
- `S_M3_1_DIST_STEADY_SINGLE_KEY`
- `S_M3_2_HYB_STEADY_SINGLE_KEY_FAIL_OPEN`
- `S_M3_2_HYB_STEADY_SINGLE_KEY_FAIL_CLOSED`
- `S_M3_2_HYB_LOCAL_DENY_SHORT_CIRCUIT`
- `S_M3_2_HYB_RETRY_AFTER_MAX_BOTH_DENY`

Extended scenario requirements for M3.3+ through M5 are defined in `docs/evaluation/scenario_catalog.md`.

Initial config anchors:

- `configs/rest_server/in_memory.toml`
- `configs/rest_server/distributed.toml`
- `configs/rest_server/hybrid.toml`
- `configs/traffic_rest/smoke/smoke__single_key__steady__1000x4__5s.toml`

## 8. Per-Trace Schema (Required)

Each request trace record must include:

- `trace_id`
- `scenario_id`
- `request_started_at`
- `request_completed_at`
- `key`
- `http_status`
- `decision` (`allow` or `deny`)
- `retry_after_ms` (nullable)
- `latency_ms`
- `backend_outcome` (`allow`, `deny`, `error`, `none`)
- `failure_policy` (nullable)
- `error_code` (nullable)

## 9. Metrics (Acceptance Set)

Correctness:

- `decision_accuracy`: expected decision match rate per scenario.
- `retry_after_accuracy`: proportion of denies with correct retry-after behavior.
- `key_isolation_error_rate`: cross-key interference violations.

Resilience:

- `backend_error_policy_conformance`: correct fail-open/fail-closed behavior under backend error.
- `short_circuit_conformance`: local deny path does not wait for distributed completion in Option A2 scenarios.

Performance:

- `throughput_rps_observed`
- `deny_ratio`
- `latency_ms_p50`
- `latency_ms_p95`
- `latency_ms_p99`

Reproducibility:

- `repeat_run_decision_delta_pp`: absolute percentage-point delta in allow/deny ratio across repeated run with same `config_hash`.
- `repeat_run_latency_p95_delta_pct`

## 10. Acceptance Criteria (Current Milestone Gate)

M1/M2 must-pass:

- Hierarchical and REST adapter tests pass.
- `decision_accuracy = 1.0` in deterministic correctness scenarios.
- `retry_after_accuracy = 1.0` for deny scenarios with explicit expectations.

M3.1 must-pass:

- Distributed unit tests pass.
- Distributed backend error mapping behavior matches expected policy for the scenario under test.
- Redis-backed integration test passes when backend mode is required for the run.

M3.2 must-pass:

- Hybrid unit tests pass, including fail-open/fail-closed and retry-after max.
- `short_circuit_conformance = 1.0` for local deny short-circuit scenario.
- `backend_error_policy_conformance = 1.0` for fail-open and fail-closed scenarios.

Reproducibility must-pass for smoke acceptance:

- Two repeated runs with identical `config_hash` and seed are required.
- `repeat_run_decision_delta_pp <= 0.5`.
- `repeat_run_latency_p95_delta_pct <= 15`.

Project-level milestone acceptance criteria, including not-yet-implemented milestones, are defined in `docs/evaluation/acceptance_criteria_matrix.md`.

## 11. Triage Labels (Deterministic)

- `ALLOWED_WHEN_SHOULD_DENY`
- `DENIED_WHEN_SHOULD_ALLOW`
- `RETRY_AFTER_MISMATCH`
- `KEY_ISOLATION_BROKEN`
- `FAILURE_POLICY_MISMATCH`
- `SHORT_CIRCUIT_BROKEN`
- `LATENCY_REGRESSION`
- `THROUGHPUT_REGRESSION`
- `NON_REPRODUCIBLE_RUN`

Each failed gate must emit at least one triage label with measured values.

## 12. Manual Gate Workflow (No CI Assumption)

1. Run preflight and capture `preflight.json`.
2. Execute scenario set for the selected gate profile (`smoke` or `full_manual`).
3. Persist `traces.jsonl`, `summary.json`, and `triage.json`.
4. Generate `evaluations/reports/run_<run_id>.md` and `evaluations/reports/run_<run_id>.json`.
5. Mark run `PASS` only if all must-pass rules in Section 10 are satisfied.

## 12.1 Fast TDD Loop (Single AT)

For rapid development on one acceptance behavior:

1. Run one AT only: `make ac-one AT=AT-00X`.
2. Repeat until the AT passes.
3. Run `make ac` before considering the change complete.
4. Run `make ac-full` when touching shared harness/reporting logic.

Single-AT runs still require full artifact generation for that run ID.

## 13. Baseline and Change Control

- Threshold changes require documented rationale in report metadata.
- Any acceptance metric definition change requires updating `docs/evaluation/metric_definitions.md`.
- Baseline updates must include prior baseline reference, new baseline value, linked run artifacts, and justification for shift.

## 14. What Must Exist Before Tuning This Guidance Further

See `docs/evaluation/shortcomings_and_remediation.md` for the maintained project-level gap list and remediation sequence.

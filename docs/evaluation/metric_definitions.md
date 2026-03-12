# Metric Definitions

This document defines acceptance metrics used by the harness.

All metrics must be computed from recorded traces and run manifests.  
If a metric cannot be computed from persisted artifacts, the run is invalid.

## 1. Correctness Metrics

`decision_accuracy`

- Definition: fraction of traces where observed decision equals expected scenario decision.
- Formula: `matching_decisions / total_traces`.
- Pass rule (deterministic scenarios): `== 1.0`.

`retry_after_accuracy`

- Definition: fraction of deny traces where `retry_after_ms` satisfies scenario expectation.
- Formula: `valid_retry_after_denies / total_deny_traces_with_expectation`.
- Pass rule (deterministic scenarios): `== 1.0`.

`http_mapping_accuracy`

- Definition: fraction of traces where decision maps to expected HTTP status (`allow->2xx`, `deny->429`).
- Formula: `valid_http_mappings / total_traces`.
- Pass rule: `== 1.0`.

`key_isolation_error_rate`

- Definition: rate of observed cross-key interference violations in key-isolation scenarios.
- Formula: `isolation_violations / total_keyed_assertions`.
- Pass rule: `== 0.0`.

## 2. Resilience Metrics

`backend_error_policy_conformance`

- Definition: conformance of observed decisions to configured failure policy under backend errors.
- Formula: `policy_conformant_error_traces / total_backend_error_traces`.
- Pass rule: `== 1.0`.

`short_circuit_conformance`

- Definition: conformance of local-deny path to Option A2 short-circuit contract.
- Formula: `short_circuit_conformant_traces / total_short_circuit_assertions`.
- Pass rule: `== 1.0`.

`mode_transition_conformance` (M3.5+)

- Definition: conformance of observed degraded/normal transitions to configured thresholds.
- Formula: `valid_transitions / expected_transitions`.
- Pass rule: `== 1.0`.

## 3. Performance Metrics

`throughput_rps_observed`

- Definition: completed requests per second during measured window.
- Formula: `completed_requests / measured_duration_seconds`.

`deny_ratio`

- Definition: proportion of denied requests.
- Formula: `deny_count / total_count`.

`latency_ms_p50`, `latency_ms_p95`, `latency_ms_p99`

- Definition: latency percentiles over measured traces.
- Formula: percentile calculation over `latency_ms`.

`latency_regression_pct`

- Definition: relative p95 increase versus baseline for comparable scenario/profile.
- Formula: `((current_p95 - baseline_p95) / baseline_p95) * 100`.
- Pass rule: scenario-specific thresholds in `baseline_policy.md`.

## 4. Fairness Metrics

`per_key_allow_variance` (M3.4+)

- Definition: variance in allow counts across keys under same offered load.
- Formula: sample variance over per-key allow counts.

`per_key_deny_variance` (M3.4+)

- Definition: variance in deny counts across keys under same offered load.
- Formula: sample variance over per-key deny counts.

`global_target_drift_pct` (M3.4+)

- Definition: percent deviation from intended global target throughput.
- Formula: `abs(observed - target) / target * 100`.

## 5. Reproducibility Metrics

`repeat_run_decision_delta_pp`

- Definition: absolute percentage-point difference in allow ratio between two runs with same `config_hash`.
- Formula: `abs(allow_ratio_run_a - allow_ratio_run_b) * 100`.
- Smoke pass rule: `<= 0.5`.

`repeat_run_latency_p95_delta_pct`

- Definition: percentage difference in p95 latency across repeated runs with same `config_hash`.
- Formula: `abs(p95_a - p95_b) / min(p95_a, p95_b) * 100`.
- Smoke pass rule: `<= 15`.

`artifact_completeness_rate`

- Definition: fraction of required artifacts present for a run.
- Formula: `present_required_artifacts / total_required_artifacts`.
- Pass rule: `== 1.0`.

## 6. Usability/Governance Metrics (M5)

`one_command_success_rate`

- Definition: fraction of one-command evaluation attempts that produce full artifact sets without manual patching.
- Formula: `successful_runs / attempted_runs`.

`baseline_update_compliance_rate`

- Definition: fraction of baseline update attempts that include all required governance metadata.
- Formula: `compliant_updates / total_update_attempts`.
- Pass rule: `== 1.0`.

## 7. Missing-Data Policy

If required data for a must-pass metric is missing:

- metric value is `null`,
- run status is `FAIL`,
- triage label `MISSING_REQUIRED_EVIDENCE` is attached.

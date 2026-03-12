# Baseline Policy

This policy governs how evaluation baselines are established, compared, and updated.

## 1. Baseline Types

`semantic_baseline`

- Stores expected correctness behavior for deterministic scenarios.
- Changes are rare and require ADR-level rationale when semantics shift.

`performance_baseline`

- Stores reference latency/throughput values by scenario and environment.
- Can evolve with justified infrastructure or implementation changes.

`resilience_baseline`

- Stores expected behavior under failure policies and outage scenarios.
- Must remain aligned with ADR-defined semantics.

## 2. Baseline Keys

Each baseline entry is keyed by:

- `scenario_id`
- `pipeline_id`
- `environment`
- `limiter_mode`
- `routing_mode`
- `config_hash_family` (stable subset identity for comparable runs)

## 3. Baseline Creation Rules

Initial baseline requires:

- at least 3 successful runs on same config-hash family,
- no correctness or resilience contract violations,
- reproducibility metrics within acceptance bounds.

Baseline values:

- correctness/resilience: strict expected values (typically exact).
- performance: median of accepted runs with p95 and p99 recorded.

## 4. Regression Rules

Hard-fail regressions:

- any correctness contract regression,
- any resilience contract regression,
- missing required evidence artifacts.

Performance regressions:

- warn if p95 latency increase is > 10% and <= 20%.
- fail if p95 latency increase is > 20%.
- fail if throughput drop is > 15%.

Reproducibility regressions:

- fail when repeat-run decision delta or latency drift exceeds accepted thresholds.

## 5. Baseline Update Rules

A baseline update must include:

- previous baseline reference,
- candidate baseline values,
- linked run artifacts,
- reason category:
  - `expected_feature_change`
  - `environment_shift`
  - `measurement_fix`
  - `prior_baseline_error`
- approver identity (manual workflow entry).

Baseline update is blocked when:

- correctness or resilience regressions exist,
- artifact completeness is below 1.0,
- reason category is absent.

## 6. Retrospective Baseline Establishment

For milestones completed before this policy:

1. Run required scenarios from `scenario_catalog.md`.
2. Collect at least 3 comparable runs per baseline key.
3. Publish retrospective baseline report.
4. Mark baseline status as `retrospective_established`.

## 7. Valkey/Redis Policy

Redis and Valkey baselines are separate keys and must not be mixed.  
Cross-backend comparisons are informative but do not replace backend-specific baselines.

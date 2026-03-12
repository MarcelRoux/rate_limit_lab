# Evaluation Harness Execution Checklist

This checklist is the deterministic implementation path from no harness to robust acceptance execution.

Use this alongside:

- `docs/evaluation/acceptance_criteria_matrix.md`
- `docs/evaluation/scenario_catalog.md`
- `docs/evaluation/shortcomings_and_remediation.md`

## Global Done Criteria

- `make ac` works for implemented smoke acceptance.
- `make ac-full` works for implemented full matrix acceptance.
- `make ac-one AT=AT-00X` works for single-AT TDD loops.
- Every run emits required artifacts.
- Reports include key metrics and evidence links.

## Phase 1: Rust Harness Skeleton

### 1.1 Create crate

- [ ] Create `crates/eval_harness/Cargo.toml`.
- [ ] Create `crates/eval_harness/src/main.rs`.
- [ ] Add `crates/eval_harness` to workspace members in `Cargo.toml`.

Verification:

- `cargo check -p eval_harness`

### 1.2 CLI command surface

- [ ] Add command: `run --profile <name>`.
- [ ] Add command: `run --at <AT-ID>`.
- [ ] Add command: `compile --input <dir> --output <dir>`.

Verification:

- `cargo run -p eval_harness -- --help`
- `cargo run -p eval_harness -- run --help`

### 1.3 Makefile integration

- [ ] Keep `make ac` mapped to smoke profile.
- [ ] Keep `make ac-full` mapped to full matrix profile.
- [ ] Keep `make ac-one AT=AT-00X` mapped to single-AT mode.

Verification:

- `make ac` (should execute harness path)
- `make ac-one AT=AT-004` (should execute single AT path)

## Phase 2: Run Manifests + Artifacts

### 2.1 Run identity

- [ ] Implement `run_id` creation (`YYYYMMDD_HHMMSS_<pipeline_id>`).
- [ ] Implement `config_hash` generation from normalized run inputs.
- [ ] Write `evaluations/runs/<run_id>/manifest.json`.

Verification:

- run any harness command and inspect manifest fields.

### 2.2 Preflight

- [ ] Implement preflight checks and write `preflight.json`.
- [ ] Fail fast on missing config/env/dependencies.

Verification:

- induce one known preflight failure and confirm clean fail reason.

### 2.3 Required run artifacts

- [ ] Emit `traces.jsonl`.
- [ ] Emit `summary.json`.
- [ ] Emit `triage.json`.
- [ ] Emit `evaluations/reports/run_<run_id>.md`.
- [ ] Emit `evaluations/reports/run_<run_id>.json`.

Verification:

- artifact presence check for a completed run.

## Phase 3: Ready AT Implementation

### 3.1 AT registry

- [ ] Add internal AT registry with AT id -> executor mapping.
- [ ] Add status metadata (`ready/planned/blocked`).
- [ ] Block execution of non-ready ATs with explicit reason.

Verification:

- run `AT-004` succeeds.
- run a planned AT fails with clear status explanation.

### 3.2 Implement current ready AT set

- [ ] Implement AT execution for `AT-004` to `AT-024`.
- [ ] Implement `AT-050` (missing artifact hard fail).

Verification:

- `make ac-one AT=AT-004`
- `make ac-one AT=AT-019`
- synthetic missing-artifact test triggers expected fail label.

### 3.3 Single-AT TDD loop guardrail

- [ ] Ensure `--at` executes only selected AT.
- [ ] Ensure single-AT run still emits full artifact set.
- [ ] Include selected AT id in report title and metadata.

Verification:

- `make ac-one AT=AT-021` and inspect report metadata.

## Phase 4: Metrics + Reporting

### 4.1 Metric scorer

- [ ] Compute correctness metrics from traces.
- [ ] Compute latency/throughput/deny metrics.
- [ ] Compute reproducibility metrics for repeated runs.

Verification:

- `summary.json` contains metrics defined in `metric_definitions.md`.

### 4.2 Evidence linking

- [ ] Add evidence links in markdown report:
  - manifest path
  - preflight path
  - traces path
  - summary path
  - triage path
- [ ] Include AT pass/fail table in report.

Verification:

- open generated markdown report and validate all links.

### 4.3 Report compiler

- [ ] Implement compiled report generation across run ids.
- [ ] Emit:
  - `evaluations/reports/compiled_<stamp>.md`
  - `evaluations/reports/compiled_<stamp>.json`

Verification:

- run compile command and confirm aggregated AT/metric sections.

## Phase 5: Distributed Backend Coverage

### 5.1 Redis-backed execution

- [ ] Integrate backend setup flow in harness execution path.
- [ ] Implement AT coverage for `AT-016`, `AT-017`.

Verification:

- backend-enabled profile run emits expected distributed evidence.

### 5.2 Reproducibility gate

- [ ] Add repeat-run flow (`--repeat 2` minimum).
- [ ] Enforce reproducibility thresholds as pass/fail criteria.

Verification:

- repeated smoke run includes reproducibility section and gate result.

## Phase 6: Failure + Topology Expansion

### 6.1 Failure injection

- [ ] Implement deterministic `outage`, `latency`, and `flapping` controls.
- [ ] Wire ATs `AT-025` to `AT-029`.

Verification:

- each failure scenario emits timeline evidence and policy conformance.

### 6.2 RR/SA profiles

- [ ] Add reusable RR and SA scenario profiles.
- [ ] Wire ATs `AT-030` to `AT-034`.

Verification:

- reports include fairness/drift comparisons between RR and SA.

## Phase 7: Productized Commands

- [ ] `make ac` executes implemented smoke acceptance end-to-end.
- [ ] `make ac-full` executes implemented full acceptance matrix end-to-end.
- [ ] `make ac-one AT=AT-00X` executes only one AT end-to-end.

Verification:

- run all three commands successfully on implemented scope.

## Final Acceptance Sign-off Checklist

- [ ] All ready ATs pass.
- [ ] Artifact completeness is `1.0` for ready scenarios.
- [ ] Reports include key metrics and evidence links.
- [ ] Compiled report generation works for multi-run batches.
- [ ] Makefile command contract remains stable.

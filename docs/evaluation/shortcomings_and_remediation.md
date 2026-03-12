# Shortcomings And Implementation-First Remediation Plan

This document is implementation-first by design.  
Goal: move from no harness to a robust harness quickly, with stable one-command execution and reporting.

## 1. Clarifications

- `RR` means round-robin routing: requests are distributed across instances without stickiness.
- `SA` means session/key affinity routing: the same key tends to hit the same instance.

## 2. Architecture Constraint (Mandatory)

The acceptance harness runtime must be implemented as a Rust crate.

- Primary implementation target: `crates/eval_harness`.
- Shell scripts are optional wrappers only.
- Core logic must remain in Rust:
  - AT execution engine
  - manifest/config hash generation
  - trace writing
  - metric scoring
  - report generation and report compilation

Stable command contract:

- `make ac` runs implemented smoke acceptance and emits required artifacts.
- `make ac-full` runs implemented full acceptance matrix and emits required artifacts.
- `make ac-one AT=AT-00X` runs exactly one acceptance test id for rapid TDD loops.
- These targets remain stable even if internal Rust APIs evolve.

## 3. Current Harness Gaps (Execution Impact)

| Gap ID | Current Gap | Impact | Severity |
| --- | --- | --- | --- |
| G-001 | No Rust AT runner crate executing AT IDs end-to-end | acceptance matrix cannot be automated | critical |
| G-002 | No standard manifest/config-hash generator | deterministic run comparison is not reliable | critical |
| G-003 | No canonical trace writer (`traces.jsonl`) | core metrics cannot be computed reliably | critical |
| G-004 | No report compiler for run-level and compiled reports | output is not consumable at project-report level | high |
| G-005 | Failure injection controls not standardized | M3.3+ contracts are not executable | high |
| G-006 | RR/SA topology profiles not formalized | M3.4 fairness/drift criteria are untestable | high |
| G-007 | `make ac`/`make ac-full` not fully implemented through Rust harness | M5 usability criteria remain blocked | high |

## 4. Fastest Path To Working Harness (Implementation Order)

### Phase 1: Thin Vertical Slice (2-3 days)

Objective: one Rust command path runs ready ATs and emits full run/report artifacts.

Concrete tasks:

1. Create `crates/eval_harness` and add workspace membership.
2. Add CLI command:
   - `cargo run -p eval_harness -- run --profile smoke_ready`
3. Implement Rust modules for:
   - manifest + config hash
   - trace writer
   - summary + triage output
   - per-run markdown/json report writer
4. Implement ready AT mapping for:
   - `AT-004` to `AT-024`
   - `AT-050`
5. Add single-AT selector (`--at AT-00X`) in Rust CLI.
6. Wire Makefile commands to Rust CLI:
   - `make ac`
   - `make ac-one AT=AT-00X`

Exit criteria:

- `make ac` succeeds.
- `make ac-one AT=AT-004` succeeds.
- Required artifacts are present for the run.

### Phase 2: Real Backend + Reproducibility (1-2 days)

Objective: include distributed backend evidence and repeatability checks.

Concrete tasks:

1. Add Rust-runner backend setup path (using existing redis/valkey make targets as helpers).
2. Wire `AT-016` and `AT-017`.
3. Add `--repeat` path for `AT-012` reproducibility.
4. Hard fail if required artifacts/trace fields are missing.
5. Wire `make ac-full` to implemented matrix profile.

Exit criteria:

- `make ac` and `make ac-full` both run implemented scope.
- Reproducibility section appears in run report outputs.

### Phase 3: Reporting That Scales (1-2 days)

Objective: compile multiple run reports into project-level report bundles.

Concrete tasks:

1. Add Rust compile command:
   - `cargo run -p eval_harness -- compile --input runs --output reports`
2. Generate:
   - `evaluations/reports/compiled_<stamp>.md`
   - `evaluations/reports/compiled_<stamp>.json`
3. Include:
   - pass/fail by AT ID
   - metric regressions
   - triage label counts
   - links to run evidence

Exit criteria:

- `AT-048` executable for available runs.
- Compiled report links all included run IDs.

### Phase 4: Failure + Topology Expansion (M3.3/M3.4)

Objective: move M3.3/M3.4 ATs from planned to ready.

Concrete tasks:

1. Implement deterministic fault injection in Rust (`outage`, `latency`, `flapping`).
2. Add RR/SA topology profiles.
3. Emit fairness and drift metrics in summaries.
4. Wire `AT-025` through `AT-034`.

Exit criteria:

- M3.3 and M3.4 scenarios become executable and report-backed.

### Phase 5: One-Command Productization (M5)

Objective: stable one-command smoke and full acceptance runs with governance.

Concrete tasks:

1. Finalize CLI entrypoints:
   - `eval_harness run --profile smoke_ready`
   - `eval_harness run --profile full_matrix`
2. Keep Makefile mapping stable:
   - `make ac`
   - `make ac-full`
   - `make ac-one AT=AT-00X`
3. Enforce baseline-governance check (`AT-047`).
4. Wire one-command criteria (`AT-045`, `AT-046`, `AT-049`).

Exit criteria:

- Both make targets produce complete artifacts and summary reports.

## 5. Concrete Backlog (Ready To Implement)

| Task ID | File/Path Target | Deliverable | Linked ATs |
| --- | --- | --- | --- |
| H-001 | `crates/eval_harness/Cargo.toml` + `src/main.rs` | Rust harness CLI scaffold | AT-045, AT-046 |
| H-002 | `crates/eval_harness/src/run_manifest.rs` | `manifest.json` + `config_hash` writer | AT-012, AT-049 |
| H-003 | `crates/eval_harness/src/trace_writer.rs` | canonical `traces.jsonl` emitter | AT-017 |
| H-004 | `crates/eval_harness/src/report_writer.rs` | `run_<run_id>.md/.json` writer with evidence links | all |
| H-005 | `crates/eval_harness/src/report_compiler.rs` | `compiled_<stamp>.md/.json` writer | AT-034, AT-048 |
| H-006 | `crates/eval_harness/src/profiles.rs` | `smoke_ready` and `full_matrix` profile registry | scenario catalog |
| H-007 | `crates/eval_harness/src/fault_injection.rs` | deterministic failure-injection controls | AT-025..AT-029 |
| H-008 | `Makefile` | stable `make ac` and `make ac-full` targets | AT-045, AT-046 |
| H-009 | `crates/eval_harness/src/main.rs` + `Makefile` | single-AT execution path (`--at`, `make ac-one`) | rapid TDD guardrail |

## 6. Governance Guardrails

1. No scenario is marked `ready` without executable command path and report evidence.
2. No AT is marked pass if any required artifact is missing.
3. No baseline update is accepted without rationale and linked run IDs.
4. No milestone acceptance claim from tests alone when scenario/report evidence is required.

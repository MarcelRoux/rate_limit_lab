# Acceptance Criteria Matrix (Atomic AT IDs)

This file defines atomic acceptance tests (`AT-###`) that can be codified directly into an automated evaluation harness.

Status values:

- `ready`: executable with current repository capabilities.
- `planned`: contract fixed; harness execution pending feature implementation.
- `blocked`: blocked by missing harness plumbing.

## Report Contract (Applies To Every AT)

Every AT run must emit:

- `evaluations/runs/<run_id>/manifest.json`
- `evaluations/runs/<run_id>/preflight.json`
- `evaluations/runs/<run_id>/summary.json`
- `evaluations/runs/<run_id>/triage.json`
- `evaluations/runs/<run_id>/traces.jsonl`
- `evaluations/reports/run_<run_id>.md`
- `evaluations/reports/run_<run_id>.json`

Any missing artifact is an automatic fail.

## Atomic Acceptance Tests

| AT ID | Milestone | Atomic Assessment | Automatable Input | Pass Condition | Status |
| --- | --- | --- | --- | --- | --- |
| AT-001 | M0 | Hook path is set to repo hooks directory | `git config --get core.hooksPath` | value equals `githooks` | ready |
| AT-002 | M0 | Pre-commit hook is executable | file mode check | executable bit set | ready |
| AT-003 | M0 | Required quality commands are documented | docs parse check | `cargo fmt`, `cargo clippy`, `typos` present in canonical docs | ready |
| AT-004 | M1 | Hierarchical allow when both tiers allow | `cargo test -p rate_limit hierarchical_allows_if_both_pass` | test pass | ready |
| AT-005 | M1 | Hierarchical deny on global exhaustion | `cargo test -p rate_limit hierarchical_denies_if_global_exceeded` | test pass | ready |
| AT-006 | M1 | Hierarchical deny on per-key exhaustion | `cargo test -p rate_limit hierarchical_denies_if_key_exceeded` | test pass | ready |
| AT-007 | M1 | Hierarchical deny uses max retry-after | deterministic assertion from test trace | observed retry-after equals max component retry-after | ready |
| AT-008 | M2 | REST middleware allow mapping | `cargo test -p rest rest_middleware_allows_request` | HTTP allow path is correct | ready |
| AT-009 | M2 | REST middleware deny mapping | `cargo test -p rest rest_middleware_denies_request` | HTTP 429 + retry-after present | ready |
| AT-010 | M2 | Traffic runner supports single-key mode | smoke run with single-key profile | non-empty trace set, no harness errors | ready |
| AT-011 | M2 | Traffic runner supports round-robin key mode | smoke run with round-robin profile | keys rotate and are reflected in traces | ready |
| AT-012 | M2 | Repeatability for same config hash | two identical smoke runs | decision and p95 drift stay within thresholds | ready |
| AT-013 | M3.1 | Distributed allow mapping | `cargo test -p rate_limit_distributed allows_when_backend_allows` | test pass | ready |
| AT-014 | M3.1 | Distributed deny mapping | `cargo test -p rate_limit_distributed denies_when_backend_denies` | test pass | ready |
| AT-015 | M3.1 | Distributed backend error mapping | `cargo test -p rate_limit_distributed denies_with_backend_error_retry_after` | test pass with expected retry-after behavior | ready |
| AT-016 | M3.1 | Redis integration fixed-window behavior | redis-backed integration run | allow then deny within window | ready |
| AT-017 | M3.1 | Distributed traces include backend outcome field | trace schema check | `backend_outcome` present and non-null for distributed traces | ready |
| AT-018 | M3.2 | Hybrid strict-AND allow path | `cargo test -p rate_limit_hybrid local_and_distributed_allow` | test pass | ready |
| AT-019 | M3.2 | Hybrid local-deny short-circuit | `cargo test -p rate_limit_hybrid short_circuits_when_local_denies_before_distributed_completes` | deny returns without awaiting distributed completion | ready |
| AT-020 | M3.2 | Hybrid distributed deny when local allows | `cargo test -p rate_limit_hybrid distributed_denies_when_local_allows` | test pass | ready |
| AT-021 | M3.2 | Hybrid retry-after max composition | `cargo test -p rate_limit_hybrid max_retry_after_when_both_deny` | composed retry-after equals max of denying branches | ready |
| AT-022 | M3.2 | Hybrid fail-open on backend error | `cargo test -p rate_limit_hybrid distributed_backend_error_obeys_fail_open` | test pass | ready |
| AT-023 | M3.2 | Hybrid fail-closed on backend error | `cargo test -p rate_limit_hybrid distributed_backend_error_obeys_fail_closed` | test pass | ready |
| AT-024 | M3.2 | Hybrid ordering head-start behavior | `cargo test -p rate_limit_hybrid option_a2_head_start_reduces_total_wait` | test pass | ready |
| AT-025 | M3.3 | Inject short backend outage deterministically | harness outage control scenario | observed behavior matches configured failure policy | planned |
| AT-026 | M3.3 | Inject long backend outage deterministically | harness outage control scenario | no silent behavior transition; contract labels emitted | planned |
| AT-027 | M3.3 | Inject backend latency spike | harness latency injection scenario | policy and latency triage labels correct | planned |
| AT-028 | M3.3 | Inject backend flapping | harness flapping scenario | mode changes and recovery are observable and deterministic | planned |
| AT-029 | M3.3 | Failure scenarios produce outage timeline report | report generation check | report includes event timeline and policy decisions | planned |
| AT-030 | M3.4 | RR routing fairness capture | RR multi-instance scenario | fairness metrics emitted and within configured bounds | planned |
| AT-031 | M3.4 | SA routing fairness capture | SA multi-instance scenario | fairness metrics emitted and compared to RR | planned |
| AT-032 | M3.4 | Global drift measurement under RR | RR drift scenario | `global_target_drift_pct` computed and reported | planned |
| AT-033 | M3.4 | Global drift measurement under SA | SA drift scenario | drift computed and reported | planned |
| AT-034 | M3.4 | Distributed metrics report bundle generation | full matrix run | consolidated comparative report is produced | planned |
| AT-035 | M3.5 | Lease refresh-loop correctness | lease scenario | no over-issuance; lease events consistent | planned |
| AT-036 | M3.5 | Degraded fallback transition correctness | outage crossing threshold scenario | transition sequence matches configured state machine | planned |
| AT-037 | M3.5 | Per-instance fallback cap enforcement | degraded scenario | cap never exceeded | planned |
| AT-038 | M3.5 | Max-outage to fail-closed transition | long outage scenario | transition occurs at configured threshold | planned |
| AT-039 | M3.5 | Recovery re-sync after backend restoration | restoration scenario | degraded mode exits and state re-sync is observable | planned |
| AT-040 | M4 | gRPC unary parity with REST semantics | unary parity scenario | decision and retry-after semantics match REST contract | planned |
| AT-041 | M4 | gRPC server streaming enforcement | streaming scenario | stream-level throttling conforms to contract | planned |
| AT-042 | M4 | gRPC client streaming enforcement | streaming scenario | client stream throttling conforms to contract | planned |
| AT-043 | M4 | gRPC bidi streaming enforcement | streaming scenario | bidi stream throttling conforms to contract | planned |
| AT-044 | M4 | gRPC reporting parity | report check | gRPC runs emit same artifact schema as REST runs | planned |
| AT-045 | M5 | One-command smoke evaluation | `make ac` | complete artifact set produced in single invocation | blocked |
| AT-046 | M5 | One-command full matrix evaluation | `make ac-full` | full matrix artifacts produced in single invocation | blocked |
| AT-047 | M5 | Baseline update governance check | baseline update attempt | update blocked if required metadata missing | planned |
| AT-048 | M5 | Multi-run comparative report generation | report compiler command | combined milestone report emitted | planned |
| AT-049 | M5 | Reproducibility gate enforcement | repeated run command | automatic pass/fail based on reproducibility thresholds | planned |
| AT-050 | M5 | Missing-artifact hard fail | synthetic incomplete run | run fails with `MISSING_REQUIRED_EVIDENCE` triage label | ready |
| AT-051 | M5 | Single-AT execution command | `make ac-one AT=AT-00X` | only requested AT executes and emits required artifacts | blocked |

## Immediate Implementation Priority (Fastest Path)

To move quickly from zero harness to robust harness, implement ATs in this order:

1. `AT-045`, `AT-050` (harness skeleton and hard-fail contract).
2. `AT-004` through `AT-024` (current implemented features).
3. `AT-016`, `AT-017` (real backend evidence and trace completeness).
4. `AT-025` through `AT-034` (failure/distributed matrix).
5. `AT-035` through `AT-049` as milestone features land.

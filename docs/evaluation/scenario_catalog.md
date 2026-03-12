# Scenario Catalog (Tabular)

This catalog maps executable/planned scenarios to atomic acceptance tests (`AT-###`).

Status values:

- `ready`: executable now.
- `planned`: contract fixed; execution pending milestone implementation.
- `blocked`: requires harness capability not yet implemented.

## Global Reporting Requirement

Every scenario run must produce:

- `evaluations/runs/<run_id>/manifest.json`
- `evaluations/runs/<run_id>/preflight.json`
- `evaluations/runs/<run_id>/traces.jsonl`
- `evaluations/runs/<run_id>/summary.json`
- `evaluations/runs/<run_id>/triage.json`
- `evaluations/reports/run_<run_id>.md`
- `evaluations/reports/run_<run_id>.json`

Additionally, matrix/aggregate scenarios must produce:

- `evaluations/reports/compiled_<stamp>.md`
- `evaluations/reports/compiled_<stamp>.json`

## M0-M2 Scenarios

| Scenario ID | Milestone | Linked AT IDs | Goal | Primary Execution Input | Expected Outcome | Required Report Add-on | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| S_M0_HOOKS_ACTIVE | M0 | AT-001, AT-002, AT-003 | Verify developer quality gate active | hook/path checks + docs checks | hooks active and required tooling documented | preflight section includes hook-path evidence | ready |
| S_M1_HIERARCHICAL_CORE | M1 | AT-004, AT-005, AT-006, AT-007 | Validate strict AND and retry-after max semantics | targeted `rate_limit` tests | all hierarchical semantic tests pass | summary includes semantic assertion results | ready |
| S_M2_REST_MAPPING | M2 | AT-008, AT-009 | Validate middleware decision->HTTP mapping | targeted `rest` middleware tests | allow path and deny path both correct | report includes HTTP mapping accuracy metric | ready |
| S_M2_TRAFFIC_SINGLE_KEY | M2 | AT-010, AT-012 | Validate deterministic smoke run with single key | smoke traffic profile | non-empty traces and repeatability bounds pass | report includes run-to-run drift section | ready |
| S_M2_TRAFFIC_ROUND_ROBIN_KEYS | M2 | AT-011 | Validate key rotation path in generator | round-robin traffic profile | key rotation visible in traces | report includes per-key traffic distribution table | ready |

## M3.1-M3.2 Scenarios

| Scenario ID | Milestone | Linked AT IDs | Goal | Primary Execution Input | Expected Outcome | Required Report Add-on | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| S_M3_1_DIST_MAPPING | M3.1 | AT-013, AT-014, AT-015 | Validate distributed decision mappings | targeted distributed tests | allow/deny/error semantics pass | report includes backend error policy section | ready |
| S_M3_1_DIST_REDIS_INTEGRATION | M3.1 | AT-016, AT-017 | Validate real Redis fixed-window behavior and trace shape | redis-backed integration run | allow then deny; backend fields present in traces | report includes backend fingerprint + environment | ready |
| S_M3_2_HYB_STRICT_AND | M3.2 | AT-018, AT-020, AT-021 | Validate hybrid strict-AND composition and retry-after max | targeted hybrid tests | semantics pass | summary includes deny-source composition stats | ready |
| S_M3_2_HYB_ORDERING | M3.2 | AT-019, AT-024 | Validate Option A2 ordering and short-circuit | targeted hybrid timing tests | short-circuit and head-start assertions pass | report includes ordering conformance section | ready |
| S_M3_2_HYB_FAILURE_POLICY | M3.2 | AT-022, AT-023 | Validate fail-open/fail-closed behavior | targeted hybrid backend-error tests | policy conformance is exact | report includes failure-policy outcome table | ready |

## M3.3-M3.5 Scenarios

| Scenario ID | Milestone | Linked AT IDs | Goal | Primary Execution Input | Expected Outcome | Required Report Add-on | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| S_M3_3_OUTAGE_SHORT | M3.3 | AT-025 | Validate short outage policy behavior | deterministic outage injection | behavior matches configured policy | outage timeline chart/table | planned |
| S_M3_3_OUTAGE_LONG | M3.3 | AT-026 | Validate long outage behavior contract | sustained outage injection | no silent transitions | explicit transition log section | planned |
| S_M3_3_LATENCY_SPIKE | M3.3 | AT-027 | Validate behavior during backend latency spikes | latency injection | conformance + triage correctness | latency spike impact section | planned |
| S_M3_3_BACKEND_FLAP | M3.3 | AT-028, AT-029 | Validate flapping resilience and reporting | flapping injection | deterministic transitions and recovery evidence | policy-decisions timeline in report | planned |
| S_M3_4_RR_FAIRNESS | M3.4 | AT-030, AT-032 | Validate fairness/drift under round-robin routing | RR multi-instance profile | fairness and drift metrics computed | per-key fairness/drift tables | planned |
| S_M3_4_SA_FAIRNESS | M3.4 | AT-031, AT-033 | Validate fairness/drift under sticky-affinity routing | SA multi-instance profile | fairness and drift metrics computed | RR vs SA comparison section | planned |
| S_M3_4_DISTRIBUTED_COMPILED_REPORT | M3.4 | AT-034 | Compile distributed matrix into aggregate report | matrix run compiler | compiled report artifacts produced | `compiled_<stamp>.md/.json` | planned |
| S_M3_5_LEASE_REFRESH | M3.5 | AT-035 | Validate lease refresh correctness | lease/refresh scenario | no over-issuance and coherent lease events | lease lifecycle section | planned |
| S_M3_5_DEGRADED_TRANSITIONS | M3.5 | AT-036, AT-038, AT-039 | Validate degraded mode transitions and recovery | long-outage + restore scenario | state transitions match config | transition timeline + recovery section | planned |
| S_M3_5_FALLBACK_CAPS | M3.5 | AT-037 | Validate fallback cap enforcement | degraded fallback stress profile | cap never exceeded | cap violation counter section | planned |

## M4-M5 Scenarios

| Scenario ID | Milestone | Linked AT IDs | Goal | Primary Execution Input | Expected Outcome | Required Report Add-on | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| S_M4_GRPC_UNARY_PARITY | M4 | AT-040 | Validate unary gRPC parity with REST semantics | unary parity suite | semantic parity achieved | REST vs gRPC parity diff section | planned |
| S_M4_GRPC_STREAMING_PARITY | M4 | AT-041, AT-042, AT-043 | Validate streaming enforcement parity | streaming suite | stream semantics conform to contract | stream lifecycle evidence section | planned |
| S_M4_GRPC_REPORT_PARITY | M4 | AT-044 | Validate artifact/report parity for gRPC runs | report schema validator | report schema identical to REST runs | schema conformance table | planned |
| S_M5_ONE_COMMAND_SMOKE | M5 | AT-045, AT-049, AT-050 | One-command smoke run with full hard-fail behavior | `make ac` | full artifact set + gate decisions produced | smoke run result section | blocked |
| S_M5_ONE_COMMAND_FULL_MATRIX | M5 | AT-046, AT-048 | One-command matrix run + compiled report | `make ac-full` | full matrix + compiled outputs produced | matrix summary + compiled outputs | blocked |
| S_M5_SINGLE_AT_TDD_LOOP | M5 | AT-051 | Run one acceptance test repeatedly for fast fix loops | `make ac-one AT=AT-00X` | only target AT runs and still emits required artifacts | single-AT result section with AT id | blocked |
| S_M5_BASELINE_GOVERNANCE | M5 | AT-047 | Validate baseline change governance | baseline update workflow test | update blocked if governance evidence missing | baseline governance audit section | planned |

## Terminology

- `RR`: round-robin routing (non-sticky distribution across instances).
- `SA`: session-affinity or key-affinity routing (same key tends to same instance).

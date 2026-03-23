# Product Facets Overview

Last updated: 2026-03-22

This table summarizes what the repository is building as product-facing capabilities, why each matters, current state, and estimated effort remaining.

| Facet | Milestone | Brief Description | Portfolio Value | Current State | Estimated Remaining Effort |
| --- | --- | --- | --- | --- | --- |
| In-memory limiter core | M1 | Local hierarchical limiting with strict AND semantics and retry-after max behavior. | Shows strong fundamentals in algorithmic correctness and deterministic enforcement. | Implemented and passing (`AT-004` to `AT-007`). | Low (1-3 days for polish and additional edge-case tests). |
| REST enforcement + traffic generation | M2 | REST middleware maps allow/deny to HTTP behavior; traffic generator supports keyed workloads and pacing. | Demonstrates practical integration and realistic load simulation. | Implemented and passing (`AT-008` to `AT-012`). | Low-Medium (3-5 days for broader profile coverage). |
| Distributed limiter (Redis-backed) | M3.1 | Backend abstraction with Redis fixed-window behavior and distributed decision mapping. | Demonstrates distributed systems integration and external state handling. | Implemented and passing (`AT-013` to `AT-017`). | Medium (1-2 weeks for deeper resilience and backend parity validation). |
| Hybrid limiter (local + distributed) | M3.2 | Composes local and distributed checks with Option A2 ordering and failure-policy controls. | Shows nuanced latency/correctness tradeoff design and policy-driven behavior. | Implemented and passing (`AT-018` to `AT-024`). | Medium (1-2 weeks for M3.5 lease/degraded extensions). |
| Failure-injection and topology evaluation | M3.3-M3.4 | Deterministic outage/latency/flapping plus RR/SA comparison and timeline evidence. | Demonstrates disciplined resilience and fairness/drift evaluation. | Implemented in harness and passing (`AT-025` to `AT-034`). | Medium (1-2 weeks to deepen realism and scenario breadth). |
| Configurability | M2-M5 | Profile-driven and feature-driven run control with stable command contracts and config hashing. | Shows product-quality operational control and repeatability. | Strong baseline in place (`make ac`, `make ac-full`, `make ac-one`). | Medium (1 week to complete full governance and config-surface hardening). |
| Observability and reporting artifacts | M3.4-M5 | Canonical traces, summaries, triage, run reports, and compiled reports with evidence links. | Shows measurable engineering discipline and auditability. | Implemented and operational (`evaluations/runs/*`, `evaluations/reports/*`). | Low-Medium (3-7 days for richer analysis views). |
| Observability UI (Grafana + live pipeline, optional) | Proposed M5.4 (optional) | Optional live metrics pipeline feeding Grafana dashboards for real-time experiment/runtime visibility. Feature-gated in binaries. | Elevates project to production-grade observability storytelling with live diagnostics. | Not implemented yet. | Medium-Large (2-4 weeks MVP; 4-6 weeks with full hardening). |
| Protocol breadth (gRPC parity) | M4 | Extend limiting and evaluation parity from REST to gRPC unary and streaming modes. | Demonstrates protocol-agnostic architecture and transport extensibility. | Not started. | Large (3-6+ weeks depending on streaming scope). |
| Backend portability (Valkey parity) | M3.x follow-on | Validate and document equivalent distributed behavior on Valkey. | Demonstrates portability and vendor-flexible backend strategy. | Partially prepared (compose profile exists), full validation pending. | Medium (1-2 weeks). |
| Usability and repeatability | M5 | One-command runs, single-AT TDD loops, reproducibility gates, deterministic artifact contracts. | Shows developer experience maturity and reliable evaluation workflows. | Strong on implemented scope; governance tail remains. | Medium (1 week). |
| Engineering practice and governance | M0-M5 | ADR-led design, milestone-driven execution, acceptance matrix, artifact-backed pass/fail discipline. | Signals senior-level delivery rigor and maintainability. | Strong process baseline; some governance policies still maturing. | Medium (1 week for baseline-governance completion and CI policy capture). |

## Observability UI MVP Recommendation

For an MVP, start with **Prometheus-compatible metrics + Grafana dashboards**, then add OpenTelemetry export as a second step.

Reasoning:

- Prometheus + Grafana is the fastest path to useful dashboards for counters, histograms, and rate/latency panels.
- It maps directly to the metrics already modeled in the evaluation harness (`throughput`, `latency p50/p95/p99`, `deny ratio`, reproducibility deltas).
- OpenTelemetry can be layered later for unified metrics/traces/logs without blocking initial dashboard value.

Suggested optional feature flag surface:

- `--features observability-ui` (or equivalent cargo feature set)
- Runtime toggle for metrics export endpoint (enabled/disabled)
- Config-driven dashboard mode (`off`, `local`, `full`)

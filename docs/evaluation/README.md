# Evaluation Docs Index

This directory defines the acceptance-evaluation system for this repository at a project level, not only current implementation state.

Use these files in this order:

1. `acceptance_harness.md`: run contract, artifact model, manual gate workflow.
2. `acceptance_criteria_matrix.md`: milestone-by-milestone acceptance contract (implemented and planned).
3. `scenario_catalog.md`: scenario IDs, expected outcomes, required evidence.
4. `metric_definitions.md`: metric formulas, units, and pass/fail semantics.
5. `baseline_policy.md`: baseline update and regression governance.
6. `shortcomings_and_remediation.md`: current harness gaps and remediation sequence.
7. `execution_checklist.md`: granular implementation checklist for deterministic harness delivery.

## Operating Principle

Acceptance criteria are defined before implementation whenever possible.  
If a milestone is not implemented yet, its acceptance contract still exists here and is marked as `planned_contract`.

## Scope Note

Manual execution is supported now. CI wiring can be layered later without changing acceptance semantics.

## Stable Developer Commands

- `make ac` for implemented smoke acceptance.
- `make ac-full` for implemented full acceptance matrix.
- `make ac-one AT=AT-00X` for single-AT TDD loop.

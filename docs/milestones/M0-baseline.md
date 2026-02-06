# Milestone M0 — Developer Environment Foundation

## Goal
Establish the quality gate described in `DESIGN.md` so every contributor runs identical hooks and tooling (`cargo fmt`, `cargo clippy`, `typos`) before touching higher milestones. (docs/DESIGN.md: Section 6 “M0 - Development Environment”)

## Implemented Components
- Git hooks directory (`githooks/pre-commit`, `githooks/pre-push`, `githooks/commit-msg`) that capture formatting/lint checks before commits/pushes. (githooks/pre-commit; githooks/pre-push; githooks/commit-msg)
- `scripts/install-hooks.sh` to wire `core.hooksPath` to `githooks/` so the hooks run automatically. (scripts/install-hooks.sh)
- Design-specified tooling checklist for `cargo fmt`, `cargo clippy`, and `typos`, referenced by the milestone definition. (docs/DESIGN.md: Section 6 “M0 - Development Environment”)

## Decision Semantics
- Git hooks are the gatekeepers: they execute formatting/linting checks before `commit`/`push` so higher milestones inherit a consistent baseline. (githooks/pre-commit; githooks/pre-push; githooks/commit-msg; scripts/install-hooks.sh)
- Tooling commands named in the design (`cargo fmt`, `cargo clippy`, `typos`) are the agreed semantics for this milestone even if their enforcement is delegated to hooks or CI. (docs/DESIGN.md: Section 6 “M0 - Development Environment”)

## Failure Behavior
- If hooks are not installed, the repo cannot enforce the milestone gates locally; users must run `scripts/install-hooks.sh` to restore them. (scripts/install-hooks.sh)
- Missing tooling (fmt/clippy/typos) renders the milestone incomplete until the tooling commands exist in CI/hook scripts per the design. (docs/DESIGN.md: Section 6 “M0 - Development Environment”)

## Out of Scope
- Business logic, limiter implementations, protocol adapters, and distributed components; those belong to later milestones. (crates/rate_limit/src/...; crates/adapters/rest/src/...; crates/rate_limit_distributed/src/...)

## Acceptance Checklist
- [ ] Git hooks are installed via `scripts/install-hooks.sh` and reference the `githooks` directory. (scripts/install-hooks.sh; githooks/pre-commit; githooks/pre-push; githooks/commit-msg)
- [ ] The milestone definition references `cargo fmt`, `cargo clippy`, and `typos` as the required tooling commands. (docs/DESIGN.md: Section 6 “M0 - Development Environment”)
- [ ] Contributors can describe how M0’s hooks/tooling foundation supports later milestones. (docs/DESIGN.md: Section 6 “M0 - Development Environment”)

## Drift Notes
- The current implementation emphasizes the baseline limiter core, which belongs to M2/M3 work, so documentation temporarily notes that the delivered code sits ahead of the M0 env foundation. (crates/rate_limit/src/direct_limiter.rs; crates/rate_limit/src/models.rs; docs/DESIGN.md: Section 6 “M0 - Development Environment”)

## Related ADRs
- None yet.

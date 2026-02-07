# docs/backlog/README.md

## Backlog system (file-per-item)

This repository uses a file-per-backlog-item approach to track future work, experiments, and improvements without requiring an external tracker.

### Goals
- Keep future scope discoverable and reviewable in-repo.
- Provide lightweight “state” transitions similar to a backlog tool.
- Preserve a clean history of why work was done (or not done) via links to ADRs and PRs.
- Avoid a single growing FUTURE-SCOPE.md becoming unmaintainable.

---

## Directory layout

- `docs/backlog/`
  - `README.md` (this file)
  - `_template.md` (copy for new items)
  - `0001-...md`, `0002-...md`, ...
- `docs/backlog/done/`
  - Completed items moved here when done.

---

## Workflow

### 1) Create a new backlog item
1. Copy `docs/backlog/_template.md` into a new file:  
   `docs/backlog/NNNN-short-slug.md`
2. Pick the next available `NNNN` number (zero-padded, monotonic).
3. Fill in metadata and content.

### 2) Selecting what to work on next
Use these fields for prioritization:
- `Priority`: P0 / P1 / P2
- `Milestone`: a project milestone (e.g., M3.2) or `TBD`
- `Impact` + `Risk` sections (optional but recommended)

### 3) When work starts
- Set `Status: In Progress`
- Add a link to the tracking PR if you use PRs for incremental work.

### 4) When work completes
- Set `Status: Done`
- Add:
  - `Implemented in: <PR links>`
  - `Shipped in: <tag or commit hash>` (optional)
  - `ADR: <link>` (optional)
- Move the file to: `docs/backlog/done/NNNN-short-slug.md`

### 5) If the item is dropped or superseded
- Set `Status: Dropped` or `Status: Superseded`
- Explain why in `Outcome`.
- Move to `docs/backlog/done/` as well (the `done/` folder is the archive of closed items, not only “successes”).

---

## Status definitions

- `Proposed`: Idea captured, not yet scheduled.
- `Planned`: Accepted into a milestone or near-term scope.
- `In Progress`: Active work underway.
- `Done`: Completed and merged.
- `Dropped`: Intentionally not pursued.
- `Superseded`: Replaced by another item (link to replacement).

---

## Priority definitions

- `P0`: Must do / blocks milestones / correctness or safety critical.
- `P1`: High value, likely next.
- `P2`: Nice-to-have / exploratory / optional.

---

## Milestone conventions

Backlog items SHOULD reference the milestone they most directly affect.

Use one of:
- `Milestone: M3.2` (if it fits cleanly in the current plan)
- `Milestone: M3.x` (if it’s in the M3 family but exact slot is TBD)
- `Milestone: Post-M5` (if it’s clearly beyond the current delivery plan)
- `Milestone: TBD` (if unknown)

Important: A backlog item can be associated with a milestone without changing the milestone plan. If an item would disrupt sequencing or is substantially new scope, prefer `Post-M5` or `M3.x` rather than renumbering milestones.

---

## Naming convention

- File name: `NNNN-short-slug.md`
- `NNNN` is a monotonic sequence: 0001, 0002, ...
- Slug should be short and stable.

Examples:
- `0001-valkey-backend.md`
- `0002-hybrid-outage-semantics.md`

---

## Required metadata (top of each item)

Each backlog item MUST begin with:

- Title
- Status
- Priority
- Milestone
- Owner (optional)
- Created (YYYY-MM-DD)
- Links (optional)

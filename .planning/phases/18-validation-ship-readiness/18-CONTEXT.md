# Phase 18: Validation + Ship Readiness — Context

**Gathered:** 2026-04-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 18 validates and closes out the v1.2 milestone. All code was shipped in phases 14–17. This phase:
1. Produces a **UAT.md** — a manual walkthrough checklist covering all 4 TUI feature areas
2. Runs the **full regression suite** and confirms all 9 v1.2 requirements are met
3. Performs a **milestone audit** (`gsd-audit-milestone`) against original intent
4. Does any required **docs updates** (README, CHANGELOG)
5. Closes the milestone via `gsd-complete-milestone`

</domain>

<decisions>
## Implementation Decisions

### TUI Validation Approach (primary discussion topic)

- **D-01:** TUI features are validated via a **manual walkthrough checklist** — no automated TUI test harness introduced in this phase
- **D-02:** The checklist is delivered as a **UAT.md file in the phase 18 directory** (`.planning/phases/18-validation-ship-readiness/UAT.md`)
- **D-03:** UAT.md covers **all 4 TUI feature areas**:
  1. Task grouping (`g` toggle) — group headers, nav skip, status bar indicator
  2. Deferred task toggle (`h` toggle) — `t:` tasks hidden by default, DIM when visible, `[+deferred]` indicator
  3. Filter Esc/restore — quick filter panel cancel behavior
  4. Filter persist/reload — configured filters survive a full restart (cold-load verification)
- **D-04:** Each scenario in UAT.md has clear PASS/FAIL criteria — the executor runs the TUI binary and marks each item manually

### Regression Testing

- **D-05:** Run `cargo test --workspace` and confirm all tests remain green — no new test additions required unless a gap is discovered during the audit
- **D-06:** If the milestone audit uncovers a test gap (e.g., missing coverage for `--all` flag deferred filtering in CLI), add the missing test(s) in the same plan

### Requirements Coverage Audit

- **D-07:** All 9 v1.2 requirements must be verified before milestone completion:
  - V12-COMPAT-01, V12-COMPAT-02 (phases 14–15)
  - V12-TUI-FILTER-01, V12-TUI-FILTER-02, V12-TUI-FILTER-03 (phase 16)
  - V12-TUI-STATUS-01, V12-TUI-GROUP-01 (phase 17)
  - V12-TUI-DEFER-01, V12-TUI-DEFER-02 (phases 14, 17)

### Agent's Discretion

- Docs scope (README, CHANGELOG detail level) — planner decides what's appropriate for a point release
- Whether any test gaps found during audit warrant a separate plan or inline fix — executor decides at runtime
- Release artifact (git tag, GitHub release) — out of scope for this phase; planner may note as a backlog item if relevant

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements baseline
- `.planning/REQUIREMENTS.md` — all 9 v1.2 requirements and traceability table
- `.planning/ROADMAP.md` — phase completion status and requirement mapping

### Prior phase verification artifacts
- `.planning/phases/15-todo-sh-compat-layer/` — compat tests and summaries
- `.planning/phases/16-tui-filter-ux-alignment/` — filter UX summaries
- `.planning/phases/17-tui-grouping-sorting-alignment-status-polish/17-VERIFICATION.md` — 11/11 automated truths verified; 2 visual checks pending (h and g)

### Implementation
- `crates/todotxt-tui/src/app.rs` — `show_deferred`, `grouping`, `DisplayRow`, `rebuild_display_indices()` — primary TUI logic for h/g features
- `crates/todotxt-core/src/filter.rs` — `suppress_future_threshold` — deferred task filtering hook
- `crates/todotxt-cli/tests/compat_tests.rs` — existing compat test surface

</canonical_refs>

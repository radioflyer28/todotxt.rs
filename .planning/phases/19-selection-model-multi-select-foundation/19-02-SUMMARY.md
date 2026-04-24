---
phase: 19-selection-model-multi-select-foundation
plan: 02
subsystem: ui
tags: [ratatui, selection, anchor, shift-range, tui, multi-select]

# Dependency graph
requires:
  - "19-01 (HashSet<usize> selected_tasks, selection_anchor: Option<usize>, disjoint_select: bool)"
provides:
  - "ensure_anchor: lazily initializes selection_anchor from cursor canonical index (D-11)"
  - "apply_range_selection: replaces selected_tasks with [anchor..cursor] task range (D-09)"
  - "Shift+j/k extend contiguous range selection downward/upward (D-09)"
  - "Shift+Down/Up extend contiguous range selection downward/upward (D-09)"
  - "Shift+Ctrl+D/U extend range by half-page (D-10)"
  - "Non-shift j/k/Down/Up/Ctrl+D/Ctrl+U clear selection_anchor (D-12)"
affects: [phase-20-bulk-actions]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TDD RED/GREEN cycle per plan task (tasks 1 and 2)"
    - "Shift arms placed BEFORE plain arms in match for deterministic dispatch (T-19-04)"
    - "ensure_anchor + apply_range_selection helper composition for range extension"
    - "GroupHeader rows skipped in both shift navigation and range population (D-08)"

key-files:
  created: []
  modified:
    - crates/todotxt-tui/src/app.rs

key-decisions:
  - "D-09: Both Shift+j/k AND Shift+Down/Up extend the contiguous range from the anchor"
  - "D-10: Shift+Ctrl+D/U extend range by half-page (same movement math as plain Ctrl+D/U)"
  - "D-11: First shift-nav lazily sets anchor to current cursor canonical index"
  - "D-12: Non-shift navigation clears anchor ONLY — does NOT clear selected_tasks set"
  - "T-19-04 mitigated: Shift+Ctrl arms placed before plain Ctrl arms; Shift arms before plain arms"

requirements-completed: [SEL-01]

# Metrics
duration: ~20min
completed: 2026-04-24
---

# Phase 19 Plan 02: Anchor Lifecycle + Shift-Range Key Handlers Summary

**Contiguous range selection implemented: ensure_anchor/apply_range_selection helpers drive Shift+j/k/Down/Up/Ctrl+D/U key matrix, with anchor lifecycle (lazy set on first shift-nav, clear on plain nav, never clear selected_tasks).**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-04-24T00:00:00Z
- **Completed:** 2026-04-24T00:00:00Z
- **Tasks:** 3 completed
- **Files modified:** 1

## Accomplishments

- Added `ensure_anchor()` — lazily initializes `selection_anchor` from cursor canonical index when None (D-11); no-op if anchor already set
- Added `apply_range_selection()` — clears and repopulates `selected_tasks` with all `DisplayRow::Task` indices between anchor and cursor display rows (D-09); skips `GroupHeader` rows (D-08)
- Implemented shift-range key matrix: `Shift+j`, `Shift+k`, `Shift+Down`, `Shift+Up` — each calls `ensure_anchor` then moves cursor then calls `apply_range_selection`
- Added `Shift+Ctrl+D` and `Shift+Ctrl+U` — reuse half-page movement math, then apply range selection (D-10)
- Plain navigation arms (`j/k/Down/Up/Ctrl+D/Ctrl+U`) now clear `selection_anchor` without touching `selected_tasks` (D-12)
- Shift arms placed BEFORE plain arms in the `match` for deterministic dispatch (T-19-04 mitigation)

## Task Commits

Each task was committed atomically (TDD RED/GREEN for tasks 1 and 2):

1. **Task 1 RED: failing tests for anchor lifecycle** - `4367240` (test)
2. **Task 1 GREEN: ensure_anchor + apply_range_selection helpers** - `26be6fa` (feat)
3. **Task 2 GREEN: shift-range key matrix** - `186c55f` (feat)
4. **Task 3: Shift+Ctrl+D/U half-page range extension** - `3533b87` (feat)

Note: Tasks 1 and 2 shared a single RED commit covering both sets of failing tests (anchor lifecycle tests and shift-range key tests).

## Files Created/Modified

- `crates/todotxt-tui/src/app.rs` — Added `ensure_anchor`, `apply_range_selection` helpers; Shift+j/k/Down/Up/Ctrl+D/Ctrl+U arms in `handle_normal_key`; anchor clear on plain nav

## Deviations from Plan

**1. [Rule 2 - Missing Critical Functionality] Combined RED test commit for Tasks 1 and 2**
- **Found during:** Task 1 RED writing
- **Issue:** Task 2 shift-nav tests naturally depend on the Task 1 helpers (`ensure_anchor`, `apply_range_selection`) — separating them into two RED commits would have caused partial compile failures
- **Fix:** Combined both Task 1 and Task 2 RED tests into a single `test(...)` commit, then implemented Task 1 GREEN first (helper methods + anchor clear), confirming Task 1 tests pass before moving to Task 2 GREEN (shift key handlers)
- **Files modified:** `crates/todotxt-tui/src/app.rs`

## TDD Gate Compliance

- RED gate (test commit): ✅ `4367240` — failing tests for all 3 groups (ensure_anchor, apply_range_selection, shift keys)
- GREEN gate Task 1 (feat commit): ✅ `26be6fa` — anchor lifecycle helpers + plain nav anchor clear
- GREEN gate Task 2 (feat commit): ✅ `186c55f` — shift-range key handlers
- Task 3 REFACTOR: N/A — no TDD required, direct feat implementation

## Known Stubs

None — all key handlers are fully wired. Half-page shift behavior is functional (uses live `self.list_height` which is 0 in tests but correct at runtime).

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| T-19-04 mitigated | crates/todotxt-tui/src/app.rs | Shift+Ctrl arms placed before plain Ctrl arms; SHIFT arms before plain arms in match |
| T-19-05 mitigated | crates/todotxt-tui/src/app.rs | apply_range_selection iterates DisplayRow::Task only; GroupHeader never enters selected_tasks |
| T-19-06 mitigated | crates/todotxt-tui/src/app.rs | ensure_anchor and apply_range_selection both guard with canonical_selected(); no unwraps |

---
phase: 19-selection-model-multi-select-foundation
plan: 03
subsystem: ui
tags: [ratatui, selection, persistence, reload, prune, tui, multi-select]

# Dependency graph
requires:
  - "19-01 (HashSet<usize> selected_tasks, disjoint mode)"
  - "19-02 (ensure_anchor, apply_range_selection, shift-range keys)"
provides:
  - "prune_stale_selections(): retain idx < task_list.len(), clear out-of-range anchor"
  - "D-18: rebuild_display_indices/rebuild_and_reanchor never clear selected_tasks"
  - "D-19: FileChanged reload and apply_pending_reload prune out-of-range indices"
  - "D-20: filter-hidden tasks remain selected; reappear selected on filter clear"
affects: [phase-20-bulk-actions]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "TDD RED/GREEN cycle per plan task"
    - "prune_stale_selections called on both immediate (FileChanged) and deferred (apply_pending_reload) reload paths"
    - "HashSet::retain for in-place conditional pruning (no clone needed)"

key-files:
  created: []
  modified:
    - crates/todotxt-tui/src/app.rs

key-decisions:
  - "D-18 already satisfied: rebuild_display_indices/rebuild_and_reanchor never touched selected_tasks; Task 1 tests confirmed this without code changes"
  - "D-19: prune_stale_selections uses HashSet::retain(|&idx| idx < task_list.len()) for clean in-place pruning"
  - "D-20: filter path tested and verified — selected_tasks not cleared during filter rebuilds"
  - "TDD Task 1: fail-fast triggered — behavior already correct from Phase 19 Plan 01 implementation"

requirements-completed: [SEL-03, SEL-04]

# Metrics
duration: ~15min
completed: 2026-04-24
---

# Phase 19 Plan 03: Selection Persistence Through Rebuild and Reload Summary

**Selection persistence completed: D-18/D-20 already correct (rebuild paths never touch selected_tasks), D-19 implemented via new prune_stale_selections() called on both reload paths.**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-24T17:27:44Z
- **Completed:** 2026-04-24
- **Tasks:** 2 completed (Task 3 optional — not needed; status bar indicator sufficient from Phase 19-01 rendering)
- **Files modified:** 1

## Accomplishments

- **Task 1:** Wrote tests verifying D-18 and D-20 (`rebuild_and_reanchor_does_not_clear_selected_tasks`, `rebuild_display_indices_does_not_clear_selected_tasks`, `filter_hidden_tasks_remain_selected_d20`, `sort_change_does_not_clear_selected_tasks`). All passed immediately — fail-fast rule applied; feature was already correct from Plan 01 (HashSet fields added without any rebuild-clear code).
- **Task 2:** RED — wrote 3 failing tests for selection pruning (`reload_prunes_out_of_range_selections`, `reload_clears_out_of_range_anchor`, `reload_retains_valid_anchor`). GREEN — added `prune_stale_selections()` method and called it after both `AppEvent::FileChanged` immediate reload and `apply_pending_reload()` deferred reload.

## Task Commits

1. **Task 1 RED + Task 2 RED** — `6050616` — `test(tui): selection persistence + reload pruning RED tests [19-03]`
2. **Task 2 GREEN** — `e0ff61e` — `feat(tui): prune out-of-range selections on reload [19-03]`

## Files Created/Modified

- `crates/todotxt-tui/src/app.rs` — Added `prune_stale_selections()`, called on both reload paths; 7 new tests

## Deviations from Plan

**1. [Rule N/A - Fail-Fast] Task 1: implementation already correct**
- **Found during:** Task 1 RED
- **Issue:** All Task 1 tests passed immediately in RED phase. Fail-fast rule applied: feature already existed (rebuild functions never contained selected_tasks clearing code).
- **Fix:** No code changes needed for Task 1. Tests committed as verification of existing correct behavior.
- **Files modified:** none (tests only)

**2. [Rule N/A] Task 3 (optional status-bar indicator) not needed**
- **Issue:** The plan marked Task 3 as optional. The existing rendering already shows `> ` prefix + BOLD for selected tasks, providing sufficient visual testability. No status-bar count added.

## TDD Gate Compliance

- Task 1 RED gate: ✅ `6050616` (tests committed — all passed immediately, fail-fast documented)
- Task 1 GREEN gate: ✅ No implementation needed (fail-fast: feature already correct)
- Task 2 RED gate: ✅ `6050616` — 2 tests failed as expected
- Task 2 GREEN gate: ✅ `e0ff61e` — all 32 tests pass

## Known Stubs

None — selection state is fully wired; pruning is live on both reload paths.

## Threat Flags

None — no new network endpoints, file access patterns, or trust boundary crossings introduced. `prune_stale_selections` is purely in-memory mutation.

## Self-Check: PASSED

- [x] `crates/todotxt-tui/src/app.rs` — exists and modified ✅
- [x] Commits `6050616`, `e0ff61e` exist in git log ✅
- [x] `cargo test -p todotxt-tui` → 32 passed; 0 failed ✅
- [x] `cargo check -p todotxt-tui` → Finished cleanly ✅

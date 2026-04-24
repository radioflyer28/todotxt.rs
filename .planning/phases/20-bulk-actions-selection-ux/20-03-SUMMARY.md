---
phase: 20-bulk-actions-selection-ux
plan: 03
subsystem: todotxt-tui
tags:
  - status-bar
  - selection-visibility
  - bulk-actions-ux
dependency_graph:
  requires:
    - 20-01 (selection foundation)
    - 20-02 (bulk delete/append)
  provides:
    - Selection count visibility in status bar
    - Bulk action key hints in status bar
  affects:
    - render_status_bar output
    - Discoverability of D/T/v/Shift+nav keys
tech_stack:
  added: []
  patterns:
    - Status bar left-segment string building (push_str pattern)
    - Conditional indicator rendering
key_files:
  created: []
  modified:
    - crates/todotxt-tui/src/app.rs (render_status_bar + unit tests)
decisions:
  - D-12: `| N selected` appended to left segment when !selected_tasks.is_empty()
  - D-14: No separate `[v]` prefix when disjoint_select=true (keeps status bar uncluttered)
  - Bulk action keys (D/T/v/Shift+nav) added to right hint string for discoverability
metrics:
  duration_minutes: 8
  completed_date: "2026-04-24"
  tasks_completed: 2
  tests_added: 3
  tests_passed: 42
  files_modified: 1
---

# Phase 20 Plan 03: Selection Count Indicator Summary

**Status:** ✅ COMPLETE

**Objective:** Make selection state discoverable — add `| N selected` to the status bar left segment and update the hint string to surface bulk action keys (D, T, v, Shift+nav).

**One-liner:** Status bar displays selected task count and bulk action keys for user discoverability.

## Completed Tasks

| Task | Name | Status | Commit |
|------|------|--------|--------|
| 1 | Add '| N selected' to status bar + update hint keys | ✅ Done | d33370d |
| 2 | Add unit tests + verify full test suite | ✅ Done | d33370d |

## Implementation Details

### Task 1: Status Bar Indicator + Hint String Update

**Location:** `crates/todotxt-tui/src/app.rs`, `render_status_bar()` function (line ~1414)

**Changes:**
1. **Selection count indicator** — Added after the `due_today/overdue` block:
   ```rust
   // Selection count indicator — only shown when tasks are selected (D-12, D-14)
   if !self.selected_tasks.is_empty() {
       left.push_str(&format!(" | {} selected", self.selected_tasks.len()));
   }
   ```
   - Appends `| N selected` to the left segment when any tasks are selected
   - No output when `selected_tasks` is empty (no regression)
   - Works regardless of `disjoint_select` state — shows count without `[v]` prefix (D-14)

2. **Hint string update** — Updated the `right` help text from:
   ```
   "  q quit | n add | u edit | d del | x done | j/k nav | f filter | ^f filt on/off | F define | o sort | g group | h deferred | t theme"
   ```
   to:
   ```
   "  q quit | n add | u edit | d del | D bulk del | T bulk app | v sel | Shift+nav range | x done | j/k nav | f filter | ^f filt on/off | F define | o sort | g group | h deferred | t theme"
   ```
   - Added `D bulk del` (bulk delete key)
   - Added `T bulk app` (bulk append key)
   - Added `v sel` (selection/disjoint mode key)
   - Added `Shift+nav range` (range selection hint)

### Task 2: Unit Tests + Full Test Suite

**Location:** `crates/todotxt-tui/src/app.rs`, `mod tests` section (line ~2325)

**Tests Added:**
1. `status_bar_selection_indicator_absent_when_empty()` — Verifies no indicator when `selected_tasks.is_empty()`
2. `status_bar_selection_indicator_present_when_tasks_selected()` — Verifies `| N selected` appears with N > 0
3. `status_bar_disjoint_mode_shows_count_not_v_prefix()` — Verifies D-14: count shown, no `[v]` prefix

**Test Results:**
- ✅ All 42 tests pass (3 new + 39 existing)
- ✅ No failures, no regressions
- ✅ Full coverage of selection indicator logic

## Verification Results

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Selection indicator in status bar | ✅ PASS | `grep "selected_tasks.len()"` returns match at line 1451 (inside render_status_bar) |
| Hint includes D bulk del | ✅ PASS | `grep "D bulk del"` returns match at line 1472 |
| Hint includes T bulk app | ✅ PASS | `grep "T bulk app"` returns match at line 1472 |
| Compilation clean | ✅ PASS | `cargo build -p todotxt-tui` exits 0 (1.52s dev build) |
| Unit tests pass | ✅ PASS | `cargo test -p todotxt-tui` exits 0 (42/42 passing) |
| No test failures | ✅ PASS | Zero failures, zero skipped |

## Deviations from Plan

**None** — plan executed exactly as written.

## Threat Surface Scan

No new security-relevant surfaces introduced. The status bar changes are display-only; no auth paths, network endpoints, or trust boundary modifications.

## Known Stubs

None — plan fully implemented with no placeholders or data stubs.

## Self-Check: PASSED

- ✅ File `crates/todotxt-tui/src/app.rs` exists
- ✅ Commit d33370d verified in git log
- ✅ All acceptance criteria met
- ✅ Tests pass
- ✅ Build succeeds

## Notes

**BULK-03 Requirement Satisfied:**
- Status bar now displays selection count indicator when tasks are selected
- Bulk action keys (D, T, v, Shift+nav) are discoverable in the hint string
- Users can see how many tasks are selected before running bulk operations
- No regression when selection is empty

**Design Decisions Honored:**
- D-12: Selection indicator follows existing left-segment string-building pattern
- D-14: No `[v]` prefix when disjoint_select is true — status bar stays uncluttered
- Bulk action keys integrated seamlessly into existing hint string

**Phase 20 Completion:**
With Plan 20-03 complete, Phase 20 (bulk-actions-selection-ux) is now **fully implemented**:
- ✅ Plan 20-01: Selection foundation (Phase 19 integration) — COMPLETE
- ✅ Plan 20-02: Bulk delete + bulk append operations — COMPLETE
- ✅ Plan 20-03: Selection visibility + hint keys — COMPLETE

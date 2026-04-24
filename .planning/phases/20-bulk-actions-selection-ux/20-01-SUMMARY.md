---
phase: 20
plan: 01
subsystem: TUI
tags: [bulk-delete, selection, hotkey, confirmation]
completed: 2026-04-24
duration: 1h
task_count: 2
file_count: 1
started_from_task: 1
all_checkpoints_passed: true
requires: [19-01, 19-02, 19-03]
provides: [BULK-01, PAR-01]
affects: [20-02, 20-03, 22-help-audit]
tech_stack:
  added: []
  patterns: [Descending-index bulk delete, Mode-conditional rendering]
key_files:
  created: []
  modified: [crates/todotxt-tui/src/app.rs]
key_decisions:
  - D-01: "D (Shift+d) hotkey when !selected_tasks.is_empty() enters DeleteConfirm mode; plain d unchanged"
  - D-02: "Confirmation panel shows 'Delete N tasks?' for >1 task, task preview for 1 task, cursor preview for empty selection"
  - D-03: "Bulk deletion in descending index order prevents index shifts invalidating subsequent deletes"
  - D-04: "After bulk delete, selected_tasks.clear() and disjoint_select = false"
---

# Phase 20 Plan 01: Bulk Delete Hotkey + Confirmation Summary

**D (Shift+d) bulk delete with count-aware confirmation and descending-index safety**

Wire the Phase 19 multi-selection set into a safe bulk delete flow, implementing BULK-01.

## What Was Built

### Task 1: Add D Hotkey Dispatch

**Location:** crates/todotxt-tui/src/app.rs, `handle_normal_key` (line 446)

Implemented:
- New `KeyCode::Char('D')` arm with guard `!self.selected_tasks.is_empty()`
- Sets `self.mode = AppMode::DeleteConfirm` to reuse existing confirmation panel
- Placed before plain `d` arm to follow established pattern (separate char codes, no ordering conflict)
- Plain `d` with empty selection retains existing single-delete behavior (no regression)

**Verification:** ✅ Build clean, D arm present in code

### Task 2: Bulk Delete with Descending Index Safety + Rendering

**Locations:** crates/todotxt-tui/src/app.rs

#### handle_delete_confirm_key (lines 878–921)

Branched on `selected_tasks.len()`:

**Bulk path (len > 0):**
- Extract indices from `selected_tasks` into `sorted_indices: Vec<usize>`
- Sort descending: `sorted_indices.sort_unstable_by(|a, b| b.cmp(a))`
- Delete in descending order: highest index first prevents index shifts
- Clear `selected_tasks` and reset `disjoint_select = false` (D-04)

**Single-task path (len == 0):**
- Use `self.canonical_selected()` to get cursor task
- Delete single task (existing behavior)

**Cancel path (any non-y key):**
- Clear selection and anchor when canceling bulk operation (D-04 cancel-path cleanup)
- Retain existing single-task cancel behavior

#### render_delete_confirm (lines 1412–1438)

Count-aware message rendering:
- `selected_tasks.len() > 1` → `"Delete N tasks?  y=confirm  any=cancel"` (D-02)
- `selected_tasks.len() == 1` → Show task preview (D-02, single-task-via-selection)
- `selected_tasks.is_empty()` → Show cursor task preview (D-02, legacy path)

#### Tests (lines 2085–2146)

Added 3 comprehensive tests:

1. **bulk_delete_descending_order** (line 2085)
   - Create app with 3 tasks, select indices 0 and 2
   - Confirm deletion with 'y'
   - Verify task list has 1 task (original index 1), selected_tasks is empty, disjoint_select is false
   - Confirms descending-order deletion prevented index-shift corruption

2. **bulk_delete_cancel_clears_selection** (line 2115)
   - Create app with 3 tasks, select indices 0 and 2, enable disjoint_select
   - Cancel deletion with 'n'
   - Verify no tasks deleted, selected_tasks empty, anchor cleared, disjoint_select reset
   - Confirms D-04 cancel-path cleanup

3. **bulk_delete_multiple_tasks_shows_count** (line 2143)
   - Validates that >1 selected task triggers bulk count rendering path
   - Confirms D-02 rendering logic branch

**Test Results:** ✅ All 3 bulk_delete tests pass; all 34 app module tests pass (no regressions)

## Deviations from Plan

None — plan executed exactly as written. All must-haves implemented and verified.

## Threat Surface Audit

No new trust boundaries or security surface introduced:
- Indices validated before deletion (task_list.delete bounds-checked internally)
- No new file I/O or network paths
- Selection state limited to HashSet<usize> — no user-controlled strings
- Descending-order deletion is correctness requirement, not security feature

## Known Stubs

None — bulk delete path is fully implemented.

## Verification Checklist

✅ D (Shift+d) hotkey enters DeleteConfirm when `!selected_tasks.is_empty()`  
✅ Confirmation shows "Delete N tasks?" for multi-task selection  
✅ Confirmation shows task preview for single-task selection  
✅ Existing cursor-task preview shown when selection is empty  
✅ Deletion happens in descending index order  
✅ selected_tasks cleared after bulk delete  
✅ disjoint_select reset to false after bulk delete  
✅ Plain d with empty selection retains existing behavior  
✅ Build compiles clean  
✅ All tests pass (34/34 in app module)

## Acceptance Criteria Status

| Criterion | Status | Evidence |
|-----------|--------|----------|
| D arm present with guard | ✅ PASS | Line 446: `KeyCode::Char('D') if !self.selected_tasks.is_empty()` |
| Descending-order deletion | ✅ PASS | Line 899: `sort_unstable_by(\|a, b\| b.cmp(a))` |
| Count message rendering | ✅ PASS | Line 1426: `format!("Delete {} tasks?...", self.selected_tasks.len())` |
| Test coverage | ✅ PASS | 3 tests, all passing (lines 2085–2146) |
| No regressions | ✅ PASS | 34/34 app tests pass |
| Compilation | ✅ PASS | `cargo build -p todotxt-tui` → 0 |

## Git Commit

**Hash:** 0c1d89a  
**Message:** `feat(20-01): implement bulk delete with D hotkey and descending-index safety`  
**Files:** `crates/todotxt-tui/src/app.rs` (+134, −14 lines)

## Requirements Traceability

**BULK-01** (bulk delete confirmed via count):
- ✅ `D` on non-empty selection enters DeleteConfirm
- ✅ Confirmation shows count when >1 task selected
- ✅ `y` deletes all selected tasks in descending order
- ✅ After bulk delete, selection is cleared

**PAR-01** (hotkey parity with todotxt.net):
- ✅ `D` (Shift+d) hotkey matches .NET Client/Controls/MainWindow.xaml binding

## Notes for Next Phase

Phase 20-02 (bulk append) will reuse this pattern:
- New `AppMode::AppendText` variant (similar to `Editing`)
- Hotkey `T` on non-empty selection
- Same descending-order iteration for symmetry
- Same selection cleanup after append

Phase 22 (help/keymap parity) should consider adding `D=bulk-delete` to the help screen (currently not shown per D-13 decision to keep help minimal in v1.3).

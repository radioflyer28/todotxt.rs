---
phase: 36-recovery-path-short-horizon-undo
plan: 01
status: complete
commit: b91811c
---

# Plan 36-01 Summary: Undo Infrastructure

## What Was Built

Single-level undo machinery for the TUI: `UndoEntry` type, `replace_all` on `TaskList`, `push_undo_entry()`/`apply_undo()` on `App`, and the Ctrl+Z key dispatch arm.

## Key Files

### Created / Modified
- `crates/todotxt-tui/src/state.rs` — Added `UndoEntry { tasks: Vec<Task>, selected: usize }` struct after `FilterDefiningState`
- `crates/todotxt-core/src/task_list.rs` — Added `replace_all(tasks: Vec<Task>) -> Result<(), TodoError>` after `delete()`
- `crates/todotxt-tui/src/app.rs` — Added `undo_entry: Option<UndoEntry>` field, `push_undo_entry()`, `apply_undo()` methods, Ctrl+Z dispatch arm in `handle_normal_key`

## TDD Results

All 5 tests written RED-first, then made GREEN:

| Test | Result |
|------|--------|
| `push_then_apply_restores_task_list` | ✓ PASS |
| `apply_undo_when_empty_is_no_op` | ✓ PASS |
| `second_push_overwrites_first` | ✓ PASS |
| `apply_undo_clears_entry` | ✓ PASS |
| `ctrl_z_in_normal_mode_triggers_apply_undo` | ✓ PASS |

## Self-Check: PASSED

- [x] `UndoEntry` struct exported from `state.rs`
- [x] `replace_all` method present on `TaskList` in `task_list.rs`
- [x] `App.undo_entry: Option<UndoEntry>` field present and initialized to `None`
- [x] `push_undo_entry()` and `apply_undo()` methods present on `App`
- [x] Ctrl+Z arm present in `handle_normal_key` before any plain `z` arm
- [x] All 5 TDD tests pass
- [x] `cargo build` succeeds with no errors

## Deviations

None. Implementation followed the plan spec exactly.

## Enables for Wave 2

`push_undo_entry()` is ready to be called at each mutation site in `app.rs`. Plan 02 wires it into all 10 call sites.

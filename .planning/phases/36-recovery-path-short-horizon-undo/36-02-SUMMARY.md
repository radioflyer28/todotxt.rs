---
phase: 36-recovery-path-short-horizon-undo
plan: 02
status: complete
commit: eb53a96
---

# Plan 36-02 Summary: Wire Mutation Sites

## What Was Built

`push_undo_entry()` wired into all 10 mutation sites in `app.rs`. Every CRUD action now captures a restorable snapshot before mutating the task list. Three integration tests verify end-to-end undo round-trips.

## Mutation Sites Wired

| Site | Function | Location |
|------|----------|----------|
| 1 | `save_and_exit()` AppMode::Adding | Before `task_list.add(task)` |
| 2 | `save_and_exit()` AppMode::Editing | Before `task_list.update(original_idx, task)` |
| 3 | `delete_active_task()` | Before `task_list.delete(idx)` |
| 4 | `handle_delete_confirm_key()` 'y' arm | Before single/bulk delete branches |
| 5 | `paste_from_clipboard()` | Before the `for line in lines` loop |
| 6 | `apply_token_to_tasks()` | Before `batch_update` in non-empty replacements block |
| 7 | `handle_append_text_key()` | Before `batch_update` in non-empty text branch |
| 8 | `handle_date_picker_key()` | Before `batch_update` in non-empty replacements block |
| 9 | `handle_priority_picker_key()` | Before `batch_update` in non-empty replacements block |
| 10 | `toggle_done()` and `pane_toggle_done()` | Before `task_list.update(idx, toggled)` |

## Verification

```
grep push_undo_entry count: 17 matches (1 def + 10 call sites + tests)
cargo build: Finished, no errors
```

## Integration Tests

| Test | Result |
|------|--------|
| `delete_undo_round_trip` | ✓ PASS |
| `add_undo_round_trip` | ✓ PASS |
| `toggle_undo_round_trip` | ✓ PASS |

## Self-Check: PASSED

- [x] All 10 mutation sites have `self.push_undo_entry()` before first task_list mutation
- [x] 3 integration tests pass (delete/add/toggle round-trips)
- [x] All Plan 01 TDD tests still pass (6 total undo tests pass)
- [x] `cargo build` succeeds with no errors

## Deviations

None. All sites follow the insertion rule from the plan spec.

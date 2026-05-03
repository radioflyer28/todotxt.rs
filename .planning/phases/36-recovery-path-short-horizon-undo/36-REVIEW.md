---
phase: 36-recovery-path-short-horizon-undo
reviewed: 2026-04-30T00:00:00Z
depth: standard
files_reviewed: 3
files_reviewed_list:
  - crates/todotxt-tui/src/state.rs
  - crates/todotxt-core/src/task_list.rs
  - crates/todotxt-tui/src/app.rs
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: issues_found
---

# Phase 36: Code Review Report

**Reviewed:** 2026-04-30  
**Depth:** standard  
**Files Reviewed:** 3  
**Status:** PASS_WITH_NOTES — no blocking issues; three low-severity observations noted

---

## Summary

Phase 36 adds depth-1 undo to the TUI. The implementation is structurally sound:
`push_undo_entry()` correctly captures state **before** every mutation across all 11 call
sites. `apply_undo()` uses `.take()` to consume the entry (entry-consumed semantics are
correct; second Ctrl+Z is always a no-op as designed). `replace_all()` delegates to the
existing atomic-rename `save()` path — no new partial-write surface. All 8 tests have
meaningful assertions; the round-trip integration tests (delete, add, toggle) verify wired
call sites end-to-end.

No critical or warning-level issues were found. Three info-level observations are noted
below.

---

## Info

### IN-01: Spurious undo snapshot when single-task delete has no selection

**File:** `crates/todotxt-tui/src/app.rs:2001`  
**Issue:** In `handle_delete_confirm_key`, `push_undo_entry()` is called unconditionally
on `KeyCode::Char('y')`, before branching on `selected_tasks.is_empty()`. In the
single-task branch the delete is further gated on `active_canonical_selected()` returning
`Some`. If it returns `None` (nothing focused, edge case in a multi-pane layout with a
concurrent reload), a snapshot is captured but nothing is mutated. The undo slot is then
consumed by the first subsequent Ctrl+Z, silently "restoring" to the identical state and
burning the previous valid undo entry.

```rust
if key.code == KeyCode::Char('y') {
    self.push_undo_entry();                         // captured unconditionally
    if self.selected_tasks.is_empty() {
        if let Some(idx) = self.active_canonical_selected() {  // may be None
            self.task_list.delete(idx) ...
        }
        // Nothing deleted → undo slot wasted
    }
```

**Fix:** Guard the snapshot so it is only captured when a real mutation will occur.
For the single-task path, move `push_undo_entry()` inside the `if let Some(idx)` arm.
For the multi-task path it can stay (selection is non-empty so deletion will proceed).

```rust
if key.code == KeyCode::Char('y') {
    if self.selected_tasks.is_empty() {
        if let Some(idx) = self.active_canonical_selected() {
            self.push_undo_entry();          // only when a task exists to delete
            self.task_list.delete(idx) ...
            ...
        }
    } else {
        self.push_undo_entry();              // multi-task path: deletion will occur
        ...
    }
}
```

---

### IN-02: `apply_undo()` restores global cursor only — per-pane cursors not captured

**File:** `crates/todotxt-tui/src/app.rs:271–293`  
**Issue:** `push_undo_entry()` snapshots `self.selected` (the global display-row index)
but not the per-pane `pane.selected` positions. After `apply_undo()` calls
`rebuild_all_panes()`, each pane's cursor is re-initialised to its default (row 0 or
whichever `rebuild_and_reanchor` chooses). In a single-pane layout this is invisible; in a
multi-pane layout a user who was, say, on row 5 of pane 2 will find that cursor reset to 0
after undo even though the task list was correctly restored.

This is within the stated design scope (depth-1, cursor-restored = `self.selected`), so it
is not a regression — but it is worth documenting as a known UX gap for future multi-pane
undo work.

**Fix (optional):** Extend `UndoEntry` with a `Vec<usize>` of pane cursor positions and
restore them after `rebuild_all_panes()`:

```rust
pub struct UndoEntry {
    pub tasks: Vec<todotxt_core::Task>,
    pub selected: usize,
    pub pane_cursors: Vec<usize>,   // one entry per pane
}
```

No action required for Phase 36 scope.

---

### IN-03: In-memory state diverges from disk if `replace_all` save fails

**File:** `crates/todotxt-core/src/task_list.rs:205–208`  
**Issue:** `replace_all()` follows the same pattern as `add()`, `update()`, and `delete()`:
it mutates `self.tasks` in-memory first, then calls `save()`. If `save()` returns an error:

- In-memory `self.tasks` has been overwritten with the restored snapshot.
- The disk file is unchanged (the atomic rename never occurred — the original is safe).
- `self.undo_entry` is `None` (consumed by `.take()` before the call).

The user's display therefore shows the "restored" state while the file still holds the
post-mutation state. On restart the app would reload the mutated content. The propagated
error surfaces the failure, so the user is not silently left in a bad state, but the
divergence itself is worth noting.

This is a pre-existing design pattern across all `TaskList` mutation methods, not a new
regression introduced by Phase 36. Noting it here so it is on record for a future
`reload-on-error` or WAL-style improvement.

**Fix (low priority):** On `save()` failure, roll back `self.tasks` before returning the
error. This is safe since the original value can be preserved:

```rust
pub fn replace_all(&mut self, tasks: Vec<Task>) -> Result<(), TodoError> {
    let previous = std::mem::replace(&mut self.tasks, tasks);
    if let Err(e) = self.save() {
        self.tasks = previous;   // roll back in-memory on disk-write failure
        return Err(e);
    }
    Ok(())
}
```

No action required for Phase 36 scope; consistent with the existing codebase pattern.

---

## Verdict

**PASS_WITH_NOTES**

The undo implementation is correct and safe. All 11 `push_undo_entry()` call sites capture
state before the corresponding mutation. The atomic-rename save path provides sufficient
durability. The 8 tests are meaningful and cover the primary happy paths and edge cases
(empty history, depth-1 overwrite, entry consumption, Ctrl+Z dispatch, and three full
round-trip integration tests). The three observations above are informational only and do
not block ship.

---

_Reviewed: 2026-04-30_  
_Reviewer: gsd-code-reviewer (standard depth)_

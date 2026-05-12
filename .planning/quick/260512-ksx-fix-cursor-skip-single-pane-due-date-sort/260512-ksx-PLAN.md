---
id: 260512-ksx
slug: fix-cursor-skip-single-pane-due-date-sort
title: Fix cursor skip after grouping toggle + due-date sort (single-pane)
date: 2026-05-12
must_haves:
  - cursor navigates to adjacent task (no skipping) after toggling grouping off
  - cursor navigates to adjacent task (no skipping) after setting sort to DueDate
  - all existing tests remain green
  - regression test added and passing
---

# PLAN: Fix cursor skip after grouping toggle + due-date sort (single-pane)

## Problem

After toggling grouping OFF and/or setting sort to DueDate in single-pane view, pressing
Down arrow skips the middle task instead of landing on the next adjacent task.

## Root Cause

`rebuild_and_reanchor()` in `app.rs` called `rebuild_visible_rows()` (which updates
`pane.display_rows`) only when `!self.should_show_single_pane() && self.panes.len() > 1`.

In single-pane mode, `rebuild_display_indices()` was called (updating `self.display_rows`)
but `pane.display_rows` was never refreshed. This left stale `GroupHeader` rows in
`pane.display_rows` even after grouping was disabled.

**Bug scenario** (3 tasks, each in its own group):

1. Grouping ON → `pane.display_rows` = `[GroupHeader, Task(0), GroupHeader, Task(1), GroupHeader, Task(2)]` (6 rows)
2. Toggle grouping OFF → `self.display_rows` updated to `[Task(0), Task(1), Task(2)]` but
   `pane.display_rows` stays at 6 rows with stale group headers
3. Set sort = DueDate → same stale state
4. Press Down: `pane.selected=0` → `next=1` = Task(0) ok → `pane.selected=1`
5. Press Down again: `pane.selected=1` → `next=2` = `GroupHeader` → SKIP → `next=3` = Task(1),
   `pane.selected=3`, `self.selected=3` (out of bounds for 3-item `self.display_rows`)

Navigation then skips the visually second task entirely.

## Fix

In `rebuild_and_reanchor()`, remove the `if !self.should_show_single_pane() && self.panes.len() > 1`
condition so that `rebuild_visible_rows()` and the pane cursor reanchor always execute in both
single-pane and multi-pane modes.

**File**: `crates/todotxt-tui/src/app.rs`

### Before
```rust
if !self.should_show_single_pane() && self.panes.len() > 1 {
    self.rebuild_visible_rows();
    // reanchor pane.selected ...
}
```

### After
```rust
// Must run in ALL modes (single-pane and multi-pane) so pane.display_rows stays in sync
// with self.display_rows. Without this, grouping/sort changes leave pane.display_rows stale,
// causing pane_move_down/up to skip tasks based on stale group headers.
self.rebuild_visible_rows();
// Reanchor the active pane's cursor after rebuild.
let new_pane_selected = old_canonical
    .and_then(|ci| { ... })
    .unwrap_or(0);
let pane = &mut self.panes[self.active_pane];
pane.selected = new_pane_selected;
if pane.selected >= pane.display_rows.len() && !pane.display_rows.is_empty() {
    pane.selected = pane.display_rows.len() - 1;
}
```

## Side Effects

None. `rebuild_visible_rows()` already handles single-pane mode correctly (checks
`should_show_single_pane()` internally, uses `pane_idx=0`). Multi-pane behavior is identical.

## Verification

- Regression test added: `app::tests::cursor_does_not_skip_middle_task_after_grouping_off_and_due_date_sort`
  - Creates 3 tasks with distinct due dates (each in its own group)
  - Enables grouping (builds stale `pane.display_rows` with headers)
  - Disables grouping, sets sort to DueDate, calls `rebuild_and_reanchor()`
  - Asserts `pane.display_rows` has 3 `Task` rows (no `GroupHeader`)
  - Asserts first Down press lands on row 1, second Down press lands on row 2
- All 229 lib unit tests pass: `cargo test -p todotxt-tui --lib`

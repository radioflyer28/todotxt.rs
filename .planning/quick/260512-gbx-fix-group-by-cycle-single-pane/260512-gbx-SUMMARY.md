---
id: 260512-gbx
status: complete
title: Fix group_by_cycle having no visual effect in single-pane mode
date: 2026-05-12
commit: a51e281
---

# SUMMARY: Fix group_by_cycle having no visual effect in single-pane mode

## What was done

Fixed a bug in the Rust TUI where pressing `g` (group_by_cycle) in single-pane mode had
no visible effect when no sort was applied. The group headers in the task list never changed
regardless of what group-by category was selected.

## Root Cause

In `rebuild_and_reanchor()`, the single-pane sync block copied `pane.filter_query`,
`pane.sort_order`, and `pane.grouping` to App-level globals but omitted `pane.group_by`.

`rebuild_display_indices()` (the global render path, used by `render_task_list`) reads
`self.group_by` to determine group headers. Since `self.group_by` was never updated, it
defaulted to `Priority` on init and stayed there permanently. Pressing `g` updated
`pane.group_by` but `self.display_rows` (rendered to the screen) never changed.

The reason it *seemed* to work with sort applied: `pane.display_rows` (used for navigation)
had a different task ordering from `self.display_rows` after group cycling, causing observable
navigation differences even though the visible group headers didn't change.

## Fix

One-line change in `rebuild_and_reanchor()` — added to the existing sync block:

```rust
if self.should_show_single_pane() || self.panes_hidden {
    let pane = &self.panes[self.active_pane];
    self.filter_query = pane.filter_query.clone();
    self.sort_order = pane.sort_order;
    self.grouping = pane.grouping;
    self.group_by = pane.group_by;  // ← ADDED
}
```

**File changed:** `crates/todotxt-tui/src/app.rs`

**Side effect (correct):** The secondary stable-sort step in `rebuild_display_indices`
also reads `self.group_by`. Now it correctly sorts by the active category (DueDate groups
sort by due date, Project groups by project, etc.) instead of always sorting by Priority.

## Tests

- New regression test: `group_by_cycle_changes_display_rows_in_single_pane`
- Updated existing test `group_key_for_groups_by_correct_field_per_variant`: was setting
  `app.group_by` directly (a workaround for the bug). Updated to set `pane.group_by` which
  is the canonical path after the fix.
- All 230 lib unit tests pass.

## Must-haves verification

- [x] pressing `g` visibly changes group headers in single-pane view
- [x] group headers reflect active group_by category (Priority/Project/Context/DueDate)
- [x] status bar shows correct group-by (self.group_by now synced, no separate change needed)
- [x] secondary sort in rebuild_display_indices uses active pane.group_by
- [x] all existing tests green (230/230)
- [x] regression test added and passing

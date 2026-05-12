---
id: 260512-gbx
slug: fix-group-by-cycle-single-pane
title: Fix group_by_cycle having no visual effect in single-pane mode
date: 2026-05-12
must_haves:
  - pressing group_by_cycle key (g) visibly changes group headers in single-pane view
  - group headers in single-pane reflect the active pane group_by category (Priority/Project/Context/DueDate)
  - status bar shows the correct group-by category in single-pane mode
  - secondary sort in rebuild_display_indices uses the active pane.group_by (not always Priority)
  - all existing tests remain green
  - regression test added confirming group_by cycles visually (display_rows) in single-pane mode
---

# PLAN: Fix group_by_cycle having no visual effect in single-pane mode

## Root Cause

In `rebuild_and_reanchor()` (`crates/todotxt-tui/src/app.rs`), when in single-pane mode,
the function syncs per-pane state to App-level globals so `rebuild_display_indices()` (the
global render path used by `render_task_list`) picks up the right values:

```rust
if self.should_show_single_pane() || self.panes_hidden {
    let pane = &self.panes[self.active_pane];
    self.filter_query = pane.filter_query.clone();
    self.sort_order = pane.sort_order;
    self.grouping = pane.grouping;
    // MISSING: self.group_by = pane.group_by  ← BUG
}
```

`self.group_by` is never synced. `rebuild_display_indices()` therefore always builds
`self.display_rows` using `self.group_by = Priority` (the App-level default, never updated).
`render_task_list` renders from `self.display_rows`, so group headers never change.

`render_status_bar` has the same issue in its single-pane fallback branch — it reads
`self.group_by` instead of `pane.group_by`.

## Fix

### Task 1: Sync self.group_by in rebuild_and_reanchor

**File:** `crates/todotxt-tui/src/app.rs`

In the `rebuild_and_reanchor` single-pane sync block, add one line:

```rust
if self.should_show_single_pane() || self.panes_hidden {
    let pane = &self.panes[self.active_pane];
    self.filter_query = pane.filter_query.clone();
    self.sort_order = pane.sort_order;
    self.grouping = pane.grouping;
    self.group_by = pane.group_by;  // ADD THIS
}
```

No other code needs to change. `rebuild_display_indices` and `render_status_bar` both read
`self.group_by`, so they automatically correct once the sync is in place.

**Verify:** After pressing `g` twice (Priority → Project → Context), `self.display_rows`
must contain group headers reflecting the active group-by category.

**Done:** `self.group_by` equals `pane.group_by` immediately after `rebuild_and_reanchor` in single-pane mode.

### Task 2: Regression test

**File:** `crates/todotxt-tui/src/app.rs` (tests module)

Add test `group_by_cycle_changes_display_rows_in_single_pane`:
- Create app with tasks spanning multiple priorities and projects
- Enable grouping (Priority)
- Call `group_by_cycle` action (cycle to Project)
- Assert `display_rows` contains `GroupHeader` entries that DO NOT start with `(A)`, `(B)`, etc.
  (i.e., not Priority headers — confirms the group type changed visually)
- Assert `pane.display_rows` and `self.display_rows` have the same structure (in sync)

**Done:** Test passes with the fix, fails without it.

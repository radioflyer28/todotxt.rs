---
id: 260512-ksx
status: complete
title: Fix cursor skip after grouping toggle + due-date sort (single-pane)
date: 2026-05-12
commit: c89610d
---

# SUMMARY: Fix cursor skip after grouping toggle + due-date sort (single-pane)

## What was done

Fixed a bug in the Rust TUI (`todotxt-tui`) where navigating with the Down arrow after
toggling grouping off and setting sort to DueDate in single-pane view caused the cursor
to skip the middle task entirely.

## Root cause

`rebuild_and_reanchor()` in `crates/todotxt-tui/src/app.rs` only called `rebuild_visible_rows()`
(which updates `pane.display_rows`) in multi-pane mode. In single-pane mode, the App-level
`self.display_rows` was refreshed but `pane.display_rows` was left stale, still containing
`GroupHeader` rows from when grouping was active. Navigation (`pane_move_down`) then skipped
over those stale headers, causing the visible cursor to jump two positions instead of one.

## Fix

One-line logic change: removed the `if !self.should_show_single_pane() && self.panes.len() > 1`
guard around `rebuild_visible_rows()` so it runs unconditionally in both single-pane and
multi-pane modes.

**File changed**: `crates/todotxt-tui/src/app.rs`

## Tests

- New regression test: `app::tests::cursor_does_not_skip_middle_task_after_grouping_off_and_due_date_sort`
- All 229 lib unit tests pass.

## Must-haves verification

- [x] cursor navigates to adjacent task (no skipping) after toggling grouping off
- [x] cursor navigates to adjacent task (no skipping) after setting sort to DueDate
- [x] all existing tests remain green (229/229)
- [x] regression test added and passing

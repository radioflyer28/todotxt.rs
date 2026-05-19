---
created: 2026-05-15T17:17:37
title: Expand Rust TUI date setter target cycle and week jumps
area: tui
files:
  - crates/todotxt-tui/src/app.rs:1449-1466
  - crates/todotxt-tui/src/app.rs:2310-2388
  - crates/todotxt-tui/src/app.rs:2846-3010
  - crates/todotxt-tui/src/state.rs:182-250
---

## Problem

In the Rust TUI, pressing `s` currently opens the date picker for due date only.
Users need to keep cycling by hand through due, recurring, threshold, and completed targets
before using the picker, and navigating dates with Left/Right currently only moves panes in
Normal mode (or day-by-day in picker), making week jumps cumbersome.

## Solution

1. Update the Normal-mode `s` action to track a new date-target cycle state and cycle through
   `due -> recurring -> threshold -> completed` on repeated presses, while keeping a single
   target selected for the active picker session.
2. Show the active date target in the picker UI text and route apply-on-Enter to mutate
   the selected task(s) date slot.
3. Ensure target-aware mutation calls the correct task updater:
   - due date currently maps to `Task::with_due_date`
   - threshold maps to `Task::with_threshold_date`
   - completed likely sets task completion metadata through `Task::with_completed`
   - recurring requires a check for existing recurring-task representation (if not yet supported,
     capture a follow-up follow-up for core parsing/mutation support first).
4. Extend picker Left/Right handling so holding or pressing those keys jumps by week
   in the currently displayed month view (±7 days), while still allowing existing row-level
   day selection behavior.
5. Add/adjust tests around:
   - `s` cycling behavior in TUI mode transitions
   - applying the new date target path
   - week-step movement in the date picker


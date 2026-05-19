# Phase 48 Research: Recurring Workflow Core

**Phase:** 48-recurring-workflow-core
**Date:** 2026-05-18
**Status:** Complete

## Summary

Phase 48 should introduce recurrence behavior in `todotxt-core` first, then route CLI and
TUI completion through the same core contract. The important product decision is that
recurring completion is implicit: completing a task with a valid `rec:` token creates the
next incomplete occurrence automatically, without prompting.

## Existing Shape

- `Task::with_completed(true)` stamps today's completion date and strips priority on the
  completed copy, per todo.txt completion semantics.
- `Task` already preserves unknown body tokens such as `rec:+1w`, because only known
  structured tags like `due:` and `t:` are lifted into fields.
- `TaskList::add` appends a task and saves, while `TaskList::update` replaces a task and
  saves. This is enough for recurrence, but execution should prefer batching through
  `replace_all` where multiple updates/appends must be persisted together.
- CLI completion is centralized in `crates/todotxt-cli/src/commands/complete.rs`.
- TUI completion currently has three paths in `crates/todotxt-tui/src/app.rs`:
  single-pane `toggle_done`, pane-aware `pane_toggle_done`, and `bulk_mark_done`.

## Recommended Implementation

Add recurrence helpers to `todotxt-core` so CLI and TUI reuse one behavior:

- `RecurrenceMode::{Strict, Relative}`
- `RecurrenceInterval` or equivalent parsed representation for day/week/month/year units.
- `Task::recurrence()` or equivalent helper that returns parsed recurrence metadata from
  the task's `rec:` token.
- `Task::next_recurring_occurrence(completion_date: NaiveDate) -> Option<Task>` or an
  equivalent shared function.

Recommended next-task behavior:

- Strict `rec:+...` anchors from the original task's `due_date` when present.
- Relative `rec:...` anchors from the completion date.
- If no due date exists, both modes anchor from the completion date.
- The next occurrence is incomplete, has no completion date, preserves priority from the
  original task, preserves creation date unless execution determines todo.txt recurrence
  compatibility requires updating it, preserves projects/contexts/unknown metadata, and
  receives the recalculated `due:` date.
- Invalid or unsupported `rec:` tokens should be preserved as plain metadata and should
  not create a next occurrence.

## Plan Split

1. `48-01` - Core recurrence parsing and next occurrence construction.
2. `48-02` - CLI `do` integration for single-ID and multi-ID completion.
3. `48-03` - TUI single-task and bulk mark-done integration.

## Documentation Reconciliation

The existing roadmap and requirements still mention prompt-driven recurring generation.
Phase 48 context supersedes that. Planning should update docs to say recurring completion
automatically creates the next occurrence with no prompt.

## Testing Targets

Core tests should cover:

- `rec:+1d` strict recurrence from prior `due:`.
- `rec:1d` relative recurrence from completion date.
- no-due fallback to completion date.
- metadata preservation for priority, creation date, project/context tags, `t:`, `rec:`,
  and unknown key-value tokens.
- completed-state reset on the next occurrence.
- invalid `rec:` token creates no next occurrence.

CLI tests should cover:

- `todotxt do <id>` completes the original recurring task and appends one next occurrence.
- multi-ID `do` creates one next occurrence for each recurring task.
- non-recurring completion behavior remains unchanged.

TUI tests should cover:

- pane-aware single-task completion creates one next occurrence.
- bulk mark-done creates one next occurrence per selected recurring task.
- already-completed tasks do not create duplicate occurrences.
- undo snapshot still captures both completion and generated occurrence.

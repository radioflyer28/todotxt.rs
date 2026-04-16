---
phase: 05
plan: 04
subsystem: cli-commands
tags: [due, postpone, date-parsing, task-enrichment]
dependency_graph:
  requires: [05-02]
  provides: [run_due, run_postpone]
  affects: [crates/todotxt-cli/src/commands/due.rs]
tech_stack:
  added: []
  patterns: [with_due_date, parse_date_input, validate_id]
key_files:
  created: []
  modified:
    - crates/todotxt-cli/src/commands/due.rs
decisions:
  - Used Task.with_due_date(Option<NaiveDate>) directly — cleaner than raw tag manipulation since core already parses/serializes due:YYYY-MM-DD
  - Exiting with CliError::Other for missing-due-date in postpone (maps to exit 2 per D-08)
metrics:
  duration: ~10m
  completed: 2026-04-15
requirements:
  - ENRICH-03
  - ENRICH-04
---

# Phase 05 Plan 04: Due/Postpone Commands Summary

**One-liner:** Due/postpone commands with shared parse_date_input utility and with_due_date Task API.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Implement due and postpone commands | 3674d49 | crates/todotxt-cli/src/commands/due.rs |

## Implementation Details

### run_due
- Parses date argument via `crate::date::parse_date_input(date, today)`
- Validates ID with local `validate_id` helper
- Loads TaskList, calls `task.with_due_date(Some(due_date))`, saves
- Output: `"Set due date to YYYY-MM-DD on task #N."`

### run_postpone
- Validates ID, loads task
- Reads `task.due_date` — returns `CliError::Other` (exit 2) if `None` (D-08)
- Adds `Duration::days(N)` to existing date
- Calls `task.with_due_date(Some(new_due))`, saves
- Output: `"Postponed task #N by N day(s) to YYYY-MM-DD."`

## Deviations from Plan

None — plan executed exactly as written. Used `with_due_date()` directly (cleaner than raw tag string manipulation) since Task struct already has native `due_date: Option<NaiveDate>` and serializes it to `due:YYYY-MM-DD` on save.

## Self-Check: PASSED

- [x] `crates/todotxt-cli/src/commands/due.rs` exists with both functions
- [x] Commit 3674d49 exists
- [x] `cargo build -p todotxt-cli` passes
- [x] `cargo clippy -p todotxt-cli -- -D warnings` passes

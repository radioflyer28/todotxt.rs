---
phase: 48-recurring-workflow-core
plan: 03
subsystem: tui-completion
tags: [rust, tui, recurrence, undo, testing]
requires:
  - phase: 48-01
    provides: shared recurrence parser and next-occurrence construction
provides:
  - "Single-task TUI completion auto-generates recurring follow-ups"
  - "Bulk mark-done auto-generates one follow-up per recurring task"
  - "Undo and selection semantics preserved through recurring completion"
affects: [phase-48-completion, pane-toggle-done, bulk-mark-done]
tech-stack:
  added: []
  patterns: [shared app helper for batch completion, replace_all persistence]
key-files:
  created: []
  modified: [crates/todotxt-tui/src/app.rs]
key-decisions:
  - "Single-task and bulk TUI completion share one recurrence helper"
  - "Recurring generation happens only when transitioning incomplete to complete"
  - "Bulk completion still clears selection and preserves one undo snapshot"
requirements-completed: [REC-02, REC-03, REC-04]
duration: 14min
completed: 2026-05-18
---

# Phase 48 Plan 03: TUI Recurring Completion Summary

The TUI now applies the same recurrence behavior as the CLI: completing a recurring task
immediately appends the next occurrence, both for single-task completion and bulk
mark-done. Existing undo and pane rebuild behavior stays intact.

## Performance

- **Duration:** 14 min
- **Completed:** 2026-05-18
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments

- Added an App helper that batch-completes tasks and appends recurring follow-ups.
- Routed `toggle_done`, `pane_toggle_done`, and `bulk_mark_done` through that helper when
  moving incomplete tasks to completed.
- Added TUI tests for single recurring completion and bulk recurring completion.

## Verification

Passed:

```powershell
cargo test -p todotxt-tui recurring_tui
cargo test -p todotxt-tui
```

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

None.

## User Setup Required

None.

## Next Phase Readiness

Phase 48 is ready for completion. Phase 49 can build archive hygiene on top of the updated
completion behavior.

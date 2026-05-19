---
phase: 48-recurring-workflow-core
plan: 01
subsystem: core-task-model
tags: [rust, core, recurrence, todo.txt, testing]
requires: []
provides:
  - "Shared recurrence parser for rec:+... and rec:..."
  - "Deterministic next-occurrence construction from completion date"
  - "Metadata-preserving recurring task regeneration"
affects: [48-02, 48-03, task-mutation]
tech-stack:
  added: []
  patterns: [task-local recurrence helper, deterministic date-based helper]
key-files:
  created: []
  modified: [crates/todotxt-core/src/task.rs]
key-decisions:
  - "Recurring rules are parsed from existing rec: tokens without introducing sidecar storage"
  - "Strict recurrence anchors from original due date, relative recurrence anchors from completion date"
  - "Next occurrence preserves non-completion metadata and recalculates due:"
requirements-completed: [REC-01, REC-03]
duration: 15min
completed: 2026-05-18
---

# Phase 48 Plan 01: Core Recurrence Support Summary

`todotxt-core` now understands recurring task rules directly from `rec:` tokens and can
construct the next incomplete occurrence from an explicit completion date. This gives CLI
and TUI one shared recurrence contract instead of each surface inventing its own version.

## Performance

- **Duration:** 15 min
- **Completed:** 2026-05-18
- **Tasks:** 3
- **Files modified:** 1

## Accomplishments

- Added `RecurrenceMode`, `RecurrenceUnit`, and `RecurrenceRule`.
- Added `Task::recurrence_rule()` and `Task::next_recurring_occurrence(...)`.
- Implemented strict, relative, and no-due fallback anchoring behavior.
- Added tests for valid recurrence parsing, invalid token rejection, date anchoring, and
  metadata preservation.

## Verification

Passed:

```powershell
cargo test -p todotxt-core recurrence
cargo test -p todotxt-core
```

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

None.

## User Setup Required

None.

## Next Phase Readiness

CLI and TUI completion paths can now call one shared recurrence helper.

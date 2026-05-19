---
phase: 50-input-ergonomics
plan: 01
subsystem: tui-date-picker-and-core-date-mutation
tags: [rust, tui, date-picker, ergonomics, testing]
requires: []
provides:
  - "TUI date picker target cycling across due, threshold, and completed dates"
  - "Left/Right week-jump navigation in the TUI date picker"
  - "Explicit completion-date mutation support in todotxt-core"
affects: [50-02, quick-setters, task-builders]
tech-stack:
  added: []
  patterns: [target-aware picker state, shared task builders, focused tui regression tests]
key-files:
  created: []
  modified:
    - crates/todotxt-core/src/task.rs
    - crates/todotxt-core/tests/task_tests.rs
    - crates/todotxt-tui/src/state.rs
    - crates/todotxt-tui/src/app.rs
key-decisions:
  - "Phase 50 was executed on the Rust TUI/core path after correcting an initial mistaken client-side start"
  - "The TUI keeps a single `s` entry point and cycles picker targets in-place instead of introducing a second date-entry workflow"
  - "Completed-date assignment uses a dedicated core builder so explicit picked dates do not depend on 'today'"
requirements-completed: [DATE-UX-01, DATE-UX-02]
duration: 30min
completed: 2026-05-19
---

# Phase 50 Plan 01: TUI Date Picker Ergonomics Summary

The TUI date picker now covers all real date-bearing task fields in scope for this milestone:
due date, threshold date, and completed date. It also gained `Left`/`Right` week jumps so
calendar movement is faster without giving up direct `YYYY-MM-DD` typing.

## Performance

- **Duration:** 30 min
- **Completed:** 2026-05-19
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Added `DatePickerTarget` and target-aware picker state in the TUI.
- Made `s` cycle the active picker target through due, threshold, and completed dates.
- Added week-jump navigation with `Left` and `Right`.
- Added `Task::with_completion_date(...)` in `todotxt-core` so picked completed dates are explicit.
- Added focused tests for target cycling, week jumps, threshold application, and completed-date application.

## Verification

Passed:

```powershell
cargo test -p todotxt-core
cargo test -p todotxt-tui
```

## Deviations from Plan

- The original plan artifact referenced desktop client dialog files. Execution was intentionally
  corrected to the Rust TUI/core implementation after scope was clarified.

## Issues Encountered

- One quick-setter regression test initially assumed the typed prefix would disappear from the
  candidate list. The implementation was correct; the test was relaxed to match the existing
  quick-setter candidate contract.

## User Setup Required

None.

## Next Phase Readiness

The date picker now exposes the right target state for future refinements, and the quick-setter
layer can build on the same continuity-first interaction model.

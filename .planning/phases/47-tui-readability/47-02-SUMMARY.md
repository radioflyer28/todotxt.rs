---
phase: 47-tui-readability
plan: 02
subsystem: tui-row-model
tags: [rust, tui, grouping, navigation, testing]
requires:
  - phase: 47-01
    provides: active-only pane cursor highlight rendering
provides:
  - "Blank spacer rows before non-first group headers"
  - "Task-only selection normalization for grouped structural rows"
  - "Single-pane and multi-pane grouped spacer parity"
affects: [phase-47-completion, grouped-views, pane-navigation]
tech-stack:
  added: []
  patterns: [structural DisplayRow variants, task-row normalization helper]
key-files:
  created: []
  modified:
    - crates/todotxt-tui/src/state.rs
    - crates/todotxt-tui/src/app.rs
    - crates/todotxt-tui/src/components/pane_list.rs
key-decisions:
  - "Represented blank group spacing as DisplayRow::GroupSpacer"
  - "Inserted spacer rows before every non-first group header in both grouped row builders"
  - "Normalized selection to task rows so headers and spacers remain non-selectable structure"
requirements-completed: [TUI-02]
duration: 10min
completed: 2026-05-15
---

# Phase 47 Plan 02: Group Spacer Rows and Task-Only Navigation Summary

Grouped TUI views now include true blank rows before non-first group headers. The spacer is
a structural display row, so rendering, status counts, selection, and navigation all treat
it consistently as non-task UI structure.

## Performance

- **Duration:** 10 min
- **Completed:** 2026-05-15
- **Tasks:** 4
- **Files modified:** 3

## Accomplishments

- Added `DisplayRow::GroupSpacer`.
- Inserted spacers in both visible-pane and all-pane grouped row builders.
- Rendered spacers as blank list rows in both pane and single-pane render paths.
- Updated selection normalization and pane movement to skip spacers and headers.
- Added tests for grouped spacer placement, no leading spacer, multi-pane parity, and spacer-skipping movement.

## Verification

Passed:

```powershell
cargo test -p todotxt-tui group_spacer
cargo test -p todotxt-tui grouped_rows
cargo test -p todotxt-tui pane_move
cargo test -p todotxt-tui
```

Note: `cargo test -p todotxt-tui tui_readability` passed but matched zero tests; the meaningful
coverage is in the concrete filters above and the full crate test suite.

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

None.

## User Setup Required

None.

## Next Phase Readiness

Phase 47 is ready for completion. Phase 48 can proceed without additional TUI readability
work.


---
phase: 47-tui-readability
plan: 01
subsystem: tui-rendering
tags: [rust, tui, ratatui, panes, testing]
requires: []
provides:
  - "Active-only pane cursor highlight rendering"
  - "Unit coverage for inactive pane render selection behavior"
affects: [47-02, multi-pane-rendering]
tech-stack:
  added: []
  patterns: [render helper for selected-row visibility]
key-files:
  created: []
  modified: [crates/todotxt-tui/src/components/pane_list.rs]
key-decisions:
  - "Inactive panes keep pane.selected but pass no selected row to ratatui ListState"
  - "Active pane remains the only pane with a visible cursor highlight"
requirements-completed: [TUI-01]
duration: 4min
completed: 2026-05-15
---

# Phase 47 Plan 01: Active-Only Pane Cursor Highlight Summary

Inactive panes now preserve their remembered selected row without rendering a cursor
highlight. `PaneList::render` delegates selected-row visibility to a focused helper so the
active pane gets `Some(pane.selected)` and inactive panes get `None`.

## Performance

- **Duration:** 4 min
- **Completed:** 2026-05-15
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Added `PaneList::selected_row_for_render`.
- Updated `PaneList::render` to gate ratatui `ListState` selection on `is_active`.
- Added tests for active pane selection, inactive pane suppression, and label-selected suppression.

## Verification

Passed:

```powershell
cargo test -p todotxt-tui inactive_pane_has_no_render_selected_row
cargo test -p todotxt-tui pane_list
```

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

None.

## User Setup Required

None.

## Next Phase Readiness

Plan 02 can rely on active-only highlighting being isolated to `PaneList`.


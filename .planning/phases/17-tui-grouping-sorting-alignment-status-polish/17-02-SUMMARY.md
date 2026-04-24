---
plan: 17-02
phase: 17-tui-grouping-sorting-alignment-status-polish
status: complete
completed: "2026-04-23"
---

# Plan 17-02: Grouped Display Rows + Navigation Safety — SUMMARY

## What Was Built

Implemented grouped task rendering with explicit display-row types and header rows,
plus navigation/action safety so task operations always resolve to real task rows.
Grouping is toggled with `g`, status displays `| group: on`, and deferred behavior from 17-01 remains intact.

## Changes Made

**crates/todotxt-tui/src/app.rs**
- Added `DisplayRow` enum with `Task(usize)` and `GroupHeader(String)` variants
- Added `grouping: bool` and `display_rows: Vec<DisplayRow>` to `App`
- Refactored `rebuild_display_indices` to populate `display_rows` and inject group headers by active sort key
- Added `group_key_for(task, sort)` helper to derive group labels across sort modes
- Refactored selection anchoring and canonical mapping to operate via `display_rows`
- Updated list rendering to draw group headers (REVERSED style) and task rows separately
- Updated `j/k` navigation to skip header rows and land only on task rows
- Replaced old `g` jump-to-top behavior with grouping toggle and re-anchor rebuild
- Added `| group: on` indicator to status bar when grouping is active
- Applied checkpoint feedback: removed `G` jump-to-bottom shortcut to avoid `g/G` top-bottom hotkey conflicts

## Self-Check: PASSED

- `cargo build -p todotxt-tui` exits 0
- Grouping/display-row symbols (`DisplayRow`, `display_rows`, `GroupHeader`, `group_key_for`) are present and wired
- No direct `display_indices[self.selected]` indexing remains
- `g` exists as grouping toggle behavior in Normal mode
- Human verification checkpoint approved after hotkey conflict adjustment

## Deviations from Plan

- Removed `G` jump-to-bottom behavior after checkpoint feedback. The original plan expected `G` to jump to last task row, but user testing requested removal of `g/G` top-bottom scrolling hotkeys.

## Key Files

- `crates/todotxt-tui/src/app.rs` — modified

## Issues Encountered

- Initial checkpoint feedback identified `g/G` hotkey conflict expectations; resolved by removing `G` jump behavior.

## Next Phase Readiness

- Grouping, deferred toggle, and status polish are in place for phase-level verification.

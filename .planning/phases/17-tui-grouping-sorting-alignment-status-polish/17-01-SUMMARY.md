---
plan: 17-01
phase: 17-tui-grouping-sorting-alignment-status-polish
status: complete
completed: "2026-04-23"
---

# Plan 17-01: Status Bar Cleanup + Deferred Toggle — SUMMARY

## What Was Built

Removed the status-bar theme label and implemented deferred task visibility controls:
`h` now toggles deferred visibility, the filter allows future-threshold tasks when enabled,
future-threshold tasks render dimmed, and `[+deferred]` appears in the status bar when active.

## Changes Made

**crates/todotxt-tui/src/app.rs**
- Added `show_deferred: bool` to `App` and initialized it in `App::new`
- Added `h` key handling in Normal mode to toggle deferred visibility and rebuild the display
- Updated `rebuild_display_indices` to set `filter.suppress_future_threshold = false` when deferred view is enabled
- Removed theme label rendering from the status bar
- Added `[+deferred]` status marker when deferred view is enabled
- Added deferred dim styling in the task list render path for tasks with `threshold_date > today`
- Updated right-side key hints to include `g group` and `h deferred`

## Self-Check: PASSED

- `cargo build -p todotxt-tui` exits 0
- `show_deferred` appears in app state, key handling, and filter wiring
- `suppress_future_threshold` override is present in `rebuild_display_indices`
- No `theme_label` or `theme:` status text remains in `app.rs`
- Deferred marker and deferred styling checks are present in rendering logic

## Key Files

- `crates/todotxt-tui/src/app.rs` — modified
- `crates/todotxt-tui/Cargo.toml` — modified (`chrono` workspace dependency)

## Issues Encountered

- Added `chrono` as a direct dependency of `todotxt-tui` to support local-date checks for deferred row dimming.

## Next Phase Readiness

- Ready for grouping refinements and navigation correctness checks in `17-02`.

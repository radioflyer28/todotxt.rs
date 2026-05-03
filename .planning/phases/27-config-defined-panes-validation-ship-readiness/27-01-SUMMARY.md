---
phase: 27-config-defined-panes-validation-ship-readiness
plan: 01
subsystem: ui
tags: [ratatui, serde, toml, pane-state, config]
requires:
  - phase: 26-pane-management-quick-hide-show
    provides: runtime pane lifecycle and active-pane state model
provides:
  - TuiConfig pane schema with defaults and snake_case sort values
  - Tolerant pane loading that skips invalid entries with warnings
  - Startup mapping from config pane blueprints into runtime panes
  - Quit-time projection of runtime pane state into config.panes and atomic save
affects: [v1.4 pane workflows, cfg requirements, startup config behavior]
tech-stack:
  added: []
  patterns: [serde defaulted schema evolution, fail-open config entry parsing, quit-only pane persistence]
key-files:
  created: []
  modified: [crates/todotxt-tui/src/config.rs, crates/todotxt-tui/src/app.rs, crates/todotxt-tui/tests/pane_integration_test.rs]
key-decisions:
  - "Keep PaneSort persisted as snake_case and map to runtime SortOrder via explicit conversion helpers."
  - "Treat malformed [[panes]] entries as non-fatal and skip per-entry during load with warnings."
  - "Persist pane blueprint state only on quit by projecting runtime panes into config.panes before save()."
patterns-established:
  - "Config list hardening: parse list entries individually and continue on malformed elements."
  - "Pane runtime/config round-trip: runtime pane fields map directly to config blueprint fields."
requirements-completed: [CFG-01, CFG-02, CFG-03]
duration: 20m
completed: 2026-04-29
---

# Phase 27 Plan 01: Config-Defined Panes Validation Ship Readiness Summary

**Config-defined pane blueprints now load safely at startup, malformed pane entries are skipped without blocking launch, and runtime pane state is persisted back to config on quit through atomic save.**

## Performance

- **Duration:** 20m
- **Started:** 2026-04-29T00:38:00Z
- **Completed:** 2026-04-29T00:57:55Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments

- Added `PaneConfig`/`PaneSort` schema to `TuiConfig` with backward-compatible serde defaults.
- Implemented tolerant `[[panes]]` loading so invalid entries are warned and skipped while valid entries still load.
- Bootstrapped runtime panes from config at app startup and added quit-path persistence of pane label/filter/sort/group fields.
- Added integration coverage for startup pane mapping, invalid-entry fallback, and quit-only persistence behavior.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add PaneConfig/TuiConfig schema for [[panes]] with defaults** - `99fa570` (feat)
2. **Task 2: Wire startup pane bootstrap and tolerant invalid-entry handling** - `a6151c2` (feat)
3. **Task 3: Persist runtime panes back to config on quit only** - `64ff31e` (feat)

**Plan metadata:** pending

## Files Created/Modified

- `crates/todotxt-tui/src/config.rs` - Added pane schema, sort conversion helpers, and tolerant per-entry pane parsing in `TuiConfig::load`.
- `crates/todotxt-tui/src/app.rs` - Added startup pane bootstrap from config and quit-time pane projection/save path.
- `crates/todotxt-tui/tests/pane_integration_test.rs` - Added pane bootstrap, invalid-entry fallback, and quit-persistence tests.

## Decisions Made

- Used explicit `PaneSort <-> SortOrder` conversion instead of serializing runtime `SortOrder` directly.
- Kept fallback behavior fail-open for `[[panes]]` by skipping malformed entries individually.
- Persisted only pane blueprint fields (`label`, `filter`, `sort`, `group`) and left session-only state unpersisted.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed deny(warnings) failure from unused helper during Task 2**

- **Found during:** Task 2 verification
- **Issue:** `PaneSort::from_sort_order` introduced for quit persistence caused a dead-code warning that failed compilation under `#![deny(warnings)]`.
- **Fix:** Marked helper with `#[allow(dead_code)]` until it became used by Task 3 persistence path.
- **Files modified:** `crates/todotxt-tui/src/config.rs`
- **Verification:** `cargo test -p todotxt-tui pane_integration_test -- --nocapture`
- **Committed in:** `a6151c2`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** No scope creep; fix was required to keep task verification unblocked.

## Issues Encountered

- Full test output exceeded tool output limits; verification status was confirmed by extracted test summary lines and explicit command pass/fail statuses.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- CFG-01/CFG-02/CFG-03 behavior and tests are in place and stable.
- PATH-01/PATH-02/PATH-03 work can now build on the same config load/save and startup bootstrap paths.

## Self-Check: PASSED

- Verified files exist: `crates/todotxt-tui/src/config.rs`, `crates/todotxt-tui/src/app.rs`, `crates/todotxt-tui/tests/pane_integration_test.rs`
- Verified task commits exist in history: `99fa570`, `a6151c2`, `64ff31e`

---
*Phase: 27-config-defined-panes-validation-ship-readiness*
*Completed: 2026-04-29*

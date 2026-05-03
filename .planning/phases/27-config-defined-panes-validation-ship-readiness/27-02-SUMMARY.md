---
phase: 27-config-defined-panes-validation-ship-readiness
plan: 02
subsystem: ui
tags: [rust, clap, tui, config, path-resolution]
requires:
  - phase: 27-01
    provides: "Config-defined pane startup and persistence baseline"
provides:
  - "TUI clap args for --todo/--archive/--config"
  - "Deterministic startup path precedence resolver"
  - "PATH-02 archive fallback to sibling done.txt when --todo is set"
affects: [startup, config, file-path-resolution]
tech-stack:
  added: [clap]
  patterns: ["single resolver for CLI+config precedence", "test-first path behavior validation"]
key-files:
  created: [crates/todotxt-tui/tests/path_resolution_test.rs]
  modified: [crates/todotxt-tui/src/main.rs, crates/todotxt-tui/src/config.rs, crates/todotxt-tui/Cargo.toml, Cargo.lock]
key-decisions:
  - "Resolved startup paths in config.rs via resolve_startup_paths to keep precedence logic unit-testable"
  - "Applied --config override before config load, then CLI-wins for todo/archive"
  - "When --todo is supplied without --archive, derive archive from todo parent as done.txt"
patterns-established:
  - "CLI overrides represented as a small struct passed to resolver"
  - "Filter-friendly test naming matches plan verification command"
requirements-completed: [PATH-01, PATH-02, PATH-03]
duration: 55min
completed: 2026-04-28
---

# Phase 27 Plan 02: Path Override Contract Summary

**TUI startup now supports clap-based file path overrides with deterministic CLI precedence and tested archive fallback derived from the override todo path.**

## Performance

- **Duration:** 55 min
- **Started:** 2026-04-28T23:59:00Z
- **Completed:** 2026-04-29T00:54:00Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments
- Added RED tests covering PATH-01/02/03 scenarios for precedence and defaulting.
- Implemented `--todo`, `--archive`, and `--config` in TUI startup with clap.
- Added `resolve_startup_paths` to centralize precedence and fallback semantics used by startup.
- Preserved startup validation by checking missing/unreadable todo path with clear errors.

## Task Commits

1. **Task 1 (RED): Add failing tests for PATH-01/02/03 precedence/defaulting** - `1f8307e` (test)
2. **Task 2 (GREEN): Implement clap args and path resolver logic in main startup** - `68a0ae9` (feat)
3. **Task 3 (REFACTOR): Harden and simplify resolver with full crate checks** - `6c1fd68` (refactor)

Additional plan-scoped support commit:
- `0a149c0` (chore): lockfile sync for added TUI clap dependency

## Files Created/Modified
- `crates/todotxt-tui/tests/path_resolution_test.rs` - New PATH-01/02/03 coverage.
- `crates/todotxt-tui/src/main.rs` - clap `Args` parsing and startup override wiring.
- `crates/todotxt-tui/src/config.rs` - Added resolver types/functions and archive fallback helper.
- `crates/todotxt-tui/Cargo.toml` - Added clap dependency.
- `Cargo.lock` - Synced dependency graph after adding clap.

## Decisions Made
- Resolver returns concrete startup paths as a `Result` to keep error messaging explicit when todo path is absent.
- PATH-02 fallback only triggers forced sibling default when `--todo` is provided without `--archive`.
- Test names include `path_resolution_test` so the plan command filter executes these tests.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Scoped verification command initially filtered out all new tests**
- **Found during:** Task 2 (GREEN)
- **Issue:** `cargo test -p todotxt-tui path_resolution_test -- --nocapture` ran 0 tests because function names did not match filter.
- **Fix:** Renamed test functions to include `path_resolution_test_` prefix.
- **Files modified:** `crates/todotxt-tui/tests/path_resolution_test.rs`
- **Verification:** Re-ran the same command; 5 tests executed and passed.
- **Committed in:** `68a0ae9`

**2. [Rule 3 - Blocking] Lockfile changed after adding clap dependency**
- **Found during:** Task 3 (REFACTOR)
- **Issue:** `Cargo.lock` became dirty after dependency graph update.
- **Fix:** Added dedicated lockfile sync commit.
- **Files modified:** `Cargo.lock`
- **Verification:** `git status --short` clean after commit.
- **Committed in:** `0a149c0`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Required for deterministic verification and reproducible dependency state. No scope creep.

## Issues Encountered
- None beyond expected RED compile failure and normal lockfile update from dependency addition.

## User Setup Required
None - no external service configuration required.

## Threat Flags
None.

## Known Stubs
None.

## Next Phase Readiness
- PATH-01/02/03 are implemented and test-validated.
- TUI startup path precedence is centralized and ready for ship-readiness documentation/checklist steps.

## Self-Check: PASSED
- Found summary file: `.planning/phases/27-config-defined-panes-validation-ship-readiness/27-02-SUMMARY.md`
- Found commits in log: `1f8307e`, `68a0ae9`, `0a149c0`, `6c1fd68`

---
*Phase: 27-config-defined-panes-validation-ship-readiness*
*Completed: 2026-04-28*

---
phase: 49-archive-hygiene
plan: 03
subsystem: tui-archive
tags: [rust, tui, archive, rotation, undo, feedback, testing]
requires: [49-01]
provides:
  - "Rotation-aware TUI archive flow"
  - "Explicit TUI feedback when done.txt rotates"
  - "Archive tests covering rotation, undo, and confirm behavior"
affects: [archive-confirm, undo, done.txt]
tech-stack:
  added: [filetime (dev-dependency)]
  patterns: [shared cadence helper, write-first archive flow, in-file app tests]
key-files:
  created: []
  modified:
    - crates/todotxt-tui/src/app.rs
    - crates/todotxt-tui/Cargo.toml
key-decisions:
  - "TUI archive confirmation keeps the existing undo boundary and adds rotation feedback only when needed"
  - "Rotation semantics are kept equivalent with CLI archive semantics"
  - "Archive tests use controlled file mtimes instead of fake period metadata"
requirements-completed: [DONE-01, DONE-02]
duration: 20min
completed: 2026-05-19
---

# Phase 49 Plan 03: TUI Archive Rotation Summary

The TUI archive flow now performs the same monthly rotation as the CLI path while preserving
its existing archive confirmation and undo behavior. When rotation happens, the TUI now tells
the user exactly which period file the old archive content moved into.

## Performance

- **Duration:** 20 min
- **Completed:** 2026-05-19
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Updated `archive_tasks()` to rotate prior-period `done.txt` content before new archive writes.
- Added a small atomic-write helper so rotated files and active `done.txt` use the same safe write pattern.
- Updated archive confirmation feedback to mention rotation explicitly.
- Added TUI tests for prior-period rotation and for user-facing rotation messaging.

## Verification

Passed:

```powershell
cargo test -p todotxt-tui archive
```

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

None.

## User Setup Required

None.

## Next Phase Readiness

CLI and TUI now share the same monthly archive rotation semantics, so Phase 49 can verify at
the milestone level as a complete archive-hygiene slice.

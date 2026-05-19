---
phase: 49-archive-hygiene
plan: 01
subsystem: core-archive-and-config
tags: [rust, archive, rotation, config, testing]
requires: []
provides:
  - "Shared time-based archive rotation decision helper"
  - "Monthly archive period naming such as done-YYYY-MM.txt"
  - "CLI/TUI cadence configuration with monthly defaults"
affects: [49-02, 49-03, archive-paths]
tech-stack:
  added: []
  patterns: [shared core helper, cadence-based config, deterministic path generation]
key-files:
  created: [crates/todotxt-core/src/archive.rs]
  modified:
    - crates/todotxt-core/src/lib.rs
    - crates/todotxt-cli/src/config.rs
    - crates/todotxt-tui/src/config.rs
key-decisions:
  - "Archive rotation is modeled in todotxt-core so CLI and TUI share one cadence contract"
  - "Monthly cadence ships first, but config terminology stays future-ready for later weekly-style expansion"
  - "Retention cleanup is intentionally deferred; this phase only rotates active done.txt into deterministic period files"
requirements-completed: [DONE-01, DONE-03]
duration: 20min
completed: 2026-05-19
---

# Phase 49 Plan 01: Shared Archive Rotation Foundation Summary

The archive system now has a shared cadence-aware rotation contract in `todotxt-core`, plus
matching CLI and TUI config fields. This gives both surfaces one place to decide when
`done.txt` should rotate and what the rotated period filename should be.

## Performance

- **Duration:** 20 min
- **Completed:** 2026-05-19
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Added `ArchiveRotationCadence`, `ArchivePeriod`, and `plan_archive_rotation(...)`.
- Added deterministic rotated archive naming like `done-2026-05.txt`.
- Exported the archive helper from `todotxt-core`.
- Added `archive_rotation_cadence` to CLI and TUI config with monthly defaults.
- Added focused tests for period bucketing, rotation decisions, and config deserialization.

## Verification

Passed:

```powershell
cargo test -p todotxt-core archive_rotation
```

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

None.

## User Setup Required

None. Existing configs continue to work and default to monthly rotation.

## Next Phase Readiness

CLI and TUI archive flows can now integrate the shared helper without reimplementing archive
period logic.

---
phase: 49-archive-hygiene
plan: 02
subsystem: cli-archive
tags: [rust, cli, archive, rotation, json, testing]
requires: [49-01]
provides:
  - "Rotation-aware CLI archive flow"
  - "Explicit CLI messaging when done.txt rotates"
  - "JSON archive output with rotated_to metadata"
affects: [archive-command, done.txt, cli-output]
tech-stack:
  added: [filetime (dev-dependency)]
  patterns: [write-first archive flow, deterministic rotation, integration test with controlled mtime]
key-files:
  created: []
  modified:
    - crates/todotxt-cli/src/commands/archive.rs
    - crates/todotxt-cli/tests/enrich_bulk_tests.rs
    - crates/todotxt-cli/Cargo.toml
key-decisions:
  - "CLI rotates only during archive writes that actually move completed tasks"
  - "Prior-period active done.txt content is moved into the deterministic period file before new active writes"
  - "CLI stderr and JSON output both expose rotation explicitly"
requirements-completed: [DONE-01, DONE-02]
duration: 20min
completed: 2026-05-19
---

# Phase 49 Plan 02: CLI Archive Rotation Summary

The CLI `archive` command now understands monthly rotation. If the active `done.txt` belongs
to an earlier period, the command moves that content into a deterministic period file before
writing the new active archive and then reports that rotation to the user.

## Performance

- **Duration:** 20 min
- **Completed:** 2026-05-19
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Updated `run_archive(...)` to consult the shared rotation helper before appending tasks.
- Preserved the existing write-first safety model: archive file first, then todo mutation.
- Added explicit rotation messaging on stderr.
- Extended JSON success output with `rotated_to`.
- Added integration coverage for same-period append and prior-period rotation behavior.

## Verification

Passed:

```powershell
cargo test -p todotxt-cli archive
```

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

One test harness import used the right predicate combinator but the wrong trait import; this
was corrected before the final verification pass.

## User Setup Required

None. Users can opt into other cadences later, but monthly is the default shipped behavior.

## Next Phase Readiness

The CLI archive path now matches the monthly rotation contract and provides good parity for
the TUI implementation.

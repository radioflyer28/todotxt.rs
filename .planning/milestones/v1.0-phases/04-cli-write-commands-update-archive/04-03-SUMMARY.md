---
phase: 04-cli-write-commands-update-archive
plan: 03
subsystem: cli
tags: [write-commands, do, undo, del]

requires:
  - phase: 04-cli-write-commands-update-archive
    plan: 02
    provides: "write command module stubs and CLI dispatch wiring"

provides:
  - "Full do and undo command implementations"
  - "Full del command implementation"
  - "Shared ID validation and descending-order multi-ID handling"

affects: []

tech-stack:
  added: []
  patterns:
    - "validate-all-ids then descending index mutation"
    - "idempotent do/undo behavior with stderr info and exit 0"
    - "render write results via Renderer.print_write_result"

key-files:
  created: []
  modified:
    - crates/todotxt-cli/src/commands/complete.rs
    - crates/todotxt-cli/src/commands/del.rs

key-decisions:
  - "do/undo reject empty id list with CliError::Other"
  - "del validates all ids before first delete to avoid partial deletes"
  - "all multi-id operations sort descending and deduplicate indices"

requirements-completed: [WRITE-02, WRITE-03, WRITE-04]

duration: 15min
completed: 2026-04-15
---

# Phase 04 Plan 03: do/undo/del Summary

Replaced stubs with complete implementations for completion and deletion commands, including idempotent behavior and fail-fast validation semantics.

## Performance

- Duration: ~15 min
- Tasks: 2
- Files modified: 2

## Accomplishments

- Implemented complete.rs with run_do and run_undo.
- Added shared validate_id helper in complete.rs.
- Implemented idempotency handling for already-completed and already-incomplete tasks via stderr info + continue.
- Implemented del.rs with validate-all-before-delete and descending-order deletion.
- Ensured write result output uses renderer.print_write_result for updated/deleted tasks.

## Verification

- cargo build -p todotxt-cli: passed
- cargo clippy -p todotxt-cli -- -D warnings: passed
- Symbol and behavior grep checks for run_do/run_undo/validate_id/list.delete: passed
- Verified no remaining todo! stubs in complete.rs and del.rs

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

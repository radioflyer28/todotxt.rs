---
phase: 48-recurring-workflow-core
plan: 02
subsystem: cli-completion
tags: [rust, cli, recurrence, integration-testing]
requires:
  - phase: 48-01
    provides: shared recurrence parser and next-occurrence construction
provides:
  - "CLI do command auto-generates next recurring occurrences"
  - "Multi-ID recurring completion batches original updates and generated follow-ups"
affects: [phase-48-completion, cli-do-command]
tech-stack:
  added: []
  patterns: [batch replace_all mutation, integration assertions on todo.txt file content]
key-files:
  created: []
  modified:
    - crates/todotxt-cli/src/commands/complete.rs
    - crates/todotxt-cli/tests/integration_tests.rs
key-decisions:
  - "CLI recurring completion is implicit and promptless"
  - "Generated next occurrences are appended during the same completion transaction"
  - "Already-completed recurring tasks remain skipped and do not duplicate follow-ups"
requirements-completed: [REC-02, REC-03, REC-04]
duration: 12min
completed: 2026-05-18
---

# Phase 48 Plan 02: CLI Recurring Completion Summary

The CLI `do` command now auto-generates the next occurrence for recurring tasks while
preserving existing single-task and multi-ID completion behavior. Recurring follow-ups are
created in the same batch update as the completed originals.

## Performance

- **Duration:** 12 min
- **Completed:** 2026-05-18
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments

- Reworked `run_do` to batch-complete selected tasks in memory.
- Appended exactly one next occurrence for each newly completed recurring task.
- Added CLI integration coverage for single-task and multi-ID recurring completion.

## Verification

Passed:

```powershell
cargo test -p todotxt-cli recurring_cli
cargo test -p todotxt-cli
```

## Deviations from Plan

None - plan executed as written.

## Issues Encountered

None.

## User Setup Required

None.

## Next Phase Readiness

TUI completion can now mirror the same implicit recurrence contract already proven in CLI.

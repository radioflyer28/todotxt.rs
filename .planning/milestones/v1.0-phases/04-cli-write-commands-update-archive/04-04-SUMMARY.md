---
phase: 04-cli-write-commands-update-archive
plan: 04
subsystem: cli
tags: [write-commands, edit, append, prepend]

requires:
  - phase: 04-cli-write-commands-update-archive
    plan: 02
    provides: "edit/append/prepend stubs and dispatch wiring"

provides:
  - "Full edit command implementation"
  - "Full append command implementation"
  - "Full prepend command implementation"

affects: []

tech-stack:
  added: []
  patterns:
    - "ID validation helper per command"
    - "Task::parse full-line replacement semantics for edit"
    - "append via format!(\"{} {}\", task.to_raw(), text) then Task::parse"
    - "prepend via Task::with_text_prepended"

key-files:
  created: []
  modified:
    - crates/todotxt-cli/src/commands/edit.rs
    - crates/todotxt-cli/src/commands/append.rs
    - crates/todotxt-cli/src/commands/prepend.rs

key-decisions:
  - "edit rejects empty replacement text"
  - "append and prepend preserve task identity by in-place update"
  - "all commands surface not-found with 1-based id error messages"

requirements-completed: [WRITE-05, WRITE-06, WRITE-07]

duration: 15min
completed: 2026-04-15
---

# Phase 04 Plan 04: edit/append/prepend Summary

Implemented the remaining mutation commands for text replacement and text composition, replacing all Wave 2 stubs.

## Performance

- Duration: ~15 min
- Tasks: 2
- Files modified: 3

## Accomplishments

- Implemented edit.rs as full replacement using Task::parse(new_text).
- Added empty replacement guard for edit to return CliError::Other.
- Implemented append.rs using D-05 append semantics with parse of combined raw line.
- Implemented prepend.rs using D-06 builder call task.with_text_prepended(text).
- Added ID validation and bounds checks for all three commands.

## Verification

- cargo build -p todotxt-cli: passed
- cargo clippy -p todotxt-cli -- -D warnings: passed
- Pattern checks for Task::parse(new_text), append parse format, and with_text_prepended: passed
- Verified no remaining todo! stubs in edit.rs/append.rs/prepend.rs

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

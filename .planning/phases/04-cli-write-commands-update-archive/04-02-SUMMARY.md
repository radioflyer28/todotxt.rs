---
phase: 04-cli-write-commands-update-archive
plan: 02
subsystem: cli
tags: [write-commands, clap, add-command, stubs]

requires:
  - phase: 04-cli-write-commands-update-archive
    plan: 01
    provides: "with_text_prepended builder, auto_creation_date config, print_write_result"

provides:
  - "CLI wiring for 7 write subcommands (add/do/undo/del/edit/append/prepend)"
  - "Full add command implementation with --date and --no-date semantics"
  - "Compilable stubs for complete/del/edit/append/prepend command modules"

affects: []

tech-stack:
  added: []
  patterns:
    - "Clap subcommand enum expansion with command(alias/name) attributes"
    - "Main dispatch fan-out to command modules"
    - "TaskList add followed by read-back for final rendered task"

key-files:
  created:
    - crates/todotxt-cli/src/commands/add.rs
    - crates/todotxt-cli/src/commands/complete.rs
    - crates/todotxt-cli/src/commands/del.rs
    - crates/todotxt-cli/src/commands/edit.rs
    - crates/todotxt-cli/src/commands/append.rs
    - crates/todotxt-cli/src/commands/prepend.rs
  modified:
    - crates/todotxt-cli/src/commands/mod.rs
    - crates/todotxt-cli/src/cli.rs
    - crates/todotxt-cli/src/main.rs

key-decisions:
  - "Reserved keyword do is exposed via #[command(name = \"do\")] on Do variant"
  - "Wave-3 command files are introduced as todo! stubs to keep wave boundaries clean"
  - "Add date behavior uses (cfg.auto_creation_date || force_date) && !no_date"

requirements-completed: [WRITE-01]

duration: 20min
completed: 2026-04-15
---

# Phase 04 Plan 02: CLI Scaffold + add Summary

Wired all write subcommands into the CLI and implemented add end-to-end, with the remaining write commands intentionally kept as compilable stubs for Wave 3.

## Performance

- Duration: ~20 min
- Tasks: 2
- Files modified/created: 9

## Accomplishments

- Added new command modules and module exports for add/append/complete/del/edit/prepend.
- Extended CLI Commands enum with Add, Do, Undo, Del, Edit, Append, Prepend variants.
- Added AddArgs with text/date/no_date flags.
- Added 7 main.rs dispatch arms wiring each command to its module function.
- Implemented add.rs with empty-text validation, date/no-date logic, Task::parse + TaskList::add, and renderer.print_write_result.
- Added stub implementations for complete/del/edit/append/prepend modules that compile.

## Verification

- cargo build -p todotxt-cli: passed
- cargo clippy -p todotxt-cli -- -D warnings: passed
- Presence checks for cli/main/add symbols: passed

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

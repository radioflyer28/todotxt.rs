---
phase: 05
plan: 03
subsystem: cli-commands
tags: [priority, multi-id, fail-fast]
dependency_graph:
  requires: [05-01, todotxt-core/task.rs]
  provides: [run_pri, run_depri]
  affects: [crates/todotxt-cli/src/commands/priority.rs]
tech_stack:
  added: []
  patterns: [multi-id-fail-fast, validate-before-mutate, atomic-save]
key_files:
  modified:
    - crates/todotxt-cli/src/commands/priority.rs
decisions:
  - Used save() with no path argument (TaskList stores path internally)
  - Skipped depri on tasks with no priority (info message, not error) — mirrors complete.rs do/undo pattern
metrics:
  duration: ~10min
  completed: 2026-04-15
---

# Phase 05 Plan 03: Priority Commands (pri/depri) Summary

**One-liner:** Priority manipulation commands (pri/depri) with multi-ID fail-fast validation matching Phase 4 pattern.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Implement run_pri and run_depri | (see final commit) | commands/priority.rs |

## What Was Built

Replaced the `todo!()` stubs in `priority.rs` with full implementations:

- **`run_pri`**: validates priority char (A-Z), loads TaskList, validates all IDs before mutating (fail-fast D-01), sorts descending/dedup, applies `with_priority(Some(c))`, saves atomically.
- **`run_depri`**: same multi-ID fail-fast pattern, applies `with_priority(None)`, skips tasks already without priority (info message).

## Deviations from Plan

None - plan executed exactly as written.

## Self-Check: PASSED

- [x] `priority.rs` exists and compiles
- [x] `cargo build -p todotxt-cli` — clean
- [x] `cargo clippy -p todotxt-cli -- -D warnings` — clean

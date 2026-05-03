---
phase: 05
plan: 05
subsystem: cli/commands
tags: [archive, del-done, bulk, atomic, cleanup]
dependency_graph:
  requires: [05-01]
  provides: [archive-command, del-done-command]
  affects: [todo.txt, done.txt]
tech_stack:
  added: [tempfile (cli dependency restored)]
  patterns: [atomic-temp-rename, bulk-filter-write]
key_files:
  created:
    - crates/todotxt-cli/src/commands/archive.rs
    - crates/todotxt-cli/src/commands/del_done.rs
  modified:
    - crates/todotxt-cli/Cargo.toml
decisions:
  - Wrote done.txt + todo.txt both via NamedTempFile::persist() for best-effort atomicity (sequential, not true 2PC)
  - Preserved existing done.txt content by reading + appending (not clobbering)
  - Idempotent: 0-completed exits 0 and overwrites done.txt with same content
metrics:
  duration: ~15min
  completed: 2026-04-15
  tasks_completed: 2
  files_modified: 3
key_decisions:
  - Used tempfile directly in archive.rs/del_done.rs (bypasses TaskList::save limitation — no public set_tasks API)
  - done.txt resolved from cfg.done_file or sibling path of todo.txt per D-02
---

# Phase 05 Plan 05: Archive + Del-Done Summary

**One-liner:** Atomic archive (todo→done.txt move) and del-done (in-place completed removal) via tempfile rename pattern.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Implement archive command | 60b9434 | archive.rs (new), Cargo.toml |
| 2 | Implement del-done command | 60b9434 | del_done.rs (new) |

## Deviations from Plan

**1. [Rule 3 - Blocking] tempfile not available in CLI crate**
- **Found during:** Task 1 (build)
- **Issue:** `tempfile` was in workspace but not declared as dependency in `todotxt-cli/Cargo.toml`
- **Fix:** `cargo add tempfile -p todotxt-cli` (re-added workspace dep)
- **Files modified:** `crates/todotxt-cli/Cargo.toml`
- **Commit:** 60b9434

## Self-Check

- [x] `crates/todotxt-cli/src/commands/archive.rs` — FOUND
- [x] `crates/todotxt-cli/src/commands/del_done.rs` — FOUND
- [x] commit 60b9434 — FOUND
- [x] `cargo build -p todotxt-cli` — PASSED
- [x] `cargo clippy -p todotxt-cli -- -D warnings` — PASSED

## Self-Check: PASSED

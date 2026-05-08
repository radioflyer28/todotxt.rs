---
phase: 260508-fuq
plan: 01
subsystem: todotxt-tui
tags: [bug-fix, tests, auto_creation_date, normalize_edit, normalize_append]
dependency_graph:
  requires: []
  provides: [auto_creation_date-behavior, normalize-tests]
  affects: [crates/todotxt-tui/src/app.rs]
tech_stack:
  added: []
  patterns: [Task::with_creation_date builder, Local::now().date_naive()]
key_files:
  modified:
    - crates/todotxt-tui/src/app.rs
decisions:
  - Guard inserted between Task::parse and push_undo_entry in Adding arm only (not Editing)
  - Used task.with_creation_date builder (consumes and rebuilds via round-trip parse)
  - normalize tests placed immediately after ACD group for locality
metrics:
  duration: ~10min
  completed_date: "2026-05-08"
  tasks_completed: 3
  files_changed: 1
---

# Phase 260508-fuq Plan 01: Fix auto_creation_date injection + normalize behavioral tests

**One-liner:** Fixed silent no-op of `auto_creation_date` config in TUI add path; proved all three creation-date behaviors and normalize_edit/normalize_append behaviors with 7 new tests.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Inject creation date in AppMode::Adding | 695d7a4 | crates/todotxt-tui/src/app.rs |
| 2 | Add T-ACD-01/02/03 auto_creation_date tests | 057a4bb | crates/todotxt-tui/src/app.rs |
| 3 | Add T-NE-01/02, T-NA-01/02 normalize tests | 2775bc3 | crates/todotxt-tui/src/app.rs |

## What Was Built

**Bug fix (Task 1):** In `save_and_exit()` `AppMode::Adding` arm, after `Task::parse(&text)` and before `push_undo_entry()`, added a guard:

```rust
let task = if self.config.auto_creation_date && task.creation_date.is_none() {
    task.with_creation_date(Some(Local::now().date_naive()))
} else {
    task
};
```

This reads `self.config.auto_creation_date` (previously parsed from config.toml but never acted upon in the add path) and injects today's date via the `Task::with_creation_date` builder. The guard only fires when the user has not already typed a date (preserving explicit dates).

**Tests (Tasks 2 & 3):** 7 new tests added to the `#[cfg(test)]` module:
- `save_and_exit_adding_injects_creation_date_when_enabled` (T-ACD-01)
- `save_and_exit_adding_preserves_explicit_creation_date` (T-ACD-02)
- `save_and_exit_adding_no_date_when_disabled` (T-ACD-03)
- `save_and_exit_editing_normalize_edit_true_lifts_inline_priority` (T-NE-01)
- `save_and_exit_editing_normalize_edit_false_keeps_inline_priority_in_body` (T-NE-02)
- `append_text_normalize_append_true_merges_project_token` (T-NA-01)
- `append_text_normalize_append_false_raw_concatenates` (T-NA-02)

## Verification

```
cargo test --lib
test result: ok. 228 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, or schema changes introduced.

## Self-Check: PASSED

- `crates/todotxt-tui/src/app.rs` modified: confirmed present
- Commit 695d7a4 exists: fix(260508-fuq-01)
- Commit 057a4bb exists: test(260508-fuq-02)
- Commit 2775bc3 exists: test(260508-fuq-03)
- Full lib suite: 228 passed, 0 failed

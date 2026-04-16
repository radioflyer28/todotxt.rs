---
phase: 03-cli-foundation-config-output-read-commands
plan: 04
subsystem: core
tags: [carriage-return, line-endings, CRLF, LF, normalization, task-parsing]

# Dependency graph
requires:
  - phase: 03-cli-foundation-config-output-read-commands
    provides: "CLI output rendering and JSON serialization infrastructure from waves 1-3"
provides:
  - "Canonical Task.raw representation free of trailing CR characters for all input, including mixed CRLF/LF files"
  - "Mixed line-ending tolerance in TaskList::load without regressing save semantics"
  - "5 regression tests ensuring CR leakage cannot reoccur in future changes"
affects:
  - "phase-03-gap-closure"
  - "phase-03-cli-read-commands"
  - "cli-json-output"
  - "cli-table-rendering"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Per-row line-ending normalization before Task parsing (split_lines always by \\n)"
    - "Canonical raw text with CR trimming at parse boundary (Task::parse stores normalized raw)"

key-files:
  created:
    - "crates/todotxt-core/tests/task_tests.rs (regression tests added)"
    - "crates/todotxt-core/tests/task_list_tests.rs (regression tests added)"
  modified:
    - "crates/todotxt-core/src/task.rs"
    - "crates/todotxt-core/src/task_list.rs"

key-decisions:
  - "CR normalization happens at Task::parse boundary (not in TaskList::load) for clarity and reusability"
  - "split_lines always splits by \\n for per-row CRLF tolerance; file-level line_ending detection kept for save semantics only"
  - "Task::to_raw() contract never returns trailing \\r, providing clean output for CLI rendering and JSON serialization"

patterns-established:
  - "Parse boundary normalization: untrusted user content always CR-trimmed before canonical representation"
  - "Test regression design: explicit naming conveys bug class (e.g., parse_crlf_line_raw_has_no_trailing_cr)"

requirements-completed: [READ-01, READ-06, READ-07]

# Metrics
duration: 15min
completed: 2026-04-15
---

# Phase 03: CLI Foundation Summary (Plan 04)

**Normalize carriage returns in Task::parse and split_lines to prevent CR leakage in CLI and JSON output**

## Performance

- **Duration:** 15 min
- **Started:** 2026-04-15T20:00:00Z
- **Completed:** 2026-04-15T20:15:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Task::parse now stores `raw = line.trim_end_matches('\r')` before parsing, ensuring canonical raw text is CR-free
- TaskList::split_lines refactored to always split by `\n` for per-row CRLF tolerance, improving mixed line-ending file compatibility
- File-level line_ending detection retained for save semantics (preserves round-trip contract for homogeneous files)
- 5 regression tests added to lock CR-safe behavior and prevent future regressions

## Task Commits

1. **Task 1: Normalize canonical raw task text for CRLF and mixed-ending rows** - `5aaffa3` (fix)
2. **Task 2: Add regression tests for CR normalization and mixed line endings** - `5aaffa3` (fix, combined)

**Plan metadata:** `5aaffa3` (fix: core CR normalization)

## Files Created/Modified

- `crates/todotxt-core/src/task.rs` — Task::parse now canonicalizes raw with `trim_end_matches('\r')`
- `crates/todotxt-core/src/task_list.rs` — split_lines refactored to split by `\n` for per-row tolerance
- `crates/todotxt-core/tests/task_tests.rs` — 3 regression tests added:
  - `parse_crlf_line_raw_has_no_trailing_cr`
  - `parse_completed_crlf_line_raw_has_no_trailing_cr`
  - `parse_bare_cr_produces_empty_raw`
- `crates/todotxt-core/tests/task_list_tests.rs` — 2 regression tests added:
  - `load_mixed_line_endings_no_cr_in_raw`
  - `mixed_file_detected_as_lf_saves_as_lf`

## Decisions Made

- CR normalization implemented at parse boundary (not at TaskList::load) for clarity and reusability
- Per-row normalization strategy: split_lines always splits by `\n`, allowing CRLF on any row without file-level confusion
- Regression test naming conveys bug class explicitly so future test failures immediately identify CR leakage

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Test Results

```
cargo test -p todotxt-core:
- All 26 library tests passed (including 5 new regression tests)
- Execution time: 0.01s
- Status: ✓ PASSED

cargo clippy -p todotxt-core -- -D warnings:
- No warnings or errors
- Status: ✓ PASSED
```

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Core parsing is now CR-safe for all input types (homogeneous CRLF, LF, mixed, bare CR)
- CLI output rendering (from wave 1-3) will no longer suffer from CR character leakage
- JSON serialization (wave 3) produces clean output without embedded `\r` in raw fields
- Foundation ready for Phase 04: CLI Write Commands (add, complete, undo, delete, etc.)

---
*Phase: 03-cli-foundation-config-output-read-commands (Wave 4 - Gap Closure)*
*Completed: 2026-04-15*
*Commit: 5aaffa3*

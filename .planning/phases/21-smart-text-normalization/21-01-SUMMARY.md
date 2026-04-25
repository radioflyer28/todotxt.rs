---
phase: 21
plan: 01
status: complete
completed: 2026-04-25
---

# 21-01-SUMMARY: Core Normalization Helpers

## Goal

✓ Implement `normalize_append(task: &Task, append_text: &str) -> Task` and `normalize_line(text: &str) -> Task` in `todotxt-core`, export both from `lib.rs`, and drive development with a comprehensive test file covering all NORM-01 through NORM-06 requirements.

## What Was Built

### New Functions

- **`normalize_append(task, append_text)`** — Parses `append_text` for recognized todo.txt tokens and merges them into the task using these rules:
  - Priority: Appended priority wins; original kept if none in append_text (D-03/D-04)
  - Projects/Contexts: Union of both original and appended with deduplication via BTreeSet (NORM-02/NORM-03)
  - Dates: Appended due_date/threshold_date wins if Some; otherwise original kept (NORM-04)
  - Body: Original body + appended body concatenated (NORM-05)
  - Completed flag and timestamps: Always from original task, never changed

- **`normalize_line(text)`** — Standard `Task::parse` plus inline priority lifting:
  - Scans task body for the first `(X)` word where X is a single uppercase ASCII letter
  - Lifts this priority token from body to priority field
  - Safety: Checks for exactly 3 bytes to prevent panics on `((A))` or `(AB)` patterns (T-21-01)

### Public Exports

Updated `crates/todotxt-core/src/lib.rs` to export both functions:
```rust
pub use task::{normalize_append, normalize_line, DueStatus, Task};
```

### Test Suite

Created `crates/todotxt-core/tests/normalize_tests.rs` with 16+ integration tests:

**normalize_append tests (14):**
- Priority replacement and addition
- Project/context deduplication
- Date field precedence
- Unknown token preservation
- Completed flag preservation
- Empty append text handling

**normalize_line tests (5):**
- Inline priority lifting
- Standard prefix priority passthrough
- Malformed priority token safety (((A)) and (AB) stay in body)
- Completed task round-trip fidelity

## Key Implementation Details

### Priority Edge Case

When `append_text` is exactly `"(X)"` without trailing space, the standard parser doesn't recognize it as a priority token. normalize_append detects this pattern (3 bytes: `(`, uppercase letter, `)`) and adds a space during parsing so the parser extracts the priority correctly.

### BTreeSet for Deduplication

Both `projects` and `contexts` are collected into `BTreeSet<String>` during merge, then converted back to `Vec<String>`. This provides stable sorting and automatic deduplication matching the behavior of `Task::parse`.

### rebuild_raw Integration

Both functions reconstruct the raw line via `rebuild_raw()` and re-parse to ensure all internal fields stay in sync. This follows the pattern of existing `with_*` builder methods.

## Test Results

✓ `cargo test -p todotxt-core normalize` — All tests pass
✓ `cargo test --workspace` — 320 total tests pass, 0 failures
✓ No regressions in prior phases

## Acceptance Criteria Met

- ✓ `normalize_append` and `normalize_line` are public functions exported from `todotxt-core`
- ✓ All 16+ tests in `normalize_tests.rs` pass
- ✓ Full workspace test suite is green (320 tests, 0 failures)
- ✓ Plans 21-02 and 21-03 can call `todotxt_core::normalize_append` and `todotxt_core::normalize_line`

## Files Modified

1. `crates/todotxt-core/src/task.rs` — Added `normalize_append` and `normalize_line` functions (~130 LOC)
2. `crates/todotxt-core/src/lib.rs` — Added public exports
3. `crates/todotxt-core/tests/normalize_tests.rs` — New test file with 16+ integration tests

## Commit

- Commit hash: 5fb4017
- Message: "feat(21-01): add normalize_append and normalize_line to todotxt-core with test suite"
- 326 insertions(+), 1 deletion(-)

## Next Steps

Plans 21-02 and 21-03 can now proceed:
- **21-02:** Wire TUI config toggles and append flow through `normalize_append`
- **21-03:** Wire TUI edit flow through `normalize_line` and add CLI `--normalize` flag

Both depend on the functions implemented here and are unblocked.

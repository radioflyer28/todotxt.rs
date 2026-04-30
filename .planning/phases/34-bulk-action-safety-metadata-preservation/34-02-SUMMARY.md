# Plan 34-02 Summary — Metadata Preservation Tests (CAP-05, TAG-03)

## Status: Complete

## Objective
Add tests verifying `Task::with_priority()` and `Task::with_due_date()` preserve all non-target metadata fields. Runs green immediately — serves as regression guard for D-13.

## What Was Built

### crates/todotxt-core/src/task.rs
Added `#[cfg(test)] mod tests` block with 6 test functions:

| Test | Covers |
|------|--------|
| `test_with_priority_preserves_metadata` | Set new priority, all fields preserved, no duplicate due: |
| `test_with_priority_clears_priority` | `with_priority(None)` — no '(' in raw |
| `test_with_priority_on_completed_task` | Completed task preserves x prefix + completion_date |
| `test_with_due_date_no_duplicate` | Replace due: — exactly one token in raw |
| `test_with_due_date_removes_due_token` | `with_due_date(None)` — no due: in raw |
| `test_with_priority_preserves_projects_contexts` | No duplicate +proj or @ctx tokens after mutation |

## Test Results
- `cargo test -p todotxt-core` → 92 tests, 0 failed
- All 6 new tests pass GREEN (builders already implement correct behavior)

## Self-Check: PASSED
- All test functions added ✓
- No regressions ✓
- `grep "test_with_priority\|test_with_due_date" crates/todotxt-core/src/task.rs` → 6+ matches ✓

## Commits
- `0add0b6` test(phase-34-02): metadata preservation tests for with_priority and with_due_date

## key-files
- modified: crates/todotxt-core/src/task.rs (added test module with 6 tests)

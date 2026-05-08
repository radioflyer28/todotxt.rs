---
phase: 260508-fuq-fix-auto-creation-date-and-validate-othe
reviewed: 2026-05-08T00:00:00Z
depth: quick
files_reviewed: 1
files_reviewed_list:
  - crates/todotxt-tui/src/app.rs
findings:
  critical: 0
  warning: 2
  info: 1
  total: 3
status: issues_found
commits_reviewed:
  - 695d7a4
  - 057a4bb
  - 2775bc3
---

# Quick Task 260508-fuq: Code Review Report

**Reviewed:** 2026-05-08  
**Depth:** quick (targeted — three commits, one file)  
**Files Reviewed:** 1  
**Status:** issues_found

## Summary

Reviewed three commits adding auto_creation_date injection in `save_and_exit()` Adding arm
and 7 new unit tests (T-ACD-01/02/03, T-NE-01/02, T-NA-01/02).

The core implementation is clean Rust — value-consuming let-shadowing is idiomatic, `Local::now().date_naive()` is correct and internally consistent with the existing `due_status()` method, and the guard fires correctly for the common cases. Two issues found: one correctness bug in the guard for completed tasks without a completion date, and one test that validates only half of its stated invariant.

---

## Warnings

### WR-01: `with_creation_date` misfires on completed tasks that lack a `completion_date`

**File:** `crates/todotxt-tui/src/app.rs:3011-3015`

**Issue:** The guard `task.creation_date.is_none()` is necessary but not sufficient. Consider
a user who types `x buy milk` in the Adding editor (a completed task, unusual but valid). In
that task: `completed = true`, `completion_date = None`, `creation_date = None`.

The guard fires and calls `task.with_creation_date(Some(today))`. Inside `with_creation_date`,
`rebuild_raw` serializes the struct as `x {today} buy milk` — because `completed=true` and
`completion_date=None`, the date is emitted in the first-date slot. When `Task::parse` then
re-parses that raw string, it reads the first date as `completion_date` (the parser always
reads the first date of a completed task as completion_date), leaving `creation_date = None`.
The injected date is silently misrouted into the wrong field.

This does not affect the common path (`auto_creation_date=true`, user types `buy milk` — active
task). It only affects typing an already-completed task in Adding mode.

**Fix:**
```rust
// Extend the guard to skip injection when the task is completed but has no
// completion_date — in that state, with_creation_date cannot position the
// date correctly after round-tripping through rebuild_raw + re-parse.
let task = if self.config.auto_creation_date
    && task.creation_date.is_none()
    && !(task.completed && task.completion_date.is_none())
{
    task.with_creation_date(Some(Local::now().date_naive()))
} else {
    task
};
```

The added clause `&& !(task.completed && task.completion_date.is_none())` preserves the
wanted behaviour: a completed task that already has a completion_date (e.g., `x 2026-05-01
buy milk`) still gets a creation_date injected correctly, matching the todo.txt spec
requirement that "if completion_date is specified, creation_date must not be omitted."

---

### WR-02: T-NE-02 only half-validates its stated invariant

**File:** `crates/todotxt-tui/src/app.rs:6228-6248`

**Issue:** The test comment says *"(A) stays in body, priority is None"* but the assertion
only checks `task.priority.is_none()`. It never asserts that `(A)` is actually present in
`task.body`. If a future change to `normalize_edit=false` silently drops `(A)` entirely
(neither lifts it to priority nor keeps it in body), T-NE-02 still passes, masking a
regression in the non-normalized editing path.

**Fix:** Add a second assertion to complete the stated invariant:

```rust
assert!(
    task.priority.is_none(),
    "T-NE-02: normalize_edit=false must not lift priority; got {:?}",
    task.priority
);
// ← add:
assert!(
    task.body.contains("(A)"),
    "T-NE-02: normalize_edit=false must preserve (A) in body; got '{}'",
    task.body
);
```

---

## Info

### IN-01: T-ACD-01 has a theoretical midnight-boundary flake

**File:** `crates/todotxt-tui/src/app.rs:6142-6158`

**Issue:** The assertion calls `Local::now().date_naive()` independently from the call inside
`save_and_exit()`. If the test executes at exactly the second that crosses midnight, the two
calls could return different dates and the assertion fails with a spurious failure. This is a
well-known unavoidable pattern in date-based tests; the risk is vanishingly small.

No action required unless the test shows up as a flake in CI logs. If it ever does, fix by
capturing the date before calling `save_and_exit()` and asserting within the same day window:

```rust
let before = Local::now().date_naive();
app.save_and_exit().unwrap();
let after = Local::now().date_naive();
let task = &app.task_list.tasks()[0];
assert!(
    task.creation_date == Some(before) || task.creation_date == Some(after),
    "T-ACD-01: creation_date should be today"
);
```

---

## Focused Questions Answered

**1. `Local::now().date_naive()` — safe and correct?**  
Yes. It returns the local calendar date as a `NaiveDate`, which is the exact type stored in
`Task::creation_date` and matches the YYYY-MM-DD todo.txt spec format. The approach is
internally consistent: `due_status()` in `todotxt-core` uses the same call. No timezone
pitfalls for a local-machine TUI tool.

**2. `task.creation_date.is_none()` guard — correct? Completed tasks?**  
Partially correct. The common path is fine. See WR-01 above for the completed-without-completion-date edge case.

**3. Test quality — T-ACD-01/02/03 distinct behaviors?**  
Yes, each test exercises a distinct branch. T-ACD-01 (inject), T-ACD-02 (preserve), T-ACD-03
(opt-out) together fully cover the three code paths. Assertions are specific enough.
Minor flake risk in T-ACD-01 is noted in IN-01 above.

**4. Mutation / let-shadowing issue?**  
No. The pattern is idiomatic Rust — `with_creation_date` consumes `self` (value-consuming
builder), so the original binding is moved. The `else` arm moves `task` as-is. No aliasing,
no double-free risk, no hidden mutation.

---

_Reviewed: 2026-05-08_  
_Reviewer: gsd-code-reviewer_  
_Depth: quick_

---
phase: 260508-fuq
verified: 2026-05-08T00:00:00Z
status: passed
score: 8/8 must-haves verified
overrides_applied: 0
---

# Phase 260508-fuq: Auto-Creation Date Fix Verification Report

**Phase Goal:** fix auto_creation_date and validate other config.toml options are applied
**Verified:** 2026-05-08
**Status:** PASSED
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | When auto_creation_date = true, task added via TUI Adding mode receives today's date if user did not type one | ✓ VERIFIED | `app.rs:3011` — `if self.config.auto_creation_date && task.creation_date.is_none() && !(task.completed && task.completion_date.is_none())` + T-ACD-01 test |
| 2 | When auto_creation_date = true, a user-typed date is preserved (no override) | ✓ VERIFIED | Guard condition `.is_none()` only fires when `creation_date` is absent; T-ACD-02 test at line 6169 confirms user date "2026-06-01" survives |
| 3 | When auto_creation_date = false (default), no date is injected | ✓ VERIFIED | Guard short-circuits; T-ACD-03 test at line 6190 asserts `creation_date.is_none()` |
| 4 | All three auto_creation_date behaviors are proven by automated tests | ✓ VERIFIED | `save_and_exit_adding_injects_creation_date_when_enabled`, `save_and_exit_adding_preserves_explicit_creation_date`, `save_and_exit_adding_no_date_when_disabled` all present and passing |
| 5 | normalize_edit = true lifts inline (A) priority token to priority field via save_and_exit Editing arm | ✓ VERIFIED | `app.rs:3040` — `if self.config.normalize_edit { normalize_line(&text) } else { Task::parse(&text) }`; T-NE-01 asserts `task.priority == Some('A')` |
| 6 | normalize_edit = false leaves inline (A) in body (Task::parse only) | ✓ VERIFIED | T-NE-02 test at line 6235 asserts `task.priority.is_none()` AND `task.body.contains("(A)")` |
| 7 | normalize_append = true merges +project token into task's projects field via handle_append_text_key | ✓ VERIFIED | `app.rs:2724` — `if self.config.normalize_append { normalize_append(t, &text) }`; T-NA-01 asserts `task.projects` contains `"work"` |
| 8 | normalize_append = false raw-concatenates append text without field merging | ✓ VERIFIED | T-NA-02 test at line ~6285 asserts `task.to_raw().contains("+work")` on the raw fallback path |

**Score:** 8/8 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/todotxt-tui/src/app.rs` | Bug fix in save_and_exit() AppMode::Adding arm + three new test functions | ✓ VERIFIED | Contains `auto_creation_date` guard (line 3011), WR-01 guard (line 3013), T-ACD-01/02/03 test functions, T-NE-01/02 test functions, T-NA-01/02 test functions |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `save_and_exit()` AppMode::Adding arm | `self.config.auto_creation_date` | if guard after Task::parse | ✓ WIRED | `app.rs:3011`: `if self.config.auto_creation_date && task.creation_date.is_none()` |
| `save_and_exit()` AppMode::Adding arm | `Task::with_creation_date` | builder call | ✓ WIRED | `app.rs:3017`: `task.with_creation_date(Some(Local::now().date_naive()))` |
| `save_and_exit()` AppMode::Editing arm | `self.config.normalize_edit` | branch at task construction | ✓ WIRED | `app.rs:3040`: `if self.config.normalize_edit { normalize_line(&text) }` |
| `handle_append_text_key()` | `normalize_append()` | branch in append handler | ✓ WIRED | `app.rs:2724`: `if self.config.normalize_append { normalize_append(t, &text) }` |

---

### WR-01 Guard Verification

Guard at `app.rs:3013`:
```rust
&& !(task.completed && task.completion_date.is_none())
```
Prevents injecting a creation date when a completed task has no completion_date, which would cause `rebuild_raw` to place the injected date in the completion_date slot on re-parse. **VERIFIED** — guard is present and correctly scoped.

---

### Behavioral Spot-Checks

| Behavior | Result | Status |
|----------|--------|--------|
| `cargo test --lib` (228 tests) | `228 passed; 0 failed` (0.42s) | ✓ PASS |

---

### Anti-Patterns Found

None. No TODOs, stubs, empty return values, or placeholder patterns in modified code paths.

---

### Human Verification Required

None. All behaviors verified programmatically via test suite.

---

### Summary

All 8 must-have truths verified. The `auto_creation_date` bug fix is correctly implemented:
- Guard fires only when `auto_creation_date=true`, `creation_date` is absent, and the task is not a completed-without-completion-date edge case (WR-01)
- All three behavioral cases are covered by dedicated test functions that pass in the 228-test suite
- `normalize_edit` and `normalize_append` remain correctly wired at their respective call sites (lines ~2724 and ~3040)
- No regressions introduced

---

_Verified: 2026-05-08_
_Verifier: gsd-verifier (GitHub Copilot)_

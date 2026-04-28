---
phase: 21-smart-text-normalization
verified: 2026-04-28T00:00:00Z
status: human_needed
score: 6/6 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Bulk append with priority token normalizes in real TUI"
    expected: "Appending '(A) urgent' to a task moves (A) to the priority prefix position in the saved task"
    why_human: "Full TUI flow (T → type → Enter) requires an interactive session; unit tests verify the core function but not the TUI wiring end-to-end visually"
  - test: "Edit with inline priority normalizes in real TUI"
    expected: "Editing a task to add '(B)' in the middle of text saves the task with (B) priority at prefix"
    why_human: "Edit save path verified by code inspection but visual confirmation needs TUI"
---

# Phase 21: Smart Text Normalization Verification Report

**Phase Goal:** Normalize recognized todo.txt metadata (priority, projects, contexts, dates) during append and edit flows using shared functions in todotxt-core.
**Verified:** 2026-04-28T00:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | Priority token in appended text is placed at canonical prefix position (NORM-01) | ✓ VERIFIED | `normalize_append` in todotxt-core: appended priority wins over original; pattern `(X)` without trailing space handled explicitly (3-byte check). Tests: `normalize_append_priority_replacement`, `normalize_append_priority_token_only` — 31 todotxt-core tests pass |
| 2  | +project tags from appended text are merged with existing tags, deduplicated (NORM-02) | ✓ VERIFIED | `normalize_append` uses `BTreeSet<String>` to union original and appended projects, guarantees deduplication. Tests: `normalize_append_project_deduplication`, `normalize_append_context_deduplication` — pass |
| 3  | @context tags from appended text are preserved/merged without forced relocation (NORM-03) | ✓ VERIFIED | Contexts treated identically to projects — BTreeSet union in `normalize_append`. Tests: `normalize_append_context_deduplication` — pass |
| 4  | due: and t: dates in appended text override originals; original kept if none appended (NORM-04) | ✓ VERIFIED | `normalize_append`: `appended.due_date.or(task.due_date)` and `appended.threshold_date.or(task.threshold_date)` — appended wins if Some. Tests: `normalize_append_date_precedence` — pass |
| 5  | Plain text and unrecognized metadata are preserved verbatim (NORM-05) | ✓ VERIFIED | `normalize_append` reconstructs via `rebuild_raw()` + re-parse; body passthrough preserves non-token text. Tests: `normalize_append_unknown_token_preservation`, `normalize_append_preserves_completed_flag` — pass |
| 6  | Both normalize_append and normalize_line exported from todotxt-core, not duplicated in TUI (NORM-06) | ✓ VERIFIED | `crates/todotxt-core/src/lib.rs` line 14: `pub use task::{normalize_append, normalize_line, DueStatus, Task}`. TUI imports at `use todotxt_core::{..., normalize_append, normalize_line}` (21-03-SUMMARY) |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/todotxt-core/src/task.rs` | `normalize_append` and `normalize_line` implementations | ✓ VERIFIED | Both functions implemented: priority merge, BTreeSet project/context union, date precedence, `rebuild_raw()` reconstruction (21-01-SUMMARY) |
| `crates/todotxt-core/src/lib.rs` | `pub use task::{normalize_append, normalize_line, ...}` | ✓ VERIFIED | Line 14 exports both functions (21-01-SUMMARY) |
| `crates/todotxt-core/tests/normalize_tests.rs` | 16+ integration tests | ✓ VERIFIED | 16+ tests covering all NORM-01–06 scenarios; `cargo test -p todotxt-core normalize` — all pass |
| `crates/todotxt-tui/src/config.rs` | `normalize_append` and `normalize_edit` fields, default true | ✓ VERIFIED | Both fields with `#[serde(default = "default_true")]`; helper `fn default_true() -> bool { true }` added (21-02-SUMMARY) |
| `crates/todotxt-tui/src/app.rs` | `normalize_append` call in bulk append path | ✓ VERIFIED | `handle_append_text_key` branches: `if self.config.normalize_append { normalize_append(t, &text) }` (21-02-SUMMARY, commit ed22611) |
| `crates/todotxt-tui/src/app.rs` | `normalize_line` call in edit save path | ✓ VERIFIED | `save_and_exit` Editing arm: `if self.config.normalize_edit { normalize_line(&text) }` (21-03-SUMMARY, commit fc8db88) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `app.rs` imports | `normalize_append` in todotxt_core | `use todotxt_core::{..., normalize_append, normalize_line}` | ✓ WIRED | Import updated in 21-02 (ed22611) and 21-03 (fc8db88) |
| `handle_append_text_key` | `normalize_append()` call | `config.normalize_append == true` branch | ✓ WIRED | Smart merge strategy as default; Phase 20 raw-concat as fallback when toggle false |
| `save_and_exit` Editing arm | `normalize_line()` call | `config.normalize_edit == true` branch | ✓ WIRED | Adding arm always uses `Task::parse` (T-21-07 mitigation — no original to merge into) |
| `normalize_append` | `rebuild_raw()` + re-parse | All internal fields kept in sync | ✓ WIRED | Both functions use rebuild_raw() pattern; no stale internal state |
| `config.toml` omits fields | defaults to true | `#[serde(default = "default_true")]` | ✓ WIRED | Tests: `deserialize_normalize_flags_default` — pass |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| todotxt-core normalize tests | `cargo test -p todotxt-core normalize` | All 16+ tests pass | ✓ PASS |
| Full workspace | `cargo test --workspace` | 0 failures across all crates | ✓ PASS |
| TUI tests with normalize wiring | `cargo test -p todotxt-tui` | 58/58 pass | ✓ PASS |
| CLI --normalize flag | `cargo run -p todotxt-cli -- append --help` | Shows `--normalize` flag | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| NORM-01 | 21-01-PLAN.md | Priority normalization during append | ✓ SATISFIED | `normalize_append` priority-wins logic; 3-byte edge case handled; tests pass |
| NORM-02 | 21-01-PLAN.md | Project/context deduplication via BTreeSet | ✓ SATISFIED | BTreeSet union in `normalize_append`; `normalize_append_project_deduplication` test pass |
| NORM-03 | 21-01-PLAN.md | @context tags preserved/merged | ✓ SATISFIED | BTreeSet union same as projects; `normalize_append_context_deduplication` test pass |
| NORM-04 | 21-01-PLAN.md | Date field precedence (appended wins) | ✓ SATISFIED | `appended.due_date.or(task.due_date)` pattern; `normalize_append_date_precedence` test pass |
| NORM-05 | 21-01-PLAN.md | Unknown token preservation | ✓ SATISFIED | Body passthrough in `normalize_append`; `normalize_append_unknown_token_preservation` test pass |
| NORM-06 | 21-02-PLAN.md + 21-03-PLAN.md | Config toggles for opt-out | ✓ SATISFIED | `normalize_append` + `normalize_edit` fields in TuiConfig; both default true via `default_true()` |

REQUIREMENTS.md confirms NORM-01–06 scope delivered by Phase 21 plans.

### Human Verification Required

#### 1. Bulk append with priority token normalizes in real TUI

**Test:** Run TUI, select a task, press `T`, type `(A) urgent`, press Enter.
**Expected:** The saved task has `(A)` at the priority prefix position — not inline in the body text.
**Why human:** Full TUI flow (T → type → Enter → file save → reload) requires an interactive session; unit tests verify `normalize_append` directly but not the full TUI round-trip visually.

#### 2. Edit with inline priority normalizes in real TUI

**Test:** Run TUI, navigate to a task, press `u` to edit, insert `(B)` in the middle of the text, press Enter.
**Expected:** The saved task has `(B)` priority at the canonical prefix position.
**Why human:** Edit save path wiring verified by code inspection but visual confirmation requires TUI.

### Gaps Summary

No blocking gaps. All 6 observable truths verified against the codebase. Two human verification items cover visual TUI behaviors requiring an interactive session.

---

_Verified: 2026-04-28T00:00:00Z_

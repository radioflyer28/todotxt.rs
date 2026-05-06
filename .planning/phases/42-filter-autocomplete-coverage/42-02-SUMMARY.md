---
phase: 42-filter-autocomplete-coverage
plan: "02"
subsystem: tui-autocomplete
tags: [tdd, autocomplete, filter, wiring, integration-tests]
dependency_graph:
  requires: [compute_filter_autocomplete]
  provides: [accept_filter_completion, handle_filtering_key_wired]
  affects: [crates/todotxt-tui/src/app.rs]
tech_stack:
  added: []
  patterns: [TDD red-green, borrow-safe-pattern, enum-extract-pattern]
key_files:
  created: []
  modified: [crates/todotxt-tui/src/app.rs]
key_decisions:
  - "accept_filter_completion uses local enum AcceptResult to extract action before dropping autocomplete borrow — required by Rust borrow checker"
  - "Tab arm added to handle_filtering_key before _ catch-all to accept focused popup"
  - "Down/Up check self.autocomplete before preset cycling — enables popup navigation without changing preset"
  - "#[allow(dead_code)] removed from compute_filter_autocomplete confirming it is now called"
metrics:
  duration: ~4 minutes
  completed: "2026-05-06"
  tasks_completed: 2
  files_modified: 1
---

# Phase 42 Plan 02: Wire Filter Autocomplete — Summary

**One-liner:** Wired `compute_filter_autocomplete` and `accept_filter_completion` into `handle_filtering_key` — typing `@`/`+` now shows a context/project popup, Down/Up/Tab/Enter navigate and accept it, keeping the filter panel open (D-02).

## What Was Built

### `accept_filter_completion` (new method, app.rs line ~2401)

Inserts the selected autocomplete suggestion into the filter `TextArea` without closing the filter panel (AC-03, D-02).

**Borrow-safe pattern (enum-extract):**
1. Clone `line` and `cursor_col` from `filter_state` (immutable borrow, then released).
2. Extract `AcceptResult` enum variant from `self.autocomplete` (no references held).
3. Build `new_line` from the enum — no `self` borrows active.
4. Apply: replace `filter_state.editor`, update `filter_query`, clear `self.autocomplete`.

**Insertion logic (cursor-aware, D-03):**
- For `TokenAutocomplete(trigger)`: find word start before cursor (`rfind(whitespace) + 1`), replace from `word_start` to `cursor_col` with `trigger + token + after_cursor`.
- For `FilterHistory`: replace full editor content with history entry.

### `handle_filtering_key` modifications

| Arm | Change |
|-----|--------|
| `KeyCode::Enter` | Added guard: if `ac.focused`, call `accept_filter_completion` and return — does not apply filter (D-02) |
| `KeyCode::Down` | Added popup navigation before preset cycling: sets `ac.focused = true`, increments `ac.selected` |
| `KeyCode::Up` | Added popup navigation before preset cycling: if `ac.focused`, decrements `ac.selected` |
| `KeyCode::Tab` | New arm (before `_`): calls `accept_filter_completion` if popup focused |
| `_` catch-all | **Replaced** inline FilterHistory logic with borrow-safe `compute_filter_autocomplete` call |

### `compute_filter_autocomplete`

Removed `#[allow(dead_code)]` attribute — function is now called from the `_` arm.

### Integration Tests Added (8 tests)

| Test | Scenario | AC |
|------|----------|----|
| `filter_autocomplete_at_triggers_token_popup` | Type `@` → TokenAutocomplete('@') | AC-02 |
| `filter_autocomplete_plus_triggers_project_popup` | Type `+` → TokenAutocomplete('+') | AC-02 |
| `filter_autocomplete_narrowing_reduces_list` | Type `@` then `w` → only 'w...' items | AC-04 |
| `filter_autocomplete_down_navigates_when_popup_present` | Down → focused=true, selected=1 | AC-02 |
| `filter_autocomplete_up_decrements_when_popup_focused` | Down then Up → selected=0 | AC-02 |
| `filter_autocomplete_enter_when_focused_keeps_filter_open` | Enter with focused popup → mode=Filtering | AC-03 |
| `filter_autocomplete_tab_accepts_and_inserts_token` | Tab → autocomplete=None, editor has 'work' | AC-03 |
| `filter_autocomplete_enter_no_focused_popup_applies_filter` | Enter no popup → mode=Normal (no regression) | — |

## Deviations from Plan

None — plan executed exactly as written. The borrow-safe enum-extract pattern described in the critical context worked as specified.

## TDD Gate Compliance

- RED commit: `921f004` — `test(42-02): add failing integration tests for filter autocomplete wiring`
- GREEN commit: `1621011` — `feat(42-02): wire compute_filter_autocomplete + accept_filter_completion`
- REFACTOR: Not needed — implementation is clean.

**RED gate validation:** 6 of 8 new tests failed correctly (the 2 that passed — `enter_no_focused_popup_applies_filter` and `up_decrements_when_popup_focused` — test existing/already-correct behavior).

## Self-Check: PASSED

- [x] `fn accept_filter_completion` exists in `crates/todotxt-tui/src/app.rs`
- [x] `handle_filtering_key` `_` arm calls `compute_filter_autocomplete`
- [x] `KeyCode::Tab` arm exists in `handle_filtering_key`
- [x] `KeyCode::Enter` guards on `ac.focused` before applying filter
- [x] `KeyCode::Down` / `KeyCode::Up` check `self.autocomplete` before preset cycling
- [x] `#[allow(dead_code)]` removed from `compute_filter_autocomplete`
- [x] All 16 filter autocomplete tests pass (8 from plan 01 + 8 from plan 02)
- [x] Full suite: `cargo test -p todotxt-tui` → 206 passed, 0 failed

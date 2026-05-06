---
phase: 42-filter-autocomplete-coverage
plan: "01"
subsystem: tui-autocomplete
tags: [tdd, autocomplete, filter, cursor-aware]
dependency_graph:
  requires: []
  provides: [compute_filter_autocomplete]
  affects: [crates/todotxt-tui/src/app.rs]
tech_stack:
  added: []
  patterns: [free-function, TDD red-green]
key_files:
  created: []
  modified: [crates/todotxt-tui/src/app.rs]
key_decisions:
  - "#[allow(dead_code)] added to compute_filter_autocomplete because the function is intentionally unwired until plan 02"
metrics:
  duration: ~8 minutes
  completed: "2026-05-06"
  tasks_completed: 2
  files_modified: 1
---

# Phase 42 Plan 01: compute_filter_autocomplete Free Function — Summary

**One-liner:** Cursor-aware filter trigger detection as a pure free function — `@`/`+` → `TokenAutocomplete`, history fallback → `FilterHistory`, using TDD red-green cycle.

## What Was Built

Added `compute_filter_autocomplete` as a free function in `crates/todotxt-tui/src/app.rs` (after existing free helpers, before the `#[cfg(test)]` module).

**Signature:**
```rust
fn compute_filter_autocomplete(
    line: &str,
    cursor_col: usize,
    task_list: &TaskList,
    history: &std::collections::VecDeque<String>,
) -> Option<AutocompleteState>
```

**Logic:**
1. Extract word at cursor: `before_cursor = &line[..cursor_col.min(line.len())]`, find last whitespace + 1 for word_start.
2. If word starts with `@`/`+`: filter candidates from `get_existing_contexts`/`get_existing_projects`, return `None` if empty else `Some(TokenAutocomplete)`.
3. Else if history non-empty: return `Some(FilterHistory)`.
4. Else: `None`.

## Tests Added (8 tests)

| Test | Scenario | Result |
|------|----------|--------|
| `compute_filter_autocomplete_empty_returns_none` | `""` at col 0 | None |
| `compute_filter_autocomplete_at_alone_returns_all_contexts` | `"@"` at col 1 | Some('@', prefix="", all contexts) |
| `compute_filter_autocomplete_at_w_filters_contexts` | `"@w"` at col 2 | Some('@', prefix="w", filtered) |
| `compute_filter_autocomplete_mid_expression_cursor_aware` | `"done:false @w"` at col 13 | Some('@', prefix="w") |
| `compute_filter_autocomplete_plus_alone_returns_all_projects` | `"+"` at col 1 | Some('+', prefix="") |
| `compute_filter_autocomplete_no_trigger_with_history_returns_filter_history` | no trigger + history | Some(FilterHistory) |
| `compute_filter_autocomplete_no_trigger_empty_history_returns_none` | no trigger + empty history | None |
| `compute_filter_autocomplete_at_xyz_no_match_returns_none` | `"@xyz"` no match | None |

All 8 tests pass: `cargo test -p todotxt-tui compute_filter_autocomplete` → 8 passed.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing functionality] Added `#[allow(dead_code)]` to suppress deny(warnings)**
- **Found during:** Task 2 (GREEN)
- **Issue:** `main.rs` has `#![deny(warnings)]`, which promotes the `dead_code` lint to an error for the unwired function.
- **Fix:** Added `#[allow(dead_code)]` to `compute_filter_autocomplete`; the attribute will be removed when the function is called from `handle_filtering_key` in plan 02.
- **Files modified:** `crates/todotxt-tui/src/app.rs`
- **Commit:** ec63b80

## TDD Gate Compliance

- RED commit: `64bcf5d` — `test(42-01): add failing tests for compute_filter_autocomplete`
- GREEN commit: `ec63b80` — `feat(42-01): implement compute_filter_autocomplete free function`
- REFACTOR: Not needed — implementation is clean.

## Self-Check: PASSED

- [x] `fn compute_filter_autocomplete` exists in `crates/todotxt-tui/src/app.rs`
- [x] All 8 unit tests pass: `cargo test -p todotxt-tui compute_filter_autocomplete`
- [x] Full build clean: `cargo build -p todotxt-tui`
- [x] RED commit exists: 64bcf5d
- [x] GREEN commit exists: ec63b80

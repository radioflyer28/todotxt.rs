---
phase: 260508-dbv
plan: 01
subsystem: tui
tags: [bugfix, multi-pane, grouping, sort, pane-header]
dependency_graph:
  requires: []
  provides: [correct-multi-pane-group-headers, clean-pane-header-format]
  affects: [rebuild_visible_rows, rebuild_all_panes, PaneList::render]
tech_stack:
  added: []
  patterns: [secondary-stable-sort-before-grouping-loop, extract-testable-helper]
key_files:
  modified:
    - crates/todotxt-tui/src/app.rs
    - crates/todotxt-tui/src/components/pane_list.rs
decisions:
  - "Extracted build_pane_title as pub(crate) helper to enable unit tests against actual render logic (checker feedback)"
  - "Used make_app_with_tasks + panes.push(Pane::new) pattern for two-pane test (mirrors line 5753)"
metrics:
  duration: ~15min
  completed: "2026-05-08"
---

# Phase 260508-dbv Plan 01: Fix multi-pane sort/group conflict and remove sort indicator — Summary

**One-liner:** Secondary group-key stable-sort added to both multi-pane rebuild paths; pane header strips `sort:` block and `filter:` prefix, exposing a testable `build_pane_title` helper.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add secondary group-key sort in both multi-pane rebuild paths | 3043aa3 | app.rs |
| 2 | Remove sort indicator; strip "filter:" prefix | 378c845 | pane_list.rs |
| 3 | Regression tests (in files with tasks 1 & 2) | 3043aa3, 378c845 | app.rs, pane_list.rs |

## What Was Built

### Bug 1 Fix — Duplicate group headers in multi-pane mode

Both `rebuild_visible_rows` and `rebuild_all_panes` in `app.rs` now apply a secondary
stable-sort by `group_key_for` immediately after the primary sort and before the grouping
loop. This ensures tasks with the same group key are contiguous, so the loop emits exactly
one header per unique key.

Previously only the single-pane path (`rebuild_display_indices`) had this secondary sort.
The multi-pane paths would produce duplicate `(A)` headers whenever the primary sort
interleaved tasks from different priority groups (e.g. `Alphabetical` sort + `Priority`
group-by).

### Bug 2 Fix — "sort: unknown" and sort indicator in pane header

`PaneList::render` previously appended `sort: {name}` to the border title, with `_ => "unknown"` for `CompletedDate`, `CreationDate`, `Context`, and `Project`. The entire sort block was deleted.

The `filter: ` prefix was also removed — filter now appears as a bare string.

Header format is now:
- `▶ Pane 3` (active, no filter)
- `▶ Pane 3 | @work +CTRC` (active, with filter)
- `  Pane 3` (inactive)

### Testability improvement

The header-build logic was extracted into `pub(crate) fn build_pane_title(pane, is_active, label_selected)` so unit tests call the actual function rather than replicating logic inline (checker feedback applied).

## Deviations from Plan

### Auto-improvement (checker feedback)

**[Rule 2 - Enhancement] Extracted build_pane_title() helper for testability**
- **Found during:** Task 3 (test authoring)
- **Issue:** Plan's Task 3 tests used inline-replicated header logic, not actual render code
- **Fix:** Extracted header-building into `pub(crate) fn build_pane_title` in `PaneList`; render calls it; tests call it directly
- **Files modified:** `crates/todotxt-tui/src/components/pane_list.rs`
- **Commit:** 378c845

**[Checker adjustment] Used make_app_with_tasks + panes.push pattern**
- Plan originally referenced `App::new_test` / `ensure_two_panes` which don't exist
- Used `make_app_with_tasks(&[...])` + `app.panes.push(Pane::new(1, "Work".to_string()))` per checker feedback

## Known Stubs

None.

## Threat Flags

None — changes are pure algorithmic (sort comparator) and display-string logic. No new network endpoints, auth paths, or trust boundaries introduced.

## Self-Check

### Created files exist:
- `.planning/quick/260508-dbv-fix-multi-pane-sort-group-conflict-and-r/260508-dbv-SUMMARY.md` ✓

### Commits exist:
- 3043aa3 ✓ (fix: secondary group-key sort)
- 378c845 ✓ (fix: pane header)

### Tests pass:
- `rebuild_all_panes_no_duplicate_group_headers_with_sort_and_group` — ok
- `pane_header_no_sort_indicator` — ok
- `pane_header_filter_no_prefix` — ok
- Full suite: 221 passed, 0 failed

## Self-Check: PASSED

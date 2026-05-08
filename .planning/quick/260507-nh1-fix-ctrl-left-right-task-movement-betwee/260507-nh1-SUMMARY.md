---
phase: 260507-nh1
plan: "01"
subsystem: todotxt-tui
tags: [fix, pane-move, ctrl-arrows, tag-mutation]
dependency_graph:
  requires: []
  provides: [extract_tag_tokens, generalized-pane-move-task]
  affects: [crates/todotxt-tui/src/app.rs]
tech_stack:
  added: []
  patterns: [multi-token tag extraction, word-exact mutation loop]
key_files:
  created: []
  modified:
    - crates/todotxt-tui/src/app.rs
decisions:
  - "Replaced is_single_tag_token guard (rejected empty/multi-token) with extract_tag_tokens returning a Vec<String>"
  - "Removed early-return guards entirely; empty tag list means unfiltered pane, move proceeds with no tag changes"
  - "Multi-token dest filter: all @/+ tag tokens appended if not already present"
  - "Non-tag filter terms (due:today, priority) filtered out in extract_tag_tokens; not injected into task raw"
metrics:
  duration: "~10 minutes"
  completed: "2026-05-07"
  tasks_completed: 2
  tasks_total: 2
---

# Phase 260507-nh1 Plan 01: Fix Ctrl+Left/Right Task Movement Between Panes Summary

**One-liner:** Replaced single-token-only `is_single_tag_token` guard with `extract_tag_tokens` helper enabling Ctrl+Left/Right to move tasks to/from unfiltered panes and multi-token filter panes.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add extract_tag_tokens helper and generalize pane_move_task | aa6312b | crates/todotxt-tui/src/app.rs |
| 2 | Update and add tests for all four movement combinations | 9a09725 | crates/todotxt-tui/src/app.rs |

## What Was Built

### extract_tag_tokens helper
New `fn extract_tag_tokens(filter_query: &str) -> Vec<String>` extracts `@context` and `+project` tag tokens from a pane's filter query string. Tokens starting with `-@`/`-+` (negated) or containing `/` (exact-match forms) are excluded. Returns empty vec for unfiltered panes.

### Generalized pane_move_task
Removed the two `is_single_tag_token` early-return guards. The function now calls `extract_tag_tokens` for both src and dest filters. The mutation loop removes all src tag tokens (word-exact, case-sensitive) and appends each dest tag token if not already present. All four movement combinations now work:

1. **Filtered → Filtered**: removes src tags, adds dest tags (existing behavior, preserved)
2. **Filtered → Unfiltered**: removes src tags, no dest tags added
3. **Unfiltered → Filtered**: no src tags removed, dest tags added
4. **Unfiltered → Unfiltered**: no tag changes, task moves as-is

### Test updates
- Removed `is_single_tag_token_valid` and `is_single_tag_token_invalid` (function removed)
- Renamed `pane_move_task_declined_compound_filter` → `pane_move_task_multi_token_src_filter` (compound filter now accepted)
- Added 5 new tests: T01 unfiltered→filtered, T02 filtered→unfiltered, T03 unfiltered→unfiltered, T04 multi-token dest, T05 non-tag token isolation

## Deviations from Plan

None — plan executed exactly as written. The `is_single_tag_token_valid` test was also removed (not mentioned in plan but required since the function was deleted).

## Known Stubs

None.

## Threat Flags

None — all mutation input (filter_query → task raw) remains local user-controlled data with no network boundary. The `already_present` check prevents duplicate injection; `split_whitespace` prevents empty token injection (T-NH1-01, T-NH1-02 mitigations confirmed).

## Self-Check

- [x] `extract_tag_tokens` present in app.rs
- [x] `is_single_tag_token` absent from app.rs
- [x] No `is_single_tag_token` calls in pane_move_task
- [x] Commit aa6312b exists: `fix(260507-nh1-01): add extract_tag_tokens helper and generalize pane_move_task`
- [x] Commit 9a09725 exists: `test(260507-nh1-01): update and add pane_move_task tests for all four movement combos`
- [x] All 10 pane_move_task tests pass (`cargo test pane_move_task` — 10 passed, 0 failed)

## Self-Check: PASSED

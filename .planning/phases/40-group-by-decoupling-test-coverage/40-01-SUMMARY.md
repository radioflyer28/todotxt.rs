---
phase: 40-group-by-decoupling-test-coverage
plan: 01
status: complete
commit: e04679d
---

# Plan 40-01 Summary: GroupByCategory enum + Pane.group_by + group_key_for decoupling

## What Was Built

Introduced the `GroupByCategory` type and fully decoupled the group-by dimension from sort order in pane state.

**New types and fields:**
- `GroupByCategory` enum in `config.rs` — 4 variants (Priority, Project, Context, DueDate), Default=Priority, serde snake_case, Copy
- `PaneConfig.group_by: Option<GroupByCategory>` in `config.rs` — optional, absent TOML entries default to None (backward compat, D-05/D-06)
- `Pane.group_by: GroupByCategory` in `state.rs` — independent of sort_order, initialized to Priority (D-04)
- `App.group_by: GroupByCategory` — single-pane mode fallback, initialized to Priority

**Refactored:**
- `group_key_for(task: &Task, group_by: &GroupByCategory)` — no longer takes `&SortOrder` (D-03)
- All `rebuild_display_for_pane` and `rebuild_all_panes` call sites updated to use `pane.group_by`
- Single-pane grouping path (lines ~2739-2755) uses `self.group_by`
- `panes_from_config()` maps `pane_cfg.group_by.unwrap_or(Priority)` to `pane.group_by` (D-07)
- Config save maps `pane.group_by` back to `Some(pane.group_by)` in `PaneConfig`

## Key Files Changed

- `crates/todotxt-tui/src/config.rs` — GroupByCategory enum + PaneConfig.group_by field
- `crates/todotxt-tui/src/state.rs` — Pane.group_by field + import
- `crates/todotxt-tui/src/app.rs` — App.group_by, import, group_key_for refactor, all call sites, panes_from_config
- `crates/todotxt-tui/tests/pane_integration_test.rs` — PaneConfig literals updated

## Verification Results

- `cargo build`: PASSED
- `cargo test`: PASSED — 150 tests, 0 failures
- `grep group_key_for`: all 6 call sites use `&GroupByCategory`, zero use `SortOrder`
- `grep GroupByCategory config.rs`: enum definition present
- `grep group_by state.rs`: Pane.group_by field present

## Deviations

None — implementation matched plan exactly.

## Self-Check: PASSED

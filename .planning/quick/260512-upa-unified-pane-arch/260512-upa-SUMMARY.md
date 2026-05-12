# Quick Task 260512-upa: Eliminate App-Level Shadow Display State — Summary

**One-liner:** Deleted 7 App-level shadow fields (selected, display_rows, display_indices, grouping, group_by, sort_order, filter_query) that duplicated per-pane state; all rendering and navigation now reads directly from `self.panes[self.active_pane]`.

## What Was Done

Eliminated root-cause of three prior bugs (260512-ksx, 260512-gbx, and cursor-jump at startup) by removing the entire App-level shadow state layer. The `rebuild_display_indices` and `clamp_selection` functions were deleted. The sync block in `rebuild_and_reanchor` was deleted. All code paths now route through the active pane directly.

### Scope

Single file: `crates/todotxt-tui/src/app.rs`

### Changes Applied

**Struct fields removed from `App`:**
- `pub selected: usize`
- `pub display_indices: Vec<usize>`
- `pub grouping: bool`
- `pub group_by: GroupByCategory`
- `pub display_rows: Vec<DisplayRow>`
- `pub sort_order: SortOrder`
- `pub filter_query: String`

**Functions deleted:**
- `fn clamp_selection(&mut self)`
- `fn rebuild_display_indices(&mut self)`

**Functions updated (all route through active pane):**
- `rebuild_and_reanchor` — removed sync block and `rebuild_display_indices()` call
- `canonical_selected` / `active_canonical_selected`
- `toggle_task_selection` / `apply_range_selection`
- `pane_move_down` / `pane_move_up` — removed `use_global_cursor` sync
- `handle_normal_key` — page scroll handlers (Ctrl+U/D, Shift variants), deferred_toggle
- `push_undo_entry` / `apply_undo`
- `save_and_exit` (both Adding and Editing branches)
- `copy_selected_to_clipboard`
- `render_task_list` — uses `let pane = &self.panes[self.active_pane]` throughout
- `render_status_bar` — removed single-pane fallback branch, always uses pane
- `status_scope_task_indices` — always uses active pane display_rows

**Tests updated:**
- Deleted 1 test: `rebuild_display_indices_does_not_clear_selected_tasks` (function no longer exists)
- Updated ~40 test assertions: `app.selected` → `app.panes[0].selected`, `app.display_rows` → `app.panes[0].display_rows`, `app.filter_query` → `app.panes[0].filter_query`, etc.
- Deleted stale global-state assertions in `filter_defining_enter_writes_to_active_pane_not_global` and `pane_initializes_group_by_to_priority`

## Result

- **Commit:** `5fd2fe5`
- **Tests:** 229/229 pass (230 - 1 deleted obsolete test)
- **Build:** Clean (`cargo build -p todotxt-tui` succeeds with 0 errors)

## Deviations from Plan

None — plan executed exactly as written. The test count is 229 (not 230) because one test that called the deleted `rebuild_display_indices` function was intentionally deleted as specified in the plan.

## Self-Check: PASSED

- [x] `crates/todotxt-tui/src/app.rs` modified
- [x] Commit `5fd2fe5` exists
- [x] 229 tests pass
- [x] `cargo build -p todotxt-tui` clean

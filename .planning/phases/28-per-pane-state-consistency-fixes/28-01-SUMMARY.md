# Phase 28-01 Summary — Per-Pane State Consistency Fixes

**Phase:** 28-per-pane-state-consistency-fixes  
**Plan:** 01  
**Status:** COMPLETE  
**Commit:** e5d25eb  
**Date:** 2025-01-29

---

## What Was Built

All five per-pane state consistency bugs identified in the v1.4 milestone audit were fixed in a single plan execution against `crates/todotxt-tui/src/app.rs`.

### FAIL-1 Fixed (PANE-03) — FilterDefining writes to active pane

`handle_filter_defining_key` Enter arm now captures `new_query` as a local value while the `state` borrow is active, drops the state, then writes `self.active_pane_mut().filter_query = new_query`. The global `self.filter_query` is no longer written. `toggled_filter_query` reset moved to after state is cleared.

Live-preview `_` arm similarly changed: `preview_query` captured from `filter_defining_state`, then assigned to `self.active_pane_mut().filter_query`.

### WARN-1 Fixed (VIEW-02) — Status bar panes_hidden guard

`render_status_bar` condition changed from  
`if !self.should_show_single_pane() && self.panes.len() > 1`  
to  
`if !self.should_show_single_pane() && self.panes.len() > 1 && !self.panes_hidden`  

Status bar now shows global filter info when Ctrl+P hides panes, matching the actual rendered task area.

### WARN-2 Fixed (PANE-04) — Non-Normal draw arms use render_panes

All seven non-Normal `draw()` arms (DeleteConfirm, Adding/Editing, AppendText, Filtering, FilterDefining, KeymapErrors, Help) now call `self.render_panes(frame, chunks[0])` instead of `self.render_task_list(...)`. The task area behind dialogs now respects per-pane filters.

### WARN-3 Fixed (PANE-01) — All panes rebuild after task mutations

New `rebuild_all_panes()` public method loops over all panes applying each pane's own `filter_query`, `sort_order`, and `grouping` to produce fresh `display_rows`. Callers updated:
- `pane_toggle_done` → `rebuild_all_panes()`
- `save_and_exit` Adding arm → `rebuild_all_panes()` after `rebuild_display_indices()`
- `save_and_exit` Editing arm → `rebuild_all_panes()` after `rebuild_display_indices()`
- `FileChanged` handler → `rebuild_all_panes()` before `rebuild_and_reanchor()`

### WARN-4 Fixed (PANE-02) — Cursor reanchor targets pane.selected

`rebuild_and_reanchor` now captures `old_canonical` from `panes[active_pane].display_rows[pane.selected]` in multi-pane mode (instead of `self.canonical_selected()` which read the global cursor). After `rebuild_visible_rows()`, the active pane's `pane.selected` is reanchored to the matching task row.

---

## Test Added

`filter_defining_enter_writes_to_active_pane_not_global` (inline `#[cfg(test)]` module in `app.rs`):
- Creates a two-pane app via helper `make_two_pane_app()`
- Sets up `FilterDefiningState` with `active_editor` containing "+work", `selected_row: 0`
- Calls `handle_filter_defining_key(Enter)`
- Asserts: `panes[0].filter_query == "+work"`, `panes[1].filter_query == ""`, `filter_query == ""`
- Asserts: mode → Normal, `filter_defining_state` → None

---

## Files Modified

| File | Changes |
|------|---------|
| `crates/todotxt-tui/src/app.rs` | 173 insertions, 16 deletions — all five fixes + test + `rebuild_all_panes()` method |

---

## Test Results

```
test result: ok. 72 passed; 0 failed — app::tests (inline)
test result: ok. 18 passed; 0 failed — pane_integration_test
test result: ok.  8 passed; 0 failed — fallback_test
test result: ok.  3 passed; 0 failed — config_panes_test
test result: ok.  5 passed; 0 failed — path_resolution_test
```

Total: 106 tests, 0 failures.

---

## Requirements Satisfied

| Requirement | Status |
|-------------|--------|
| PANE-03 | SATISFIED — FilterDefining dialog writes to active pane filter |
| PANE-04 | SATISFIED — Non-Normal draw arms use render_panes (per-pane filters respected behind dialogs) |

---

## Decisions Made

- `rebuild_all_panes()` uses a sub-block to drop `filtered: Vec<(usize, &Task)>` before mutably borrowing `self.panes[idx]`, making the borrow pattern explicit and safe.
- Kept `rebuild_and_reanchor()` still calling `rebuild_visible_rows()` (not `rebuild_all_panes()`) because it is called frequently (every keypress during live-preview) and only needs to update the active pane for cursor anchoring; `rebuild_all_panes()` is called separately before it in mutation paths.
- Global `self.filter_query` intentionally NOT written in FilterDefining flow; single-pane mode continues to read from `self.filter_query` via the normal `f` filter panel which still writes to the global field (no change to that flow).

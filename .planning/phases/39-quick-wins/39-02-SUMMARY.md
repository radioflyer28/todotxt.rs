# Phase 39-02 Summary: TDD bulk_mark_done

## Status: COMPLETE ✅

## What Was Built
- `App::bulk_mark_done()` method: marks all incomplete tasks in `selected_tasks` as done in one batch
  - Pushes a single undo entry before the loop
  - Skips already-completed tasks
  - Clears `selected_tasks` after
  - Posts `"Marked N task(s) done"` to the status bar
  - Calls `rebuild_all_panes()` + `rebuild_and_reanchor()` once after all marks
- `x` key routing in `handle_normal_key` updated: routes to `bulk_mark_done()` when `selected_tasks` is non-empty, otherwise falls through to `pane_toggle_done()` (existing single-task behavior preserved)

## Files Modified
- `crates/todotxt-tui/src/app.rs` — `bulk_mark_done()` method, `toggle_done` key arm routing

## TDD Cycle
**RED commit**: `test(39-02): RED — failing tests for bulk_mark_done`  
**GREEN commit**: `feat(39-02): implement bulk_mark_done — batch mark selected tasks done, single undo, x routing`

## Tests Added (6)
- `bulk_mark_done_marks_incomplete_tasks` — all selected incomplete tasks become done
- `bulk_mark_done_skips_already_done_tasks` — already-done tasks remain done, incomplete become done
- `bulk_mark_done_pushes_single_undo_entry` — exactly one undo entry is created
- `bulk_mark_done_clears_selection_after` — selection is cleared post-bulk
- `bulk_mark_done_posts_status_message` — status bar contains "Marked" and "done"
- `toggle_done_routes_to_bulk_when_selection_nonempty` — routing produces correct side effects

---
id: SEED-009
status: dormant
planted: 2026-05-04
planted_during: v1.5 complete / preparing v1.6
trigger_when: next milestone (v1.6)
scope: Small
---

# SEED-009: Bulk mark-done via multi-selection

## Why This Matters

Multi-selection already drives bulk delete (`D`) and bulk append (`T`). Pressing `x` with a multi-selection silently ignores the selection and only toggles the cursor task. Users who batch-complete work items (e.g., end-of-day review) have to press `x` once per task — a friction-heavy workflow that the existing selection infrastructure was designed to eliminate.

## When to Surface

**Trigger:** Next milestone (v1.6).

Matches when:
- TUI action or selection-driven workflows are in scope
- Bulk operation improvements are planned

## Scope Estimate

**Small** — The bulk delete path (`bulk_delete`) is the direct template. The same selection drain loop applies, replacing delete with `toggle_done`. Key work:
- When `selected_tasks` is non-empty and `toggle_done` is triggered, route to a `bulk_toggle_done` path
- Apply `with_completed(!task.completed)` across all selected canonical indices
- Push a single undo entry covering the whole batch (consistent with bulk delete)
- Rebuild all panes and clear selection

## Breadcrumbs

| File | Notes |
|------|-------|
| `crates/todotxt-tui/src/app.rs` line 831–832 | `toggle_done` action handler — entry point to extend |
| `crates/todotxt-tui/src/app.rs` line 2700–2712 | `toggle_done()` single-task implementation |
| `crates/todotxt-tui/src/app.rs` line 2735–2749 | `pane_toggle_done()` — pane-aware single-task version |
| `crates/todotxt-tui/src/app.rs` (bulk_delete handler) | Pattern to follow for bulk toggle |
| `crates/todotxt-tui/src/app.rs` line 3333 | Action name list — add `bulk_toggle_done` or reuse `toggle_done` |

## Notes

Decide whether bulk mark-done requires a confirmation step (like bulk delete has `DeleteConfirm` mode). Given `x` is reversible via undo, a direct action without confirmation is probably fine — consistent with how bulk append works.

Also: toggling a mixed selection (some done, some not) should complete all incomplete tasks (not toggle each independently), matching the principle of least surprise.

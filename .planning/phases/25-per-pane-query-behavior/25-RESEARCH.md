# Phase 25: Per-Pane Query Behavior - Research

**Date:** 2026-04-28  
**Phase:** 25 - Per-Pane Query Behavior (Sort/Group/Filter)  

## RESEARCH COMPLETE

### Technical Analysis

#### Current State (Phase 24 Foundation)
- Pane struct exists with `filter_query`, `sort_order`, `label` fields
- Active pane focus mechanics implemented (left/right arrow navigation)
- App struct maintains `panes: Vec<Pane>` and `active_pane: usize`
- Task display rebuilding exists but currently single-pane scoped

#### What Phase 25 Needs to Implement
1. **Route hotkeys to active pane context** — Existing filter/sort/group hotkeys must check `app.active_pane` before mutating state
2. **Per-pane task rebuilding** — `rebuild_visible_tasks` must operate on `panes[active_pane]` fields
3. **State consistency** — When pane switches, re-render from that pane's filter_query + sort_order

#### Key Implementation Areas
- `handle_input()` in app.rs — Hotkey routing (filter, sort, group)
- `rebuild_visible_tasks()` — Apply active pane's filter/sort/group
- `render()` — Display active pane's filtered/sorted task list
- Pane struct — Ensure sort_order and filter_query are properly typed and initialized

#### No Blockers
- Phase 24 laid all groundwork (Pane struct, active focus)
- Existing filter/sort engines can be called with pane-scoped state
- No new dependencies or external libraries needed

### Validation Architecture

**Dimension 1 (Requirements Coverage):** PANE-03 (per-pane filter), PANE-04 (per-pane sort/group) — both must show in plan acceptance criteria

**Dimension 2 (Hotkey Routing):** Verify that filter/sort/group hotkeys mutate active pane only:
- Filter hotkey applies to `panes[active_pane].filter_query`
- Sort hotkey applies to `panes[active_pane].sort_order`
- Inactive panes unchanged

**Dimension 3 (Task Rebuilding):** Verify display_rows reflect active pane's query state:
- When filter changes, `display_rows` updates from `panes[active_pane].filter_query`
- When sort changes, `display_rows` re-sorts using `panes[active_pane].sort_order`
- When pane switches, `display_rows` rebuilt from new active pane's state

**Dimension 4 (Navigation Safety):** Verify pane switching with empty/filtered results:
- Switching to empty pane doesn't crash (empty display_rows OK)
- Switching between filtered panes correctly applies each pane's filter
- Switching back to previously-viewed pane preserves its filter/sort state

---

## Research Summary

Phase 25 is straightforward: wire up per-pane query state that Phase 24 created. No new patterns needed — route existing hotkey handlers to check active pane and mutate accordingly. Main risk: ensuring filter/sort rebuilding runs per-pane, not globally.


# Phase 25: Per-Pane Query Behavior (Sort/Group/Filter) - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Route existing filter/sort/group hotkeys to the active pane context so each pane maintains independent query state (filter, sort, group). Enables multi-pane task workflows where users can view different filtered/grouped views simultaneously.

</domain>

<decisions>
## Implementation Decisions

### Filter Query Behavior Per-Pane
- **D-01:** Each pane holds its own `filter_query: String` (established in Phase 24 Pane struct)
- **D-02:** Filter hotkeys (e.g., `/` for filter input) apply only to the active pane
- **D-03:** Existing filter preset hotkeys (if any) must also apply per-pane context
- **D-04:** When switching between panes, each pane's task list updates to reflect its filter query immediately (no rebuild delay)

### Sort/Group State Per-Pane
- **D-05:** Each pane holds its own `sort_order: SortOrder` (established in Phase 24 Pane struct)
- **D-06:** Each pane holds independent grouping state (to be determined: separate field or part of sort_order enum)
- **D-07:** Sort hotkeys (e.g., `s` for sort menu) apply to the active pane only
- **D-08:** Group hotkeys (e.g., `g` for group menu) apply to the active pane only
- **D-09:** When pane switches, the active pane's tasks re-render with its own sort/group rules

### Query Hotkey Routing
- **D-10:** The active pane is the sole recipient of query hotkeys (filter/sort/group) — inactive panes remain unchanged
- **D-11:** Pane focus (left/right arrow) updates `app.active_pane` and immediately triggers re-render with the newly-active pane's filter/sort/group
- **D-12:** Query hotkey dispatch in `handle_input` checks `app.active_pane` before applying changes

### Empty State and Navigation Safety
- **D-13:** If a filter leaves a pane empty (0 tasks), the pane still displays its empty state and remains interactive
- **D-14:** User can still apply sort/group to an empty pane (settings are preserved for when matching tasks return)
- **D-15:** Navigating to an empty pane is allowed (no "skip empty panes" auto-jump)

### Integration with Phase 24 Foundation
- **D-16:** Phase 24 established Pane struct with `filter_query`, `sort_order`, `label` fields — Phase 25 activates and routes to these
- **D-17:** Phase 24's active-pane focus mechanics (left/right arrow) are the input path for Phase 25 state updates
- **D-18:** Phase 24's task list rebuild logic must call filter + sort + group per active pane before rendering

### the agent's Discretion
- Exact enum/struct names for GroupingState (can reuse existing or create new)
- Whether sort_order and grouping are combined or separate fields in Pane
- Performance optimization strategies for multi-pane filtering (e.g., caching, lazy evaluation)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 24 Foundation
- `.planning/phases/24-pane-model-layout-foundation/24-01-PLAN.md` — Pane struct definition and active-pane focus mechanics
- `.planning/REQUIREMENTS.md` § PANE-03, PANE-04 — Per-pane filter and sort/group requirements

### Existing Patterns and Code
- `crates/todotxt-tui/src/app.rs` § App struct and display_rows derivation — how tasks are currently filtered/sorted
- `crates/todotxt-tui/src/state.rs` § Pane struct (from Phase 24) — per-pane state fields to activate
- `crates/todotxt-core/src/list.rs` (if exists) — filter and sort engine signatures and return types

### Related Requirements
- `.planning/REQUIREMENTS.md` § All PANE-* and VIEW-* requirements — understand full pane lifecycle scope

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Pane struct (Phase 24):** Already holds `filter_query: String`, `sort_order: SortOrder`, `label` — Phase 25 wires these
- **filter_query field:** Ready to receive user input from hotkey handler
- **sort_order field:** Ready to receive user sort preferences

### Established Patterns
- **Active pane focus pattern (Phase 24):** `app.active_pane` index determines which pane receives input
- **Task display derivation:** `display_rows` rebuild pattern — needs to call filter → sort → group on active pane
- **Hotkey dispatch pattern:** `handle_input` routes KeyCode to app methods — Phase 25 extends to pass pane context

### Integration Points
- **Input handling:** Hotkey handler must check `app.active_pane` before modifying `panes[active_pane].filter_query`, `.sort_order`, etc.
- **Render loop:** `rebuild_visible_tasks` must operate on active pane's state fields
- **Mode enum:** May need to track which pane's filter input is active (if filter dialog is pane-scoped)

</code_context>

<specifics>
## Specific Ideas

- Users should be able to see filter queries at a glance (consider status bar hint showing "active pane: Work (filtered by 'project:home')")
- When switching panes, consider brief visual indicator (highlight or animation) so user knows which pane is now active
- Preserve sort/group state when pane is empty (so adding matching tasks shows them pre-sorted)

</specifics>

<deferred>
## Deferred Ideas

- Per-pane presets (Phase 27+)
- Per-pane saved query templates (future feature)
- Pane reordering and pinning (v2 requirements)

</deferred>

---

*Phase: 25 - Per-Pane Query Behavior*
*Context gathered: 2026-04-28 — chain mode (discuss → plan → execute)*

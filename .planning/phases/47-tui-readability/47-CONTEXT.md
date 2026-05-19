# Phase 47: TUI Readability - Context

**Gathered:** 2026-05-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 47 improves visual readability in grouped and multi-pane TUI task lists without
changing the underlying pane model. The scope is limited to two behaviors already in the
roadmap: inactive panes must stop rendering a cursor highlight while still remembering
their selected row, and grouped views must insert a blank spacer row before each
non-first group header while preserving predictable task-only navigation.

</domain>

<decisions>
## Implementation Decisions

### Inactive Pane Focus Rendering
- **D-01:** Inactive panes keep their remembered `pane.selected` index, but render with no
  row emphasis at all. There should be exactly one visible cursor highlight in the UI: the
  active pane's.
- **D-02:** Restoring focus to a pane should make the cursor reappear at that remembered
  row without re-anchoring elsewhere just because the pane was inactive.

### Group Spacer Rendering
- **D-03:** Each non-first group header gets a truly blank spacer row before it. The spacer
  is whitespace only, not a separator glyph or stylized divider.
- **D-04:** The first group in a grouped list does not receive a leading spacer row.

### Selection and Navigation Semantics
- **D-05:** Selection must always anchor to a task row, never a group header or spacer row,
  after rebuilds caused by grouping, sorting, pane switching, or other refresh paths.
- **D-06:** Navigation should treat both headers and spacer rows as non-selectable
  structure. Movement logic must skip over them consistently.

### Single-Pane and Multi-Pane Consistency
- **D-07:** Grouped single-pane mode and grouped multi-pane mode should use the same spacer
  insertion rules and the same non-selectable structure-row behavior.

### Folded Todos
- **Hide cursor highlight in inactive TUI panes** — The pending todo precisely matches the
  phase requirement that inactive panes preserve selection state without rendering a cursor
  highlight. It grounds the render-path change in `pane_list.rs`.
- **Add spacer row between TUI group headers** — The pending todo precisely matches the
  phase requirement for a blank visual gap between grouped sections and already identifies
  the likely row-model, render, and navigation touchpoints.

### the agent's Discretion
- Whether spacer rows are represented by a new `DisplayRow` variant or another equivalent
  internal structure, as long as they remain non-selectable and render as true blank lines.
- Exact test placement and naming for cursor-hiding and grouped-row navigation regressions.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and roadmap
- `.planning/ROADMAP.md` — Phase 47 goal, requirement mapping, and success criteria.
- `.planning/REQUIREMENTS.md` — `TUI-01` and `TUI-02` define the required inactive-pane and
  grouped-spacing behavior.

### Folded todos
- `.planning/todos/pending/2026-05-15-hide-cursor-in-inactive-panes.md` — concrete render
  expectation for inactive panes.
- `.planning/todos/pending/2026-05-15-add-group-spacer-row.md` — concrete row-model and
  navigation expectations for grouped spacing.

### Prior context
- `.planning/phases/43-view-state-persistence/43-CONTEXT.md` — establishes that pane state,
  including per-pane selection context, is meant to persist independently rather than be
  flattened into a single shared active-view model.

### TUI implementation targets
- `crates/todotxt-tui/src/components/pane_list.rs` — pane row rendering and list highlight
  behavior for active vs inactive panes.
- `crates/todotxt-tui/src/state.rs` — `DisplayRow` and pane selection state model.
- `crates/todotxt-tui/src/app.rs` — grouped `display_rows` construction, cursor re-anchoring,
  and header-skipping navigation logic.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `PaneList::render` in `crates/todotxt-tui/src/components/pane_list.rs` already receives
  `is_active` and controls `ListState`, so it is the natural place to suppress inactive-pane
  highlight rendering.
- `DisplayRow` in `crates/todotxt-tui/src/state.rs` already models structural list rows via
  `GroupHeader`, so grouped spacing can extend the same display-row abstraction.
- `app.rs` already contains multiple navigation guards that skip `GroupHeader` rows, giving
  the phase an established pattern for non-task row skipping.

### Established Patterns
- Pane state is independent per pane (`display_rows`, `selected`, `grouping`, `group_by`),
  so readability changes should preserve that independence rather than introduce shared
  active/inactive row state.
- Grouped views are built as explicit `display_rows` lists and then rendered generically,
  which means spacer behavior should be added at row-construction level, not hacked into
  rendering only.

### Integration Points
- Grouped row-building paths in `crates/todotxt-tui/src/app.rs` need spacer insertion.
- Cursor movement and re-anchor flows in `crates/todotxt-tui/src/app.rs` need to skip both
  headers and spacers.
- Any exhaustive `DisplayRow` matches in render/export/helper code need to account for the
  new structural row behavior.

</code_context>

<specifics>
## Specific Ideas

- Inactive panes should look like plain lists with no remembered-row cue at all.
- Spacer rows should read as empty breathing room, not as decorative separators.
- The grouped behavior should stay visually and behaviorally identical between single-pane
  and multi-pane layouts.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 47-tui-readability*
*Context gathered: 2026-05-15*

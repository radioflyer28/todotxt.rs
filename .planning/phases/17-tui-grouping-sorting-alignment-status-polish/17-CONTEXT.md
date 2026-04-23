# Phase 17: TUI Grouping/Sorting Alignment + Status Polish - Context

**Gathered:** 2026-04-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 17 delivers three TUI features:
1. **Task grouping** (V12-TUI-GROUP-01) — visually group tasks by shared sort key, toggled with `g`; header rows with reversed styling; groups by whatever sort order is active
2. **Status bar polish** (V12-TUI-STATUS-01) — remove the theme label from the status bar entirely
3. **Deferred task toggle** (V12-TUI-DEFER-02) — `h` key toggles deferred (`t:`) task visibility; dim styling for visible deferred tasks; `[+deferred]` indicator in status bar when active

</domain>

<decisions>
## Implementation Decisions

### Grouping (V12-TUI-GROUP-01)

- **D-01:** Grouping is toggled on/off with the `g` key (independent of sort order)
- **D-02:** When grouping is on, groups are determined by the active sort order — all 8 sort orders produce groups (e.g., Priority → groups by letter; Project → groups by `+tag`; FileOrder → single group)
- **D-03:** Group boundaries are rendered as **header rows** — a dedicated row styled with `Modifier::REVERSED` showing just the key value (e.g., `+myproject`, `@work`, `(A)`, `none`)
- **D-04:** Header rows are **decorative only** — `j`/`k` navigation skips them; task action keys (`x`, `d`, `n`, `u`, Enter) always act on a task row, never a header
- **D-05:** Group header label is just the key value — no task count, no extra decoration
- **D-06:** Status bar shows `| group: on` when grouping is active (alongside the existing sort label)

### Status Bar (V12-TUI-STATUS-01)

- **D-07:** The theme label (`| theme: ...`) is **removed entirely** from the status bar — no conditional display, just gone

### Deferred Task Toggle (V12-TUI-DEFER-02)

These decisions are **locked** in `14-DEFER-SPEC.md`. Phase 17 implements the TUI side:
- **D-08:** `h` key toggles deferred task visibility (mnemonic: hidden tasks)
- **D-09:** `show_deferred: bool` field on `App` struct (default: `false` — deferred tasks hidden)
- **D-10:** When `show_deferred` is `true`, deferred tasks are rendered with `Modifier::DIM`
- **D-11:** Status bar shows `[+deferred]` when `show_deferred` is `true`

### Agent's Discretion

- Internal data structure for grouping (e.g., `GroupedDisplay` enum or `Vec<DisplayRow>` with a header variant) — planner decides
- How `rebuild_display_indices()` is extended or replaced to support header rows — planner decides
- Key hint display in the right help string for `g` and `h` keys — planner decides

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core sort/filter logic
- `crates/todotxt-core/src/sort.rs` — `SortOrder` enum and `compare()` — grouping key extraction needs to match sort semantics
- `crates/todotxt-core/src/filter.rs` — `suppress_future_threshold: bool` field used for deferred task filtering

### TUI app state and rendering
- `crates/todotxt-tui/src/app.rs` — `App` struct, `display_indices: Vec<usize>`, `rebuild_display_indices()`, `render_task_list()`, `render_status_bar()`, `cycle_sort()` — all need changes for grouping and deferred toggle
- `crates/todotxt-tui/src/theme.rs` — `Theme` enum, `StyleSheet` — `Modifier::REVERSED` already used for selected rows; reuse for group headers

### Spec documents (LOCKED decisions)
- `.planning/phases/14-compat-discovery/14-DEFER-SPEC.md` — Locked spec for deferred task (`t:`) TUI behavior (D-08 through D-11 above). MUST read before planning the deferred toggle.

### Prior phase context
- `.planning/phases/12-filter-sort/12-CONTEXT.md` — D-10/D-11: `display_indices` view model design, sort-is-view-only constraint
- `.planning/phases/16-tui-filter-ux-alignment/16-CONTEXT.md` — Esc/snapshot patterns, mode split for `AppMode`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Modifier::REVERSED`: already used for selected task row styling — reuse directly for group header rows
- `Modifier::DIM`: already used for completed task styling — reuse for deferred tasks
- `cycle_sort()` / `sort_name()`: functions in `app.rs` used for the `o` key cycle — grouping references the same `SortOrder`
- `rebuild_display_indices()`: current function applies filter + sort to build `Vec<usize>` — needs to be extended/replaced to also emit header rows when grouping is active

### Established Patterns
- View model is `display_indices: Vec<usize>` — sorting is view-only, never mutates `TaskList`. Grouping must follow the same pattern (no task mutation).
- `AppMode` enum gates behavior (e.g., `Filtering`, `FilterDefining`, `Editing`) — a new mode may not be needed for grouping (it's a rendering concern, not a modal state)
- Status bar `middle` string is built as a `String` with `push_str` — `| group: on` and `[+deferred]` follow the same pattern

### Integration Points
- `rebuild_display_indices()` is the central integration point — it's called on every filter/sort/reload/mutation cycle and produces the list the renderer consumes
- `render_task_list()` iterates `display_indices` to draw rows — needs to also handle header rows
- `render_status_bar()` builds `middle` string — two new indicators to add (`| group: on`, `[+deferred]`)
- Key handler dispatch (the main `match event.code` in `app.rs`) — add `g` and `h` cases

</code_context>

<specifics>
## Specific Ideas

- The `g` key was chosen for grouping toggle (mnemonic: group)
- The `h` key was chosen for deferred toggle (mnemonic: hidden tasks)
- Theme label removed entirely — not just conditionally hidden — simplifies `render_status_bar()` logic
- Group headers show just the raw key value (`+myproject`, `@work`, `(A)`, `none` for tasks with no value in the sort dimension)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 17-tui-grouping-sorting-alignment-status-polish*
*Context gathered: 2026-04-23*

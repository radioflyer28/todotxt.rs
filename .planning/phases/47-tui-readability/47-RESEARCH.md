---
phase: 47-tui-readability
type: research
created: 2026-05-15
requirements: [TUI-01, TUI-02]
---

# Phase 47: TUI Readability Research

## Scope

Phase 47 is a focused TUI readability phase. It changes the visible behavior of pane
cursor highlighting and grouped row spacing without changing task data, pane persistence,
filter semantics, sorting semantics, or command behavior.

## Existing Implementation

### Pane Rendering

`crates/todotxt-tui/src/components/pane_list.rs` renders each pane through
`PaneList::render`. It already receives `is_active`, but currently creates a
`ListState` with `Some(pane.selected)` for every non-empty pane. Ratatui therefore draws a
highlight in inactive panes as well as the active pane.

The minimal fix is to keep `pane.selected` untouched and only pass
`Some(pane.selected)` to `ListState` when `is_active` is true and `label_selected` is
false. Inactive panes should pass `None`.

### Grouped Row Model

`crates/todotxt-tui/src/state.rs` defines:

```rust
pub enum DisplayRow {
    Task(usize),
    GroupHeader(String),
}
```

Grouped rows are built in `crates/todotxt-tui/src/app.rs` in at least two paths:

- `rebuild_display_indices` / active pane rebuild path around grouped row construction
- `rebuild_all_panes` grouped row construction for all panes

Both paths emit `DisplayRow::GroupHeader(key)` immediately followed by task rows. Phase 47
needs a blank structural row before each non-first group header. The cleanest model is a
new `DisplayRow::GroupSpacer` variant because rendering, status counts, task actions, and
navigation can all treat it as non-task structure.

### Navigation and Re-Anchor Behavior

`app.rs` already contains repeated guards like:

```rust
matches!(pane.display_rows[idx], DisplayRow::GroupHeader(_))
```

These occur in half-page movement, pane up/down movement, and re-anchor paths. Phase 47
should replace ad hoc header-only checks with a shared predicate such as
`is_non_task_structure_row` or `is_selectable_task_row`, then use that helper everywhere
navigation or normalization needs to skip non-task rows.

### Exhaustive Matches

Adding `DisplayRow::GroupSpacer` requires updating exhaustive matches in:

- `components/pane_list.rs` row-to-`ListItem` mapping
- single-pane `render_task_list`
- `status_scope_task_indices`
- any tests or helpers that match `DisplayRow`

## Recommended Plan Shape

Use two plans:

- `47-01` handles TUI-01 only: active pane owns the only visible cursor highlight.
- `47-02` handles TUI-02 plus shared navigation rules: add `GroupSpacer`, insert it in
  grouped row construction, render it as a blank row, and skip it everywhere headers are
  skipped.

This split keeps the low-risk render-only fix separate from the wider row-model change.

## Validation Strategy

Automated verification should focus on Rust unit tests in `todotxt-tui`:

- Unit-level rendering tests for inactive pane `ListState` behavior are difficult to
  inspect directly through ratatui internals, so add a small helper in `PaneList` if
  needed to make active/inactive selected-row selection testable without snapshot tests.
- Row-model tests should assert grouped rows include spacer rows only before non-first
  group headers.
- Navigation tests should assert Down/Up and half-page movement skip both group headers
  and spacer rows.
- Regression tests should cover single-pane and multi-pane grouped views using the same
  row rules.

Suggested verification commands:

```powershell
cargo test -p todotxt-tui pane_list
cargo test -p todotxt-tui group_spacer
cargo test -p todotxt-tui tui_readability
```

## Risks

- Adding a new `DisplayRow` variant can break exhaustive matches. Search for all
  `DisplayRow::GroupHeader` and `match row` call sites during execution.
- If spacer rows are inserted only in rendering, navigation will not know about them and
  tests may miss single-pane/multi-pane parity. Prefer model-level spacer rows.
- If selection normalization only clamps by index, it can still land on a spacer/header
  after row rebuilds. Normalize to the nearest task row when any task exists.


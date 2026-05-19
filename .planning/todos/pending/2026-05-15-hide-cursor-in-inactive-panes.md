---
created: 2026-05-15T00:00:00
title: Hide cursor highlight in inactive TUI panes
area: tui
resolves_phase: 47
files:
  - crates/todotxt-tui/src/components/pane_list.rs:184-200
  - crates/todotxt-tui/src/app.rs:129
---

## Problem

When moving the cursor between panes in the TUI, the cursor highlight remains visible in the previously active pane(s). With multiple panes open, the user sees multiple highlighted cursor rows simultaneously — one per pane that has ever been visited — which is visually confusing.

The desired behavior:
- Only the **active pane** (identified by `app.active_pane`) shows a highlighted cursor row.
- Inactive panes remember their cursor position (`pane.selected`) but render **without** the highlight.
- When the user returns to a previously visited pane, the cursor reappears at the remembered position.

## Solution

In `PaneList::render` (pane_list.rs), the `ListState::with_selected` call unconditionally passes `Some(pane.selected)`, which causes ratatui to render the highlight style on that row regardless of `is_active`.

Fix: only pass `Some(pane.selected)` to `list_state.with_selected` when `is_active` is `true`; pass `None` otherwise. This preserves the stored cursor position in `pane.selected` while suppressing the visual highlight in inactive panes.

```rust
// Before
if !label_selected && !pane.display_rows.is_empty() {
    list_state = list_state.with_selected(Some(pane.selected));
}

// After
if is_active && !label_selected && !pane.display_rows.is_empty() {
    list_state = list_state.with_selected(Some(pane.selected));
}
```

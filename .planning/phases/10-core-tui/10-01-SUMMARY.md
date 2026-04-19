# Plan 10-01 Summary — List+ListState nav

**Status:** Complete  
**Commit:** `3e2ecd6`  
**Date:** 2026-04-19

## What was built

Replaced the Phase 9 `Paragraph`-based render with ratatui `List` + `ListState`, adding full keyboard navigation and selection state.

### Changes to `crates/todotxt-tui/src/app.rs`

**New fields on `App`:**
- `selected: usize` — 0-based index of highlighted row; clamped to `[0, task_count-1]` at all write sites
- `list_height: u16` — height of the list area (total rows minus 1 status bar row); updated on `Resize`

**`App::run()`:** Captures initial terminal size to seed `list_height` before the first draw.

**`handle_event()` — new key branches:**
| Key | Action |
|-----|--------|
| `j` / `↓` | Move down 1 |
| `k` / `↑` | Move up 1 |
| `g` | Jump to first task |
| `G` | Jump to last task |
| `Ctrl+d` | Half-page down (`list_height / 2`) |
| `Ctrl+u` | Half-page up (`list_height / 2`) |

`FileChanged` handler now clamps `selected` after reload (D-07).  
`Resize` handler updates `list_height` (D-09).

**`draw()` — full replacement:**
- `Layout::vertical([Min(0), Length(1)])` splits list area from 1-row status footer
- `ListItem` per task: `"{1-based}: {raw_text}"` format (D-01)
- Completed tasks: `Modifier::DIM` (D-03)
- Selected row: `Modifier::REVERSED` via `List::highlight_style` (D-06)
- `ListState::with_selected(Some(self.selected))` built fresh each draw (D-05)
- Status bar rendering: `"{filename} | N tasks | V visible | D due today | O overdue  q quit | x done | j/k nav"` (D-15/D-16/D-17)
- Due counts use `DueStatus::Today` / `DueStatus::Overdue` from `todotxt_core`

## Decisions implemented

D-01, D-02, D-03, D-04, D-05, D-06, D-07, D-08, D-09, D-14, D-15, D-16, D-17

## Verification

```
cargo build -p todotxt-tui  → success, 0 warnings
```

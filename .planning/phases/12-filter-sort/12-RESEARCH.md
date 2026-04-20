# Phase 12: Filter + Sort — Research

## Standard Stack

### Core Architecture: display_indices

**New fields on `App`:**
```rust
pub display_indices: Vec<usize>,        // maps display row -> canonical task index
pub sort_order: SortOrder,              // current active sort (FileOrder = no sort)
pub filter_query: String,               // raw text of active filter (empty = no filter)
pub filter_state: Option<FilteringState>, // Some when filter panel is open
pub presets: Vec<(String, String)>,     // (name, query) sorted alphabetically
```

**New struct:**
```rust
pub struct FilteringState {
    pub editor: TextArea<'static>,  // tui-textarea for free-text input
    pub selected_preset: usize,     // highlight position in preset list (0-based)
}
```

**`SortOrder::FileOrder` (new variant):** When `sort_order == SortOrder::FileOrder`, display_indices = 0..task_list.len() (after filtering). No sorting is applied.

**`rebuild_display_indices()` — new private method on App:**
```rust
fn rebuild_display_indices(&mut self) {
    let filter_q = self.filter_query.trim();
    let pairs: Vec<(usize, &Task)> = if filter_q.is_empty() {
        self.task_list.tasks().iter().enumerate().collect()
    } else {
        let f = Filter::from_query(filter_q);
        self.task_list.filter(&f)
    };
    let mut pairs: Vec<(usize, &Task)> = pairs;
    if self.sort_order != SortOrder::FileOrder {
        pairs.sort_by(|(_, a), (_, b)| self.sort_order.compare(a, b));
    }
    self.display_indices = pairs.into_iter().map(|(idx, _)| idx).collect();
}
```

**Rebuild triggers:**
| Event | Action |
|-------|--------|
| Keystroke in Filtering mode | `rebuild_display_indices()` after updating `filter_query` from editor |
| Arrow key / number key loads preset | Set `filter_query` from preset, `rebuild_display_indices()` |
| `o` key cycles sort | Advance `sort_order`, `rebuild_display_indices()` |
| `FileChanged` reload applied | `task_list.reload()`, then `rebuild_display_indices()`, then `clamp_selection()` |
| Task mutated (add/update/delete) | `rebuild_display_indices()` then re-anchor selection |

**Selection tracking (D-12):** After rebuild, try to preserve the canonical index under the cursor:
```rust
fn rebuild_and_reanchor(&mut self) {
    let old_canonical = self.display_indices.get(self.selected).copied();
    self.rebuild_display_indices();
    self.selected = old_canonical
        .and_then(|ci| self.display_indices.iter().position(|&x| x == ci))
        .unwrap_or(0);
    self.clamp_selection();
}
```

**`clamp_selection()` — updated:**
```rust
fn clamp_selection(&mut self) {
    let count = self.display_indices.len();
    if count == 0 { self.selected = 0; } else { self.selected = self.selected.min(count - 1); }
}
```

**`canonical_selected()` — new guard helper:**
```rust
fn canonical_selected(&self) -> Option<usize> {
    self.display_indices.get(self.selected).copied()
}
```
All write operations (toggle done, edit, delete) must use `canonical_selected()` and return early if `None`.

**`App::new()` initialization:**
```rust
let n = task_list.len();
App {
    display_indices: (0..n).collect(),
    sort_order: SortOrder::FileOrder,
    filter_query: String::new(),
    filter_state: None,
    presets,      // passed in from main.rs
    // ... existing fields unchanged
}
```

**Edge case — delete shifts canonical indices:** After `task_list.delete(canonical_idx)`, all indices above `canonical_idx` shift down by 1. display_indices must be fully rebuilt (not patched) — `rebuild_display_indices()` re-enumerates from scratch and handles this correctly.

**Edge case — add:** After `task_list.add(task)`, the new task is at `task_list.len()-1` canonically. Rebuild display_indices; find the display position of the new canonical index and move selection there.

---

### SortOrder Extension

**Three new variants to add to `crates/todotxt-core/src/sort.rs`:**

```rust
/// Original file order — no sort applied (display baseline).
FileOrder,
/// Earliest completion date first. Tasks with no completion date sort last.
CompletedDate,
/// Earliest creation date first. Tasks with no creation date sort last.
CreationDate,
```

**Add to `SortOrder::compare()` match arms:**

```rust
SortOrder::FileOrder => Ordering::Equal, // never called in practice
SortOrder::CompletedDate => {
    match (a.completion_date, b.completion_date) {
        (None, None)         => Ordering::Equal,
        (None, _)            => Ordering::Greater,
        (_, None)            => Ordering::Less,
        (Some(da), Some(db)) => da.cmp(&db),
    }
}
SortOrder::CreationDate => {
    match (a.creation_date, b.creation_date) {
        (None, None)         => Ordering::Equal,
        (None, _)            => Ordering::Greater,
        (_, None)            => Ordering::Less,
        (Some(da), Some(db)) => da.cmp(&db),
    }
}
```

**Task fields used** (confirmed from `task.rs`):
- `completion_date: Option<NaiveDate>`
- `creation_date: Option<NaiveDate>`

Both follow the same None-sorts-last pattern as the existing `DueDate` variant.

**Sort cycle helper (D-09):**
```rust
fn cycle_sort(current: SortOrder) -> SortOrder {
    match current {
        SortOrder::FileOrder      => SortOrder::Alphabetical,
        SortOrder::Alphabetical   => SortOrder::CompletedDate,
        SortOrder::CompletedDate  => SortOrder::Context,
        SortOrder::Context        => SortOrder::DueDate,
        SortOrder::DueDate        => SortOrder::CreationDate,
        SortOrder::CreationDate   => SortOrder::Priority,
        SortOrder::Priority       => SortOrder::Project,
        SortOrder::Project        => SortOrder::FileOrder,
        _ => SortOrder::FileOrder,
    }
}
```

**`SortOrder` is `#[non_exhaustive]`** — adding variants is safe. Fix all exhaustive match arms in `sort.rs` tests and in any match on `SortOrder` in the TUI crate.

---

### Filter Panel Layout

**Panel height:** `panel_height = 1 + min(preset_count, 5)` — 1 row for text input, up to 5 rows for preset list. Minimum 1 (no presets configured).

**Layout split in `draw()` for `AppMode::Filtering`:**
```rust
AppMode::Filtering => {
    let panel_height = 1 + (self.presets.len() as u16).min(5);
    let chunks = Layout::vertical([Min(0), Length(panel_height), Length(1)])
        .split(frame.area());
    self.render_task_list(frame, chunks[0]);
    self.render_filter_panel(frame, chunks[1]);
    self.render_status_bar(frame, chunks[2]);
}
```

**Rendering the filter panel:**
```rust
fn render_filter_panel(&mut self, frame: &mut Frame, area: Rect) {
    // Row 0: tui-textarea text input
    let input_area = Rect { height: 1, ..area };
    if let Some(ref state) = self.filter_state {
        frame.render_widget(&state.editor, input_area);
    }
    // Rows 1..: numbered preset list
    if area.height > 1 && !self.presets.is_empty() {
        let list_area = Rect { y: area.y + 1, height: area.height - 1, ..area };
        let selected_preset = self.filter_state.as_ref().map(|s| s.selected_preset);
        let items: Vec<ListItem> = self.presets.iter().enumerate()
            .map(|(i, (name, query))| {
                ListItem::new(format!("{}. {} - {}", i + 1, name, query))
            })
            .collect();
        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        let mut list_state = ListState::default().with_selected(selected_preset);
        frame.render_stateful_widget(list, list_area, &mut list_state);
    }
}
```

No `ratatui::widgets::Clear` needed — the layout split owns the area exclusively.

---

### Preset Support

**New struct in `crates/todotxt-tui/src/config.rs`:**
```rust
/// Named filter preset from the [presets] TOML section.
/// Mirrors CLI's PresetConfig — duplicated to avoid cross-crate dependency.
#[derive(Debug, Deserialize, Default, Clone)]
pub struct TuiPreset {
    pub filter: Option<String>,
}
```

**Field added to `TuiConfig`:**
```rust
/// Named filter presets. Max 9 per CFG-02. Keys are preset names.
#[serde(default)]
pub presets: HashMap<String, TuiPreset>,
```

**Ordering for stable 1-9 numbering:** HashMap iteration is unordered. Convert to sorted Vec in `main.rs` before passing to `App::new()`:
```rust
let mut presets: Vec<(String, String)> = config.presets
    .into_iter()
    .filter_map(|(name, p)| p.filter.map(|f| (name, f)))
    .collect();
presets.sort_by(|(a, _), (b, _)| a.cmp(b)); // alphabetical by preset name
```

**`App::new()` signature:**
```rust
pub fn new(task_list: TaskList, todo_path: PathBuf, presets: Vec<(String, String)>) -> Self
```

**Number key access:** `self.presets.get(digit as usize - 1)` for `Char('1')..='Char('9')` — 0-based index = key digit - 1. No-op if index >= presets.len().

**TOML config shape** (same as CLI):
```toml
[presets.work]
filter = "+work @office"

[presets.today]
filter = "due:today"
```

---

### Key Dispatch in Filtering Mode

**Full dispatch table for `handle_filtering_key()`:**

| Key | Action |
|-----|--------|
| `Esc` | Clear `filter_query = ""`; reset `filter_state = None`; `mode = AppMode::Normal`; `rebuild_display_indices()` |
| `Enter` | `filter_state = None`; `mode = AppMode::Normal`; keep `filter_query` (filter stays applied) |
| `Down` | `selected_preset = min(selected_preset + 1, presets.len() - 1)`; load preset query into `filter_query` and editor; `rebuild_display_indices()` |
| `Up` | `selected_preset = selected_preset.saturating_sub(1)`; load preset query; `rebuild_display_indices()` |
| `Char('1')..='Char('9')` | Compute `idx = digit - '1'`; if `idx < presets.len()`: load preset query into `filter_query` and editor; `rebuild_display_indices()` |
| Any other key | `state.editor.input_without_shortcuts(Event::Key(key))`; sync `filter_query = state.editor.lines().first().cloned().unwrap_or_default()`; `rebuild_display_indices()` |

**Opening filter panel (`f` in Normal mode, D-08):**
```rust
KeyCode::Char('f') => {
    let mut editor = TextArea::default();
    editor.insert_str(&self.filter_query); // restore prior query if panel was closed with Enter
    self.filter_state = Some(FilteringState { editor, selected_preset: 0 });
    self.mode = AppMode::Filtering;
}
```

**Cycling sort (`o` in Normal mode, D-08):**
```rust
KeyCode::Char('o') => {
    self.sort_order = cycle_sort(self.sort_order);
    self.rebuild_and_reanchor();
}
```

**Write ops blocked in Filtering mode:** `n`, `u`, `e`, `d`, `x` are only bound in `handle_normal_key()`. Since `handle_event()` dispatches on `AppMode` first, these keys are routed to `handle_filtering_key()` which passes them to `editor.input_without_shortcuts()` — correct behavior (typing in the filter query).

**`pending_reload` guard in Filtering mode:** Same as Adding/Editing — set `pending_reload = true` on `FileChanged` if mode is Filtering. Call `apply_pending_reload()` (which calls `rebuild_and_reanchor()`) when transitioning to Normal via Esc or Enter.

---

### Status Bar Changes

**Current format:**
```
{file} | {total} tasks | {visible} visible | {due_today} due today | {overdue} overdue
```

**New format when filter or sort active (D-13):**
```
{file} | {visible}/{total} tasks | {filter_query} | sort: {sort_name}
```
- Middle filter section shown only when `!filter_query.trim().is_empty()`
- Sort section shown only when `sort_order != SortOrder::FileOrder`
- Both can appear simultaneously

**Sort name strings:**
```rust
fn sort_name(order: SortOrder) -> &'static str {
    match order {
        SortOrder::FileOrder     => "file order",
        SortOrder::Alphabetical  => "alpha",
        SortOrder::CompletedDate => "completed",
        SortOrder::Context       => "context",
        SortOrder::DueDate       => "due",
        SortOrder::CreationDate  => "created",
        SortOrder::Priority      => "priority",
        SortOrder::Project       => "project",
        _ => "?",
    }
}
```

**Due today / overdue counts in filtered view:** Compute from `display_indices` only:
```rust
let due_today = self.display_indices.iter()
    .filter(|&&ci| { let t = &tasks[ci]; !t.completed && t.due_status() == DueStatus::Today })
    .count();
```

**Truncation approach:** `total_width = frame.area().width as usize`. Build left, middle, right strings separately; truncate middle (filter query) with `…` if left+middle+right exceeds total_width. Right (key hints) is the lowest priority — omit if no space.

---

## Architecture Patterns

**Methods that use `self.task_list.len()` / `self.task_list.tasks()[idx]` directly and must change:**

| Method | Current | Change |
|--------|---------|--------|
| `handle_normal_key()` navigation | `task_count = self.task_list.len()` | `display_count = self.display_indices.len()` |
| `handle_normal_key()` j/k/g/G/Ctrl+D/Ctrl+U | `.min(task_count - 1)` | `.min(display_count - 1)` |
| `handle_normal_key()` edit (u/e) | `self.task_list.tasks()[self.selected]` | `self.task_list.tasks()[self.display_indices[self.selected]]` |
| `handle_normal_key()` edit: `original_idx` | `original_idx: self.selected` | `original_idx: self.display_indices[self.selected]` (canonical) |
| `handle_delete_confirm_key()` | `self.task_list.delete(self.selected)` | `self.task_list.delete(self.display_indices[self.selected])` |
| `toggle_done()` | `self.task_list.tasks()[self.selected]` | `self.task_list.tasks()[self.display_indices[self.selected]]` |
| `save_and_exit()` Adding | `self.selected = self.task_list.len() - 1` | rebuild then find display pos of new canonical idx |
| `save_and_exit()` Editing | `self.selected = original_idx` (canonical) | rebuild then find display pos of `original_idx` |
| `render_task_list()` | `tasks.iter().enumerate()` | `self.display_indices.iter().map(\|&ci\| (ci, &tasks[ci]))` |
| `render_status_bar()` | `visible = total` | `visible = self.display_indices.len()` |
| `render_delete_confirm()` | `tasks[self.selected]` | `tasks[self.display_indices[self.selected]]` |
| `clamp_selection()` | `self.task_list.len()` | `self.display_indices.len()` |
| `FileChanged` handler | `clamp_selection()` | `rebuild_and_reanchor()` |

**New methods needed:**
- `rebuild_display_indices(&mut self)` — core rebuild logic
- `rebuild_and_reanchor(&mut self)` — rebuild + preserve selection by canonical index
- `canonical_selected(&self) -> Option<usize>` — safe accessor for write ops
- `cycle_sort(SortOrder) -> SortOrder` — (free function or method)
- `sort_name(SortOrder) -> &'static str` — (free function)
- `handle_filtering_key(&mut self, key) -> Result<()>` — Filtering mode dispatcher
- `render_filter_panel(&mut self, frame, area)` — filter panel renderer

---

## Common Pitfalls

1. **`TaskList::sort()` mutates canonical order (D-10):** Never call `task_list.sort()` for display. Use `display_indices` exclusively. Mitigation: `rebuild_display_indices()` uses `sort_by()` on the local `pairs` Vec — the task_list itself is never mutated for sorting.

2. **Post-delete index shift:** After `task_list.delete(canonical_idx)`, every stored canonical index > canonical_idx is now off by 1. Mitigation: always call `rebuild_and_reanchor()` after any delete — full re-enumeration from the current task_list fixes all indices.

3. **Post-add selection:** After `task_list.add()`, the new task is at `task_list.len()-1` canonically but its display position depends on active sort/filter. Mitigation: after rebuild, do `self.display_indices.iter().position(|&x| x == new_canonical_idx)` to find and set the display position. If the new task is filtered out (e.g., filter is active and it doesn't match), stay at current position.

4. **`f` key captured by editor in Filtering mode:** When the filter panel is open, typing `f` goes into the tui-textarea — correct behavior, not a bug. The panel is closed with `Esc` (clear filter) or `Enter` (keep filter). Document this in key hints.

5. **Preset ordering instability:** `HashMap` has no guaranteed iteration order. Mitigation: convert to `Vec<(String, String)>` sorted alphabetically by name immediately after loading from `TuiConfig`. Store the sorted Vec on App. Key `1` always maps to presets[0], etc.

6. **`selected_preset` out of bounds:** If presets are reloaded or panel is reopened, reset `selected_preset = 0` in `FilteringState::new()`.

7. **Empty display_indices on write ops:** When active filter matches nothing, `display_indices` is empty. All write-path code must call `canonical_selected()` and return early on `None`. Guards: `if self.display_indices.is_empty() { return Ok(()); }` at the top of `handle_normal_key()` for all mutation branches.

8. **`pending_reload` during Filtering mode:** `FileChanged` while Filtering must set `pending_reload = true` and not reload immediately. On Esc/Enter exit from Filtering mode, `apply_pending_reload()` fires — but `apply_pending_reload()` must call `rebuild_and_reanchor()` not just `clamp_selection()` after reload.

9. **tui-textarea borrow conflicts in render:** `render_filter_panel` needs `&mut self` for the tui-textarea widget. Accessing `self.presets` after borrowing `self.filter_state` requires care — extract `selected_preset` before the mutable render call, or split the rendering into two steps.

10. **Non-exhaustive SortOrder match after new variants:** Adding `FileOrder`, `CompletedDate`, `CreationDate` will cause compile errors in existing exhaustive matches (tests in `sort.rs`, any match in TUI code). Fix all match arms; add `_ => Ordering::Equal` guard where `#[non_exhaustive]` semantics require it for external crates.

---

## Implementation Order

**Step 1 — display_indices + SortOrder extension (foundation; no visible UX change except sort cycle)**
1. Add `FileOrder`, `CompletedDate`, `CreationDate` to `SortOrder` in `todotxt-core/src/sort.rs`; fix all match arms; add tests for new variants
2. Add `display_indices`, `sort_order`, `filter_query`, `filter_state`, `presets` to `App`; implement `rebuild_display_indices()`, `rebuild_and_reanchor()`, `canonical_selected()`, `clamp_selection()` (updated), `cycle_sort()`
3. Update `render_task_list()`, `render_delete_confirm()` to use `display_indices`
4. Update all write operations (`toggle_done`, `save_and_exit`, `handle_delete_confirm_key`) to use `canonical_selected()`
5. Wire `o` key in `handle_normal_key()` to sort cycle
6. Update `FileChanged` handler to call `rebuild_and_reanchor()`
7. **Verify:** Sort cycles with `o`, file not mutated, selection preserved

**Step 2 — filter panel + key dispatch**
1. Add `AppMode::Filtering` variant; add `FilteringState` struct
2. Add `TuiPreset` + `presets` field to `TuiConfig`; add sorted Vec conversion in `main.rs`; update `App::new()` signature
3. Implement `handle_filtering_key()` with full dispatch table
4. Implement `render_filter_panel()` with tui-textarea + numbered preset List
5. Add `AppMode::Filtering` arm to `draw()` with 3-way layout split
6. Wire `f` key in `handle_normal_key()` to open filter panel
7. Add `pending_reload` guard for Filtering mode; update `apply_pending_reload()` to call `rebuild_and_reanchor()`
8. **Verify:** live filtering, Esc clears, Enter keeps, presets load on arrow/number keys

**Step 3 — status bar + polish**
1. Update `render_status_bar()` with new format: visible/total, filter query, sort name, truncation
2. Add `sort_name()` helper
3. Update due_today/overdue counts to use `display_indices`
4. Update right-side key hints to include `f filter | o sort`
5. End-to-end verify TUI-FILTER-01 through TUI-FILTER-04 requirements

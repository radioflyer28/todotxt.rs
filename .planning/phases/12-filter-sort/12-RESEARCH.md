# Phase 12: Filter + Sort — Research

## Standard Stack

### Core Architecture: display_indices

**The problem:** Currently `selected` is a direct index into `task_list.tasks()`. After Phase 12, the visible row order differs from the canonical storage order (due to filtering and sorting). A mapping layer is required.

**New field on `App`:**
```rust
pub display_indices: Vec<usize>,  // maps display_row → canonical task index
pub sort_order: SortOrder,        // current display sort (FileOrder = no sort)
pub active_filter: Option<String>, // raw query string (None = no filter)
```
`SortOrder::FileOrder` is the new "no-sort" sentinel (canonical order).

**Rebuild function (called after every filter/sort/mutation change):**
```rust
fn rebuild_display_indices(&mut self) {
    use todotxt_core::Filter;

    // Step 1: apply filter (or take all)
    let mut pairs: Vec<(usize, &Task)> = if let Some(ref q) = self.active_filter {
        let f = Filter::from_query(q);
        self.task_list.filter(&f)     // returns Vec<(usize, &Task)>
    } else {
        self.task_list.tasks().iter().enumerate().collect()
    };

    // Step 2: sort pairs if sort_order != FileOrder
    if self.sort_order != SortOrder::FileOrder {
        pairs.sort_by(|(_, a), (_, b)| self.sort_order.compare(a, b));
        // sort_by is stable — ties preserve canonical order
    }

    // Step 3: collect canonical indices
    self.display_indices = pairs.into_iter().map(|(i, _)| i).collect();
}
```

**Canonical index lookup:** `let canonical = self.display_indices[self.selected];`

**When to call `rebuild_display_indices()`:**
| Trigger | Location |
|---------|----------|
| Filter query changes (keystroke) | `handle_filtering_key()` |
| Sort order cycles (`o` key) | `handle_normal_key()` |
| Preset loaded | `handle_filtering_key()` |
| Esc clears filter | `handle_filtering_key()` |
| Task added (`save_and_exit` Adding branch) | after `task_list.add()` |
| Task updated (`save_and_exit` Editing branch) | after `task_list.update()` |
| Task deleted (`handle_delete_confirm_key`) | after `task_list.delete()` |
| File reloaded (`apply_pending_reload`, `FileChanged`) | after `task_list.reload()` |

**Initialization in `App::new()`:**
```rust
// After building the App struct, call rebuild_display_indices() to populate.
// Initial state: no filter, FileOrder → display_indices = 0..task_list.len()
```

---

### Methods in `app.rs` that currently use `task_list` directly and must change

**`handle_normal_key()`**
- `let task_count = self.task_list.len();` → `let task_count = self.display_indices.len();`
- Navigation clamps: same expressions, now over display row count
- `self.task_list.tasks()[self.selected]` (edit branch) → `self.task_list.tasks()[self.display_indices[self.selected]]`

**`clamp_selection()`**
- `let count = self.task_list.len();` → `let count = self.display_indices.len();`
- All clamp arithmetic stays the same

**`toggle_done()`**
- `let count = self.task_list.len();` → `let count = self.display_indices.len();`
- `let idx = self.selected;` → `let idx = self.display_indices[self.selected];`
- Post-update re-clamp: `self.task_list.len()` is still correct (canonical count)
- After mutation: call `rebuild_display_indices()` then `clamp_selection()`

**`save_and_exit()` (Adding branch)**
- After `task_list.add()`: call `rebuild_display_indices()`
- New task canonical index = `task_list.len() - 1`; find its display row: `self.display_indices.iter().position(|&i| i == canonical).unwrap_or(0)`

**`save_and_exit()` (Editing branch)**
- After `task_list.update(original_idx, ...)`: call `rebuild_display_indices()`
- `original_idx` is still valid (update doesn't shift indices)
- Restore display selection: `self.selected = self.display_indices.iter().position(|&i| i == original_idx).unwrap_or(0)`

**`handle_delete_confirm_key()`**
- `let idx = self.selected;` → `let idx = self.display_indices[self.selected];`
- After `task_list.delete(idx)`: call `rebuild_display_indices()` then `clamp_selection()`

**`render_task_list()`**
- Currently iterates `self.task_list.tasks().iter().enumerate()`
- Phase 12: iterate `self.display_indices.iter().enumerate()`:
  ```rust
  let items: Vec<ListItem> = if self.display_indices.is_empty() {
      vec![ListItem::new("(no tasks)")]
  } else {
      self.display_indices.iter().enumerate().map(|(display_row, &canonical_idx)| {
          let t = &self.task_list.tasks()[canonical_idx];
          // canonical_idx + 1 is the file line number (TUI-NAV-03)
          let content = format!("{}: {}", canonical_idx + 1, t.to_raw());
          let style = if t.completed { Style::default().add_modifier(Modifier::DIM) }
                      else { Style::default() };
          ListItem::new(content).style(style)
      }).collect()
  };
  ```
- `list_state.with_selected(Some(self.selected))` is unchanged (still a display-row index)

**`render_status_bar()`**
- `let total = tasks.len();` stays (canonical count)
- `let visible = total;` → `let visible = self.display_indices.len();`
- due_today / overdue counts now iterate `display_indices` (visible tasks only)

---

### SortOrder Extension

Two new variants added to `crates/todotxt-core/src/sort.rs`. `SortOrder` is `#[non_exhaustive]` so adding variants is non-breaking for external users; internal `match` arms in `compare()` and any TUI sort-cycle code must be exhaustive.

**New variant: `FileOrder`** (sentinel for "no sort applied — use canonical order")
```rust
/// Canonical file order — no reordering applied. Used as the cycle start/end.
FileOrder,
```
`compare()` arm: `SortOrder::FileOrder => Ordering::Equal` (stable sort preserves order).

**New variant: `CompletedDate`**
```rust
/// Earliest completion date first. Tasks with no completion date sort last.
CompletedDate,
```
`compare()` arm:
```rust
SortOrder::CompletedDate => {
    match (a.completion_date, b.completion_date) {
        (None, None)   => Ordering::Equal,
        (None, _)      => Ordering::Greater, // None sorts last
        (_, None)      => Ordering::Less,
        (Some(da), Some(db)) => da.cmp(&db),
    }
}
```

**New variant: `CreationDate`**
```rust
/// Earliest creation date first. Tasks with no creation date sort last.
CreationDate,
```
`compare()` arm:
```rust
SortOrder::CreationDate => {
    match (a.creation_date, b.creation_date) {
        (None, None)   => Ordering::Equal,
        (None, _)      => Ordering::Greater,
        (_, None)      => Ordering::Less,
        (Some(da), Some(db)) => da.cmp(&db),
    }
}
```

**Task fields used:** `completion_date: Option<NaiveDate>` and `creation_date: Option<NaiveDate>` — both already exist on `Task`.

**Sort cycle order (D-09):**
```
FileOrder → Alphabetical → CompletedDate → Context → DueDate → CreationDate → Priority → Project → FileOrder
```
Implement as:
```rust
fn next_sort(current: SortOrder) -> SortOrder {
    match current {
        SortOrder::FileOrder     => SortOrder::Alphabetical,
        SortOrder::Alphabetical  => SortOrder::CompletedDate,
        SortOrder::CompletedDate => SortOrder::Context,
        SortOrder::Context       => SortOrder::DueDate,
        SortOrder::DueDate       => SortOrder::CreationDate,
        SortOrder::CreationDate  => SortOrder::Priority,
        SortOrder::Priority      => SortOrder::Project,
        SortOrder::Project       => SortOrder::FileOrder,
    }
}
```

---

### Filter Panel Layout

**Constraints (D-01, D-02):**
```rust
let preset_count = self.filtering_state.presets.len();
let panel_height = 1 + (preset_count as u16).min(5);  // 1 input + up to 5 preset rows

let chunks = Layout::vertical([Min(0), Length(panel_height)]).split(frame.area());
// chunks[0] = task list area
// chunks[1] = filter panel area
```

**Panel internal layout:**
```rust
// Row 0 of panel = text input (tui-textarea)
let input_rect = Rect { y: chunks[1].y, height: 1, ..chunks[1] };

// Rows 1.. = preset list
let preset_rect = Rect {
    y: chunks[1].y + 1,
    height: chunks[1].height.saturating_sub(1),
    ..chunks[1]
};
```

**Text input rendering:**
```rust
// FilteringState holds `editor: TextArea<'static>`
frame.render_widget(&self.filtering_state.editor, input_rect);
```
No border on the input row — it consumes exactly 1 row.

**Preset list rendering:**
```rust
let items: Vec<ListItem> = self.filtering_state.presets.iter().enumerate()
    .map(|(i, (name, query))| {
        ListItem::new(format!("{}. {} — {}", i + 1, name, query))
    })
    .collect();

let list = List::new(items)
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

let mut list_state = ListState::default();
if !self.filtering_state.presets.is_empty() {
    list_state = list_state.with_selected(Some(self.filtering_state.selected_preset));
}

frame.render_stateful_widget(list, preset_rect, &mut list_state);
```

**`AppMode::Filtering` branch in `draw()`:**
```rust
AppMode::Filtering => {
    let preset_count = self.filtering_state.presets.len();
    let panel_height = 1 + (preset_count as u16).min(5);
    let chunks = Layout::vertical([Min(0), Length(panel_height)]).split(frame.area());
    self.render_task_list(frame, chunks[0]);
    self.render_filter_panel(frame, chunks[1]);
    // No separate status bar row — filter panel serves as the bottom chrome
}
```
When filter is open, the status bar is replaced by the filter panel. The key hint row is omitted; the input field itself signals the mode.

**`AppMode::Normal` branch** still renders status bar. When filter is *active but panel is closed*, the normal status bar shows the filter/sort info (D-13).

---

### Preset Support

**New structs in `crates/todotxt-tui/src/config.rs`:**
```rust
#[derive(Debug, Deserialize, Default)]
pub struct TuiPreset {
    pub filter: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct TuiConfig {
    pub todo_file: Option<PathBuf>,
    pub done_file: Option<PathBuf>,
    #[serde(default)]
    pub auto_creation_date: bool,
    /// Named filter presets. Each entry: { filter = "query string" }
    #[serde(default)]
    pub presets: HashMap<String, TuiPreset>,
}
```

**TOML format (mirrors CLI `[presets]` table):**
```toml
[presets.work]
filter = "@work -DONE"

[presets.today]
filter = "due:today"
```

**Stable 1-9 ordering:** `HashMap` does not preserve insertion order. Sort preset names alphabetically before storing in `App`:
```rust
let mut sorted_presets: Vec<(String, String)> = config.presets
    .into_iter()
    .filter_map(|(name, p)| p.filter.map(|q| (name, q)))
    .collect();
sorted_presets.sort_by(|(a, _), (b, _)| a.cmp(b));
// Only first 9 are addressable by number keys; all are navigable by arrow keys
```

**`FilteringState` struct on `App`:**
```rust
pub struct FilteringState {
    pub editor: TextArea<'static>,
    pub presets: Vec<(String, String)>,  // (name, query) sorted by name
    pub selected_preset: usize,           // highlight index in preset list
}
```

**`App::new()` signature change:**
```rust
pub fn new(task_list: TaskList, todo_path: PathBuf, presets: Vec<(String, String)>) -> Self
```
Caller (main.rs) loads config, builds sorted preset vec, passes to `App::new()`.

---

### Key Dispatch in Filtering Mode

**`handle_filtering_key()` dispatch table:**

| Key | Action |
|-----|--------|
| `Esc` | Clear `active_filter`, close panel (`mode = AppMode::Normal`), rebuild display_indices (now unfiltered), clamp selection |
| `Enter` | Close panel keeping filter active (`mode = AppMode::Normal`); display_indices already up to date |
| `Down` | `selected_preset = (selected_preset + 1).min(presets.len() - 1)`; load preset query into editor and into active_filter live; rebuild display_indices |
| `Up` | `selected_preset = selected_preset.saturating_sub(1)`; same live-load behavior |
| `Char('1')..='Char('9')` | `let idx = (c as usize - '1' as usize)`; if `idx < presets.len()`, load `presets[idx].1` into editor and active_filter; rebuild display_indices; `selected_preset = idx` |
| Any other key | `self.filtering_state.editor.input_without_shortcuts(Event::Key(key))`; read back `editor.lines()[0]` as new `active_filter`; rebuild display_indices |

**Opening the filter panel (`f` in Normal mode):**
```rust
KeyCode::Char('f') => {
    // Preserve any existing active_filter in the editor
    let mut ed = TextArea::default();
    if let Some(ref q) = self.active_filter {
        ed.insert_str(q);
    }
    self.filtering_state.editor = ed;
    self.mode = AppMode::Filtering;
}
```

**`o` key (sort cycle) in Normal mode:**
```rust
KeyCode::Char('o') => {
    self.sort_order = next_sort(self.sort_order);
    self.rebuild_display_indices();
    self.clamp_selection();
}
```

**Key blocking note:** In `Filtering` mode, all keys except `Esc`/`Enter`/`Down`/`Up`/digits are routed to `input_without_shortcuts()`. Keys `n`, `u`, `e`, `d`, `x`, `q` are absorbed by the text area — this is correct behavior (D-07 "text input captures them"). The `AppMode` check in `handle_event()` already ensures Normal-mode handlers never fire while Filtering.

---

### Status Bar Changes

**Current format (Normal mode, no active filter):**
```
{file} | {total} tasks | {visible} visible | {due_today} due today | {overdue} overdue
```

**New format (Normal mode, filter OR sort active):**
```
{file} | {visible}/{total} tasks | {filter_query} | sort: {sort_name}
```
Omit `| {filter_query}` segment when `active_filter` is None. Omit `| sort: {sort_name}` when `sort_order == SortOrder::FileOrder`.

**Sort name lookup:**
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
    }
}
```

**Truncation strategy:**
```rust
let width = frame.area().width as usize;
let left_max = width.min(width / 2 + 20).min(60); // generous left budget
let left = if left.len() > left_max { format!("{}…", &left[..left_max - 1]) } else { left };
```
Full right-side key hint string is appended with `"  "` separator; terminal clips naturally.

**`render_status_bar()` updated logic:**
```rust
let total = self.task_list.len();
let visible = self.display_indices.len();
let due_today = self.display_indices.iter()
    .filter(|&&i| { let t = &tasks[i]; !t.completed && t.due_status() == DueStatus::Today })
    .count();
let overdue = self.display_indices.iter()
    .filter(|&&i| { let t = &tasks[i]; !t.completed && t.due_status() == DueStatus::Overdue })
    .count();

let filter_is_active = self.active_filter.is_some();
let sort_is_active = self.sort_order != SortOrder::FileOrder;

let left = if filter_is_active || sort_is_active {
    let mut parts = vec![format!("{} | {}/{} tasks", file_name, visible, total)];
    if let Some(ref q) = self.active_filter { parts.push(q.clone()); }
    if sort_is_active { parts.push(format!("sort: {}", sort_name(self.sort_order))); }
    parts.join(" | ")
} else {
    format!("{} | {} tasks | {} visible | {} due today | {} overdue",
        file_name, total, visible, due_today, overdue)
};
```

---

## Architecture Patterns

**`FilteringState` as a nested struct** — Keeps filtering state cohesive and avoids bloating `App` fields:
```rust
pub struct FilteringState {
    pub editor: TextArea<'static>,
    pub presets: Vec<(String, String)>,
    pub selected_preset: usize,
}
```
`App` holds `pub filtering_state: FilteringState` (always present, initialized in `App::new()`).

**`active_filter: Option<String>` stays on `App` top level** (not inside `FilteringState`) because it affects `rebuild_display_indices()` which is called from many sites — avoids nested borrow gymnastics.

**`sort_order: SortOrder` stays on `App` top level** for the same reason.

**`display_indices` initialization order in `App::new()`:**
```rust
let mut app = App { ..., display_indices: Vec::new(), ... };
app.rebuild_display_indices(); // populates from empty filter + FileOrder
app
```

**`AppMode::Filtering` in existing dispatch:**
- `handle_event()` match arm: `AppMode::Filtering => self.handle_filtering_key(key)?`
- `draw()` match arm: `AppMode::Filtering => { ... render filter panel ... }`
- `apply_pending_reload()` — reload is permitted while in Filtering mode (panel is read-only relative to the file); just call `rebuild_display_indices()` after reload and clamp selection.

**`clamp_selection()` updated:**
```rust
fn clamp_selection(&mut self) {
    let count = self.display_indices.len();
    if count == 0 { self.selected = 0; } else { self.selected = self.selected.min(count - 1); }
}
```

---

## Common Pitfalls

1. **`TaskList::sort()` mutates canonical order — never call it for display.**
   - `sort()` reorders `self.tasks` in-place; all saved canonical indices become wrong.
   - Mitigation: sort `(idx, &Task)` pairs in `rebuild_display_indices()` only. `TaskList::sort()` is never called from the TUI.

2. **After `delete(canonical_idx)`: all indices above it shift down.**
   - Patching `display_indices` in-place is fragile. Always call `rebuild_display_indices()` from scratch after any mutation.
   - The `display_indices` rebuild is O(n) and imperceptibly fast for todo.txt sizes.

3. **After `add()`: new task is at `task_list.len()-1` canonically, but may appear at any display position after sort.**
   - Find the display row by scanning `display_indices` for the new canonical index after rebuild.
   - Do NOT assume `selected = display_indices.len() - 1`.

4. **`HashMap` preset ordering is non-deterministic.**
   - Sort preset names alphabetically immediately after loading from config.
   - The `Vec<(String, String)>` on `FilteringState` is the stable source of truth.

5. **`tui-textarea` absorbs `f` key in Filtering mode.**
   - This is intentional — the text area captures all printable characters.
   - Closing the panel requires `Esc` or `Enter` (not `f`). Document this in key hints.

6. **Selection validity after filter change.**
   - After rebuild, `selected` may be out of bounds (e.g. selected row 5 but only 3 tasks match filter).
   - Always call `clamp_selection()` immediately after `rebuild_display_indices()`.

7. **`pending_reload` while Filtering.**
   - Unlike edit mode (where a queued reload could overwrite in-progress edits), filtering is read-only.
   - Safe approach: allow immediate reload while in Filtering mode (same as Normal mode). The filter is re-applied over the fresh task list. No special guard needed.

8. **`Editing { original_idx }` — original_idx is canonical, not display.**
   - When saving an edit: `task_list.update(original_idx, ...)` is correct.
   - After rebuild, restore display selection by scanning `display_indices` for `original_idx`.
   - Do NOT use `original_idx` as a display row index directly.

9. **`FileOrder` variant is `#[non_exhaustive]`-safe but requires exhaustive match inside the crate.**
   - All `match sort_order` arms in `app.rs` and `sort.rs` must include `FileOrder`.
   - `compare()` for `FileOrder` returns `Ordering::Equal` (stable sort preserves order).

---

## Implementation Order

### Step 1 — Core data model (no visible behavior change)
1. Add `FileOrder`, `CompletedDate`, `CreationDate` to `SortOrder` in `todotxt-core/src/sort.rs`
2. Add `display_indices: Vec<usize>`, `sort_order: SortOrder`, `active_filter: Option<String>` to `App`
3. Implement `rebuild_display_indices()` on `App`
4. Update `App::new()` to call `rebuild_display_indices()`
5. Update `render_task_list()` to use `display_indices`
6. Update `clamp_selection()` to use `display_indices.len()`
7. Update `handle_normal_key()` navigation to use `display_indices.len()`
8. Update `toggle_done()`, `handle_delete_confirm_key()`, `save_and_exit()` to translate via `display_indices`
9. Wire `rebuild_display_indices()` into all mutation + reload paths
10. Compile + run: list display and all task actions must work identically to before

### Step 2 — Sort cycle + filter panel + key dispatch
1. Implement `next_sort()` helper in `app.rs`
2. Add `o` key in `handle_normal_key()` → sort cycle → rebuild
3. Add `FilteringState` struct; add `filtering_state` field to `App`
4. Add `AppMode::Filtering` variant
5. Add `f` key in `handle_normal_key()` to open panel
6. Implement `handle_filtering_key()` with full dispatch table
7. Add `AppMode::Filtering` arm to `draw()` calling `render_filter_panel()`
8. Implement `render_filter_panel()` with textarea + numbered preset list
9. Test: open panel, type query, see list narrow live; Esc clears; `o` cycles sort

### Step 3 — Status bar + preset config support
1. Extend `TuiConfig` with `TuiPreset` struct and `presets: HashMap<String, TuiPreset>`
2. Update `main.rs` to build sorted `Vec<(String, String)>` and pass to `App::new()`
3. Update `render_status_bar()` with new format strings and filter/sort display
4. Test: TOML preset loading, 1-9 number keys, arrow-key navigation, status bar display

# Phase 34 Research: Bulk Action Safety + Metadata Preservation

## Summary
Phase 34 adds (1) an affected-count preview gate before all high-impact bulk actions (`s`, `i`, `T`, `D`) when N > 1, (2) a new `i` priority picker overlay modeled on the existing `s` date picker with added type-to-jump behavior, and (3) structured Task-model mutation for all property setters so non-target metadata is never corrupted. The Task struct already has full builder methods (`with_priority`, `with_due_date`) that do safe round-trip mutation via `rebuild_raw` — raw string surgery is never needed. The `DatePickerState` + `render_date_picker_overlay` pattern is directly reusable for the priority picker with a different data source (A–Z list + "— no priority").

---

## Task Struct + Structured Mutation (D-13)

### Current State
Tasks are mutated via **builder methods** that update a single field, call the private `rebuild_raw()` function to produce a canonical string, then re-parse with `Task::parse()` to re-sync all fields. This is already the correct structured-mutation pattern. Phase 34 must use these builders — no raw whitespace-split token surgery.

### Fields Available
```rust
pub struct Task {
    raw: String,                        // private — canonical serialization
    pub completed: bool,
    pub priority: Option<char>,         // ← target for `i` setter
    pub creation_date: Option<NaiveDate>,
    pub completion_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,    // ← target for `s` setter
    pub threshold_date: Option<NaiveDate>,
    pub projects: Vec<String>,          // sorted, deduped, no `+`
    pub contexts: Vec<String>,          // sorted, deduped, no `@`
    pub body: String,                   // plain text with tags stripped
}
```

### Serialization
- `task.to_raw()` → `&str` — the canonical line (what gets written to file).
- `rebuild_raw(&task)` (private, in task.rs module) — reconstructs the full todo.txt line in canonical order: `x <completion_date> (<priority>) <creation_date> <body> +proj @ctx due:DATE t:DATE`.
- `Task::parse(line)` — infallible; re-syncs all fields from the raw string.
- `normalize_line(text)` — like `Task::parse` but also lifts an inline `(X)` priority token from body to `priority` field. Used for user-typed edit/append flows; setter mutations do NOT need it — they call `with_priority`/`with_due_date` directly, which internally call `rebuild_raw` + `Task::parse`.

### Builder Methods (D-13 ready)
```rust
task.clone().with_priority(Some('A'))    // sets priority; None clears it
task.clone().with_due_date(Some(date))   // sets due:YYYY-MM-DD; None removes it
task.clone().with_creation_date(...)
task.clone().with_threshold_date(...)
task.clone().with_completed(bool)        // strips priority on complete per spec
```
All are value-consuming; return a new `Task` with `raw` rebuilt and all fields re-parsed.

### Mutation Strategy
- For the `s` setter: `let new_task = task.clone().with_due_date(Some(chosen_date));`
- For the `i` setter: `let new_task = task.clone().with_priority(chosen_priority);` (pass `None` for "clear priority")
- Then replace in task list via `batch_update`. No manual string manipulation needed.

### Duplicate Tag Prevention
`rebuild_raw` builds tags from struct fields (BTreeSet-deduplicated for projects/contexts). Setting `due_date` replaces the old value — there is no risk of duplicate `due:` tokens because the tag is derived from the struct field, not appended to the raw string.

---

## `s` Date Picker — Reuse Pattern for `i` Priority Picker

### DatePickerState
Defined in `crates/todotxt-tui/src/state.rs`:
```rust
pub struct DatePickerState {
    pub month_year: String,          // e.g., "2026-07"
    pub selected_day: Option<u32>,   // currently highlighted day index
    pub suggestions: Vec<String>,    // formatted as "01 Mon", "02 Tue", etc.
    pub focused: bool,
}
```
Methods: `select_next()`, `select_prev()` (wrap at boundaries).

### AppMode Variant
`AppMode::DatePicker` — set on entry, checked in main event dispatch loop. The picker state lives in `App::date_picker: Option<DatePickerState>`.

### Key Handler Pattern (`handle_date_picker_key`)
- `Up`/`Down` → `date_picker.select_prev()` / `select_next()`
- `Tab`/`Enter` → accept selected item, apply `with_due_date()` to target tasks, `batch_update` + `rebuild_and_reanchor`, `mode = AppMode::Normal`
- `Esc` → cancel, `date_picker = None`, `mode = AppMode::Normal`, **selection preserved** (D-03)
- Month navigation (`[`/`]`) to change `month_year` and regenerate suggestions
- Count appears in overlay header text when N > 1 (D-05)

### Rendering
`render_date_picker_overlay()` — overlay widget rendered in the draw cycle when `app.date_picker.is_some()`. Pattern: enter mode → render overlay each frame → handle nav keys → Tab/Enter accepts → Esc cancels. Overlay sits on top of the normal list view.

### Adaptation Required for `i` Priority Picker
| Aspect | `s` Date Picker | `i` Priority Picker |
|--------|----------------|---------------------|
| AppMode | `DatePicker` | **`PriorityPicker`** (new) |
| State type | `DatePickerState` | **`PriorityPickerState`** (new) |
| Data source | Day suggestions for a month | Static `["A", "B", ..., "Z", "— (no priority)"]` |
| Month nav | Yes | No |
| Type-to-jump | No | **Yes** — typing A–Z jumps to that letter (D-09) |
| Accept action | `with_due_date(Some(date))` | `with_priority(Some(ch))` or `with_priority(None)` |
| Count header | "Setting due date — N tasks" | "Setting priority — N tasks" |

A lightweight dedicated struct is cleaner than repurposing `DatePickerState`:
```rust
pub struct PriorityPickerState {
    pub items: Vec<String>,         // "A", "B", ..., "Z", "— (no priority)"
    pub selected_idx: usize,
    pub type_filter: Option<char>,  // last typed letter (for jump, cleared on accept)
}
impl PriorityPickerState {
    pub fn select_next(&mut self) { /* wrap */ }
    pub fn select_prev(&mut self) { /* wrap */ }
    pub fn jump_to(&mut self, ch: char) { /* find index of ch in items */ }
}
```

---

## Bulk Mutation Patterns

### apply_token_to_tasks
Idempotent token application that already implements descending-index targeting. Used by `@`/`+` quick setters. For `s` and `i`, bulk mutation goes through a separate path: collect target task indices → map `with_due_date`/`with_priority` → `batch_update`.

### Targeting Logic
- If `self.selected_tasks` is non-empty → bulk targets those tasks (D-10 for `i`, same as `s`)
- Else → targets the cursor task (single-task path, no count preview per D-02)
- `selected_tasks` is a `HashSet<usize>` of canonical task indices

### batch_update + rebuild_and_reanchor
- `self.task_list.batch_update(replacements: Vec<(usize, Task)>)` — applies a set of (canonical_idx, new_task) replacements atomically
- `rebuild_and_reanchor()` — rebuilds display rows, reapplies filter/sort/group, re-anchors cursor to the same task

### Descending-index ordering
D-17: bulk mutations use descending canonical index order. For `s` and `i` setters (which replace, not remove), descending order is required per D-17 contract for consistency. Pattern: `let mut indices: Vec<usize> = ...; indices.sort_unstable_by(|a, b| b.cmp(a));`

---

## AppMode Enum

### Current Variants (app.rs ~line 29)
```rust
pub enum AppMode {
    Normal,
    QuickSetter(char),
    Adding,
    Editing { original_idx: usize },
    PaneLabelEditing { pane_idx: usize },
    DeleteConfirm,
    Filtering,
    FilterDefining,
    AppendText,
    KeymapErrors,
    Help,
    DatePicker,
}
```
**AppMode is `Copy`** (confirmed at line ~563) — new variants must also be `Copy` (no non-Copy fields).

### New Variants Needed
```rust
PriorityPicker,
AppendTextConfirm,   // for T count banner before text entry (D-06)
```
`AppendTextConfirm` shows "Appending to N tasks — Enter to confirm, Esc to cancel", then transitions to `AppendText` on Enter. Needs a field `append_confirm_count: usize` on `App`.

---

## Count Preview Gate (D-01 through D-07)

### Current DeleteConfirm Pattern
`AppMode::DeleteConfirm` + `render_delete_confirm()`: panel shows count and "y/n" prompt. `handle_delete_confirm_key`: `y` → execute + `batch_update` + `rebuild_and_reanchor` + `mode = Normal`; `n`/`Esc` → `mode = Normal`, selection preserved.

### Integration for `s` and `i` (D-05)
Count is shown **inline in the picker overlay header** when N > 1. The picker opens regardless; the header text conditionally includes the count. No separate confirmation step — Enter applies directly.

### Integration for `T` (D-06)
New `AppMode::AppendTextConfirm` variant. In `handle_normal_key` for `T`:
```rust
let n = if self.selected_tasks.is_empty() { 1 } else { self.selected_tasks.len() };
if n > 1 {
    self.append_confirm_count = n;
    self.mode = AppMode::AppendTextConfirm;
} else {
    self.mode = AppMode::AppendText;
}
```

### Integration for `D` (D-07)
Wording update only to the existing `render_delete_confirm()` text — ensure count is clearly shown.

---

## handle_normal_key Dispatch

### Current Bindings (from grep, lines ~821/835/850/1017)
- `s` → line ~1017: `self.mode = AppMode::DatePicker` with target resolution
- `D` → line ~821/835: `self.mode = AppMode::DeleteConfirm`
- `T` → line ~850: `self.mode = AppMode::AppendText`

### Current `i` Binding
`i` is currently unbound (no `PriorityPicker` mode exists). It is free for Phase 34.

### Count Preview Gate Pattern
```rust
// Compute n at each entry point:
let n = if self.selected_tasks.is_empty() { 1 } else { self.selected_tasks.len() };
// s and i: n passed to picker state for header rendering — no gate
// T: gate on n > 1 → AppendTextConfirm vs AppendText
// D: existing DeleteConfirm already handles this
```

---

## Architectural Notes

### Component Reuse Strategy
- `render_date_picker_overlay()` → adapt for `render_priority_picker_overlay()` (same widget, different title and data source)
- `handle_date_picker_key()` → direct template for `handle_priority_picker_key()` (add type-to-jump branch)
- `AppendTextConfirm` → new minimal mode with `render_append_text_confirm()` that shows count banner

### New Files Needed
None. All changes in existing files:
- `crates/todotxt-tui/src/state.rs` — `PriorityPickerState` struct + `AppMode::PriorityPicker`, `AppMode::AppendTextConfirm`
- `crates/todotxt-tui/src/app.rs` — `handle_priority_picker_key`, count preview gate in `handle_normal_key`, wording update in `render_delete_confirm`, `append_confirm_count` field
- UI render file (app.rs or separate ui.rs) — `render_priority_picker_overlay`, `render_append_text_confirm`

### Key Risk Areas
1. **Metadata corruption**: if `s` currently uses raw string surgery, it must be refactored to `with_due_date()`.
2. **Duplicate `due:` tokens**: prevented by `rebuild_raw` — but only if builder methods are used.
3. **Cancel path clearing selection**: Esc in priority/date picker must NOT call `self.selected_tasks.clear()`.
4. **Completed task mutation (D-15)**: `with_priority()` on `completed=true` task — `rebuild_raw` places priority correctly after `x <date>` prefix.
5. **AppMode Copy constraint**: `PriorityPicker` must be `Copy` — no non-Copy fields.

---

## Validation Architecture

### Test Approach
- **Structured mutation round-trip**: `Task::parse("x 2026-01-01 (A) 2025-12-01 fix bug @work +proj due:2026-03-01 t:2026-02-01").with_due_date(Some(new_date))` — assert all fields unchanged except `due_date`, no duplicate tokens in `to_raw()`.
- **Priority setter on completed task**: parse completed task, call `with_priority(Some('B'))`, assert `completed == true`, `completion_date` preserved, `priority == Some('B')`.
- **Clear priority**: `with_priority(None)` — assert `to_raw()` has no `(X)` pattern.
- **Count preview**: 3 tasks in `selected_tasks`, press `i` → `mode == PriorityPicker`, overlay header contains "3".
- **Cancel preserves selection**: enter `PriorityPicker`, press Esc → `selected_tasks` unchanged, `mode == Normal`.
- **Descending bulk**: bulk `with_priority` across 3 tasks at indices `[0, 2, 4]` — all 3 updated correctly.

---

## Standard Stack
From Cargo.toml:
- `ratatui` ~0.26–0.28 — TUI framework
- `crossterm` — terminal backend and key events
- `tui-textarea` — text input widget (used for Adding/Editing modes)
- `chrono` — `NaiveDate`, `Local`
- `winnow` — parser combinator (task.rs parsing)

---

## Don't Hand-Roll
- **Task mutation**: use `Task::with_priority()` / `Task::with_due_date()` — they call `rebuild_raw` + re-parse correctly.
- **Duplicate tag prevention**: automatic via `rebuild_raw` deriving tags from struct fields.
- **Descending-index loop**: copy pattern from `apply_token_to_tasks` or `handle_delete_confirm_key`.
- **batch_update + rebuild_and_reanchor**: always use this pair.
- **Overlay rendering**: adapt `render_date_picker_overlay`, don't build from scratch.
- **Cancel path**: copy `Esc → mode = AppMode::Normal` from `handle_date_picker_key` — it preserves selection correctly.

---

## Common Pitfalls
1. **Raw string surgery** instead of `with_priority`/`with_due_date` — corrupts `x` prefix, dates, produces duplicate tokens.
2. **Forgetting `x` prefix on completed tasks** — `rebuild_raw` handles this; hand-rolled code forgets.
3. **Index shift bugs** — use `sort_unstable_by(|a,b| b.cmp(a))` on target indices before iterating.
4. **Clearing `selected_tasks` on Esc** — cancel must NOT clear selection (D-03).
5. **Case sensitivity in type-to-jump** — map `'a'`→`'A'` before jumping (D-09 says case-insensitive).
6. **AppMode Copy constraint** — ensure `PriorityPicker` and `AppendTextConfirm` have no non-Copy fields.
7. **Existing `s` setter may use raw surgery** — audit `handle_date_picker_key` accept branch before assuming it uses `with_due_date`.

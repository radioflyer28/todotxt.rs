# Phase 34 Pattern Map

## AppMode Enum (app.rs ~line 29)
```rust
#[derive(Clone, PartialEq, Debug, Copy)]
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
    // Phase 34 adds: PriorityPicker, AppendTextConfirm
}
```
**Note:** `AppMode` is `Copy` — new variants must have no non-Copy fields.

---

## DatePickerState (state.rs ~line 151) — Analog for PriorityPickerState

```rust
pub struct DatePickerState {
    pub month_year: String,
    pub selected_day: Option<u32>,
    pub suggestions: Vec<String>,
    pub focused: bool,
}

impl DatePickerState {
    pub fn select_next(&mut self) {
        if let Some(idx) = self.selected_day {
            if (idx as usize) < self.suggestions.len().saturating_sub(1) {
                self.selected_day = Some(idx + 1);
            }
        }
    }
    pub fn select_prev(&mut self) {
        if let Some(idx) = self.selected_day {
            if idx > 0 {
                self.selected_day = Some(idx - 1);
            }
        }
    }
}
```

---

## handle_date_picker_key (app.rs ~line 1882) — Template for handle_priority_picker_key

Key structure (paraphrased from verbatim read):
```rust
fn handle_date_picker_key(&mut self, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            self.date_picker = None;
            self.mode = AppMode::Normal;
            // NOTE: selected_tasks is NOT cleared on Esc (D-03 cancel preservation)
        }
        KeyCode::Down => {
            if let Some(ref mut dp) = self.date_picker {
                dp.select_next();
            }
        }
        KeyCode::Up => {
            if let Some(ref mut dp) = self.date_picker {
                dp.select_prev();
            }
        }
        KeyCode::Tab | KeyCode::Enter => {
            if let Some(ref dp) = self.date_picker {
                if let Some(selected_idx) = dp.selected_day {
                    let chosen_date = /* parse from dp.suggestions[selected_idx] */;
                    // Build target list — descending order (D-17)
                    let targets: Vec<usize> = if !self.selected_tasks.is_empty() {
                        let mut v: Vec<usize> = self.selected_tasks.iter().cloned().collect();
                        v.sort_unstable_by(|a, b| b.cmp(a));
                        v
                    } else {
                        vec![self.active_canonical_selected()]
                    };
                    // IMPORTANT: existing code uses RAW STRING SURGERY here
                    // (strips due: token via split_whitespace().filter(), appends new token)
                    // Phase 34 D-13 must REFACTOR this to use with_due_date()
                    let replacements: Vec<(usize, Task)> = targets.iter().map(|&idx| {
                        let task = self.task_list.get(idx).clone();
                        (idx, task.with_due_date(Some(chosen_date)))
                    }).collect();
                    self.task_list.batch_update(replacements);
                    self.rebuild_and_reanchor();
                    // selection cleared on accept (only on Enter/Tab, not Esc)
                    self.selected_tasks.clear();
                    self.disjoint_select = false;
                }
            }
            self.date_picker = None;
            self.mode = AppMode::Normal;
        }
        _ => {}
    }
}
```

**CRITICAL FINDING:** The existing `handle_date_picker_key` accept branch uses **raw string surgery** (`split_whitespace().filter()` to strip old `due:` token, then appends new one). Phase 34 D-13 requires refactoring this to `with_due_date()`. This is a dual responsibility: fix `s` + implement `i` correctly.

---

## render_date_picker_overlay (app.rs ~line 3147) — Template for render_priority_picker_overlay

```rust
fn render_date_picker_overlay(&self, f: &mut Frame, area: Rect) {
    if let Some(ref dp) = self.date_picker {
        if dp.suggestions.is_empty() { return; }
        let popup_height = (dp.suggestions.len() as u16).min(10) + 2; // +2 for borders
        let popup_width = 22u16;
        // Position above footer — compute centered Rect
        let popup_rect = /* centered overlay Rect */;
        // Render List widget
        let items: Vec<ListItem> = dp.suggestions.iter()
            .map(|s| ListItem::new(s.as_str()))
            .collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Due date"))
            .highlight_style(if dp.focused {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            });
        let mut state = ListState::default();
        state.select(dp.selected_day.map(|d| d as usize));
        f.render_stateful_widget(list, popup_rect, &mut state);
    }
}
```

For `render_priority_picker_overlay`: same structure, title becomes `"Priority — N tasks"` (when N > 1) or `"Priority"`, data source is A–Z + "— (no priority)".

---

## handle_normal_key — s/D/T Entry Points (app.rs ~lines 821, 850, 1007–1017)

```rust
// D — bulk delete (line ~821/835)
KeyCode::Char('D') => {
    // guard: display_count > 0
    self.mode = AppMode::DeleteConfirm;
}

// T — bulk append (line ~850)
KeyCode::Char('T') => {
    if !self.selected_tasks.is_empty() {
        self.mode = AppMode::AppendText;
        // Phase 34 D-06: if selected_tasks.len() > 1, use AppendTextConfirm instead
    }
}

// s — due-date picker (line ~1007–1017)
KeyCode::Char('s') => {
    if self.display_count > 0 {
        // initialize DatePickerState with suggestions for current month
        let dp = DatePickerState { /* ... */ };
        self.date_picker = Some(dp);
        self.mode = AppMode::DatePicker;
        // Phase 34 D-05: count (selected_tasks.len() or 1) embedded in header
    }
}

// i — priority picker (Phase 34 NEW)
// Currently unbound — no existing handler
```

---

## handle_delete_confirm_key (app.rs ~line 1762) — Template for AppendTextConfirm handler

```rust
fn handle_delete_confirm_key(&mut self, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') => {
            if self.selected_tasks.is_empty() {
                // single-task delete
                if let Some(idx) = self.active_canonical_selected_opt() {
                    self.task_list.delete(idx);
                }
            } else {
                // bulk delete — descending index order (D-03 from Phase 20)
                let mut indices: Vec<usize> = self.selected_tasks.iter().cloned().collect();
                indices.sort_unstable_by(|a, b| b.cmp(a));
                for idx in indices {
                    self.task_list.delete(idx);
                }
                self.selected_tasks.clear();
                self.disjoint_select = false;
            }
            self.rebuild_and_reanchor();
        }
        _ => {
            // Any non-y key cancels — selection is preserved (D-03)
        }
    }
    self.mode = AppMode::Normal;
    self.apply_pending_reload();
}
```

---

## Task Builder Methods (task.rs ~lines 124–180, 444) — D-13 Structured Mutation

```rust
pub fn with_priority(mut self, priority: Option<char>) -> Self {
    self.priority = priority;
    self.raw = rebuild_raw(&self);
    // then re-sync: self = Task::parse(&self.raw); but via internal rebuild
    self
}

pub fn with_due_date(mut self, date: Option<NaiveDate>) -> Self {
    self.due_date = date;
    self.raw = rebuild_raw(&self);
    self
}

// rebuild_raw assembles in canonical order:
// "x " (if completed) + completion_date + priority "(X) " + creation_date + body
// + " +proj" * n + " @ctx" * n + " due:YYYY-MM-DD" (if Some) + " t:YYYY-MM-DD" (if Some)
fn rebuild_raw(task: &Task) -> String {
    let mut parts: Vec<String> = vec![];
    if task.completed { parts.push("x".into()); }
    if let Some(cd) = task.completion_date { parts.push(cd.to_string()); }
    if let Some(p) = task.priority { parts.push(format!("({})", p)); }
    if let Some(cr) = task.creation_date { parts.push(cr.to_string()); }
    parts.push(task.body.clone());
    for proj in &task.projects { parts.push(format!("+{}", proj)); }
    for ctx in &task.contexts { parts.push(format!("@{}", ctx)); }
    if let Some(d) = task.due_date { parts.push(format!("due:{}", d)); }
    if let Some(t) = task.threshold_date { parts.push(format!("t:{}", t)); }
    parts.join(" ").trim_end().to_string()
}
```

---

## Key Patterns Summary

1. **New AppMode variants**: Add `PriorityPicker` and `AppendTextConfirm` to the `AppMode` enum in `app.rs`. Both must be `Copy` (no non-Copy fields). Add dispatch in the central `match self.mode` block (~line 563).

2. **PriorityPickerState mirrors DatePickerState**: Create `PriorityPickerState { items: Vec<String>, selected_idx: usize, focused: bool }` in `state.rs`. Items = `["A".."Z"] + ["— (no priority)"]`. `select_next`/`select_prev` wrap at boundaries.

3. **handle_priority_picker_key adapts handle_date_picker_key**: Esc cancels (preserve selection), Up/Down navigate, Enter/Tab accepts. Add type-to-jump branch: `KeyCode::Char(ch) if ch.is_alphabetic()` → `priority_picker.jump_to(ch.to_ascii_uppercase())`. On accept: use `task.with_priority(Some(ch))` or `with_priority(None)` for "no priority" — **not raw string surgery**.

4. **Refactor existing `s` setter to use `with_due_date()`**: The current `handle_date_picker_key` accept branch uses raw string surgery. Phase 34 D-13 requires replacing it with `task.clone().with_due_date(Some(chosen_date))`.

5. **AppendTextConfirm gate for T**: In `handle_normal_key` for `T`, gate on `selected_tasks.len() > 1` → set `self.append_confirm_count = n; self.mode = AppMode::AppendTextConfirm`. In `handle_append_text_confirm_key`: Enter → `self.mode = AppMode::AppendText`, Esc → `Normal` (preserve selection).

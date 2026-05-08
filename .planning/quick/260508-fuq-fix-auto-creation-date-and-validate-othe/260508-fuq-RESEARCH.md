# Quick Task 260508-fuq: fix auto_creation_date — Research

**Researched:** 2026-05-08
**Scope:** Targeted — fix site, API verification, config wire-up audit, test patterns

---

## 1. Bug Confirmed — `auto_creation_date` Is Never Read in `save_and_exit()`

`TuiConfig.auto_creation_date` exists and deserializes correctly (`config.rs` line 139,
`#[serde(default)]` → `false` when absent from TOML).

`save_and_exit()` `AppMode::Adding` arm (`app.rs` lines 3007–3023) calls `Task::parse(&text)`
then immediately calls `task_list.add(task)` **without consulting `self.config.auto_creation_date`
at all**. No other code path injects a creation date for the Adding flow.

**Exact fix location:** `app.rs` line 3009, after `let task = Task::parse(&text);` and before
`self.push_undo_entry()`.

```rust
// AppMode::Adding arm, after Task::parse:
let task = Task::parse(&text);
// ↓ INSERT HERE
let task = if self.config.auto_creation_date && task.creation_date.is_none() {
    task.with_creation_date(Some(Local::now().date_naive()))
} else {
    task
};
// ↑ END INSERT
self.push_undo_entry();
self.task_list.add(task)...
```

`Local` is already imported: `app.rs` line 11 — `use chrono::{Local, Datelike};`.
No new imports needed.

---

## 2. `Task::with_creation_date` Is the Correct API

`task.rs` line 157:
```rust
pub fn with_creation_date(self, date: Option<NaiveDate>) -> Self {
    let new_task = Task { creation_date: date, ..self };
    let new_raw = rebuild_raw(&new_task);
    Task::parse(&new_raw)
}
```
- Builder pattern (consumes self, returns new Task).
- Calls `rebuild_raw` + `Task::parse` round-trip — canonical serialisation guaranteed.
- Passing `Some(date)` sets the date; passing `None` clears it.
- **No other date-stamping utility exists** in the TUI codebase for this purpose.

**Guard required:** only inject when `task.creation_date.is_none()` (user may type `2026-06-01 buy milk`; preserve their explicit date).

---

## 3. `normalize_append` and `normalize_edit` Are Correctly Wired

### `normalize_append` — `app.rs` line 2724
```rust
let new_task = if self.config.normalize_append {
    normalize_append(t, &text)   // parse-then-merge strategy
} else {
    // raw concat fallback
```
Config field read directly from `self.config`. No bypass path exists.
`TuiConfig.normalize_append` has `#[serde(default = "default_true")]` → defaults `true`.

### `normalize_edit` — `app.rs` lines 3026–3033
```rust
let task = if self.config.normalize_edit {
    normalize_line(&text)
} else {
    Task::parse(&text)
};
```
Config field read directly from `self.config`. No bypass path exists.
`TuiConfig.normalize_edit` has `#[serde(default = "default_true")]` → defaults `true`.

**Verdict:** Both fields are live. No gap in config plumbing for these two.

---

## 4. Test Coverage Audit

### `auto_creation_date` — **zero tests in app.rs**
No existing test verifies that a task added via `save_and_exit()` in `AppMode::Adding` gets a
creation date when `auto_creation_date = true`.

### `normalize_edit` — **zero behavioral tests in app.rs**
The `normalize_edit` code path exists and reads from config, but no test in app.rs exercises
`normalize_edit = false` to confirm it skips `normalize_line`.

### Closest existing test pattern — `add_undo_round_trip` (app.rs line 6115)
```rust
fn add_undo_round_trip() {
    let mut app = make_app_with_tasks(&["task A", "task B"]);
    app.mode = AppMode::Adding;
    app.editor = { let mut ta = tui_textarea::TextArea::default(); ta.insert_str("new task"); ta };
    app.save_and_exit().unwrap();
    assert_eq!(app.task_list.tasks().len(), original_count + 1, "task should be added");
    ...
}
```

For config-driven tests, use `make_app_with_config` (app.rs line 6654):
```rust
fn make_app_with_config(task_lines: &[&str], config: TuiConfig) -> App { ... }
```

### New tests to write (3 total)

**T-ACD-01** — `auto_creation_date = true` injects today's date:
```rust
#[test]
fn save_and_exit_adding_injects_creation_date_when_enabled() {
    let mut cfg = TuiConfig::default();
    cfg.auto_creation_date = true;
    let mut app = make_app_with_config(&[], cfg);
    app.mode = AppMode::Adding;
    app.editor = { let mut ta = tui_textarea::TextArea::default(); ta.insert_str("buy milk"); ta };
    app.save_and_exit().unwrap();
    let task = &app.task_list.tasks()[0];
    assert_eq!(task.creation_date, Some(Local::now().date_naive()));
}
```

**T-ACD-02** — `auto_creation_date = true` does NOT overwrite user-typed date:
```rust
#[test]
fn save_and_exit_adding_preserves_explicit_creation_date() {
    let mut cfg = TuiConfig::default();
    cfg.auto_creation_date = true;
    let mut app = make_app_with_config(&[], cfg);
    app.mode = AppMode::Adding;
    app.editor = { let mut ta = tui_textarea::TextArea::default(); ta.insert_str("2026-06-01 buy milk"); ta };
    app.save_and_exit().unwrap();
    let task = &app.task_list.tasks()[0];
    use chrono::NaiveDate;
    assert_eq!(task.creation_date, Some(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()));
}
```

**T-ACD-03** — `auto_creation_date = false` (default) leaves creation_date as None:
```rust
#[test]
fn save_and_exit_adding_no_date_when_disabled() {
    // TuiConfig::default() has auto_creation_date = false
    let mut app = make_app_with_tasks(&[]);
    app.mode = AppMode::Adding;
    app.editor = { let mut ta = tui_textarea::TextArea::default(); ta.insert_str("buy milk"); ta };
    app.save_and_exit().unwrap();
    let task = &app.task_list.tasks()[0];
    assert!(task.creation_date.is_none());
}
```

---

## 5. Default / Backward Compatibility

| Field | `#[serde(default)]` helper | Default value | Impact |
|---|---|---|---|
| `auto_creation_date` | built-in `Default` (bool) | `false` | Existing configs with no entry get no date injection — safe |
| `normalize_append` | `default_true()` | `true` | Existing behavior preserved |
| `normalize_edit` | `default_true()` | `true` | Existing behavior preserved |

---

## Summary

| Finding | Actionable |
|---|---|
| `auto_creation_date` never read in `save_and_exit()` | Insert 4-line guard after `Task::parse` in `AppMode::Adding` arm |
| `Task::with_creation_date` is correct API | Use `task.with_creation_date(Some(Local::now().date_naive()))` |
| `Local` already imported | No new imports |
| `normalize_append` / `normalize_edit` already wired | No fix needed; add behavioral tests if desired |
| 3 tests to add for `auto_creation_date` | See T-ACD-01/02/03 patterns above |

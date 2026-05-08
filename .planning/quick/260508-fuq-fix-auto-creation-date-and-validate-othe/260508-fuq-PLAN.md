---
phase: 260508-fuq
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/todotxt-tui/src/app.rs
autonomous: true
requirements:
  - FUQ-01-auto-creation-date-fix
  - FUQ-02-auto-creation-date-tests
  - FUQ-03-normalize-append-edit-tests

must_haves:
  truths:
    - "When auto_creation_date = true, a task added via the TUI Adding mode receives today's date if the user did not type one"
    - "When auto_creation_date = true, a user-typed date is preserved (no override)"
    - "When auto_creation_date = false (default), no date is injected"
    - "All three auto_creation_date behaviors are proven by automated tests"
    - "normalize_edit = true lifts inline (A) priority token to priority field via save_and_exit Editing arm"
    - "normalize_edit = false leaves inline (A) in body (Task::parse only)"
    - "normalize_append = true merges +project token into task's projects field via handle_append_text_key"
    - "normalize_append = false raw-concatenates append text without field merging"
  artifacts:
    - path: "crates/todotxt-tui/src/app.rs"
      provides: "Bug fix in save_and_exit() AppMode::Adding arm + three new test functions"
      contains: "auto_creation_date"
  key_links:
    - from: "save_and_exit() AppMode::Adding arm"
      to: "self.config.auto_creation_date"
      via: "if guard after Task::parse"
      pattern: "auto_creation_date && task\\.creation_date\\.is_none"
    - from: "save_and_exit() AppMode::Adding arm"
      to: "Task::with_creation_date"
      via: "builder call"
      pattern: "with_creation_date\\(Some\\(Local::now"
---

<objective>
Fix `auto_creation_date` config option having no effect when adding tasks in TUI, and add tests
proving all three behavioral cases.

Purpose: Users who set `auto_creation_date = true` in config.toml expect the TUI to stamp new
tasks with today's date automatically; currently the field is parsed but never read in the add
path.

Output: Patched `save_and_exit()` Adding arm + three test functions in `app.rs`.
</objective>

<execution_context>
@~/.copilot/get-shit-done/workflows/execute-plan.md
@~/.copilot/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/quick/260508-fuq-fix-auto-creation-date-and-validate-othe/260508-fuq-CONTEXT.md
@.planning/quick/260508-fuq-fix-auto-creation-date-and-validate-othe/260508-fuq-RESEARCH.md

<interfaces>
<!-- From crates/todotxt-tui/src/app.rs lines 3003–3023 — current Adding arm before fix -->
```rust
fn save_and_exit(&mut self) -> color_eyre::Result<()> {
    let text = self.editor.lines().first().cloned().unwrap_or_default();
    let mode = self.mode; // Copy
    match mode {
        AppMode::Adding => {
            // T-21-07: Adding always uses Task::parse — normalize_edit does not apply here.
            let task = Task::parse(&text);
            self.push_undo_entry();
            self.task_list
                .add(task)
                .map_err(|e| color_eyre::eyre::eyre!("Failed to add task: {}", e))?;
```

<!-- From crates/todotxt-core/src/task.rs line 157 -->
```rust
pub fn with_creation_date(self, date: Option<NaiveDate>) -> Self {
    // Builder: consumes self, sets creation_date, rebuilds raw via round-trip parse.
}
```

<!-- From crates/todotxt-tui/src/config.rs lines 139, 144, 149 -->
```rust
pub auto_creation_date: bool,   // #[serde(default)] → false
pub normalize_append: bool,     // #[serde(default = "default_true")] → true
pub normalize_edit: bool,       // #[serde(default = "default_true")] → true
```

<!-- Test helpers in app.rs -->
```rust
fn make_app_with_tasks(task_lines: &[&str]) -> App { ... }
fn make_app_with_config(task_lines: &[&str], config: TuiConfig) -> App { ... }
```

<!-- `Local` is already imported at app.rs line 11 -->
```rust
use chrono::{Local, Datelike};
```
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Inject creation date in AppMode::Adding when auto_creation_date is enabled</name>
  <files>crates/todotxt-tui/src/app.rs</files>
  <behavior>
    - With `auto_creation_date = true` and input `"buy milk"` (no date) → `task.creation_date == Some(today)`
    - With `auto_creation_date = true` and input `"2026-06-01 buy milk"` → `task.creation_date == Some(2026-06-01)` (unchanged)
    - With `auto_creation_date = false` (default) and input `"buy milk"` → `task.creation_date == None`
  </behavior>
  <action>
In `save_and_exit()` `AppMode::Adding` arm, after `let task = Task::parse(&text);` (currently
line 3009) and before `self.push_undo_entry();`, insert the following guard:

```rust
let task = if self.config.auto_creation_date && task.creation_date.is_none() {
    task.with_creation_date(Some(Local::now().date_naive()))
} else {
    task
};
```

This uses the already-imported `Local` from chrono and the existing `Task::with_creation_date`
builder. No imports needed. Do NOT apply this guard to `AppMode::Editing` or any paste path
(per D-12 in CONTEXT.md).

After the guard the arm continues unchanged:
```rust
self.push_undo_entry();
self.task_list.add(task)...
```
  </action>
  <verify>
    <automated>cd crates/todotxt-tui &amp;&amp; cargo test --lib save_and_exit_adding 2>&amp;1</automated>
  </verify>
  <done>
Three new tests (T-ACD-01/02/03 from Task 2) all pass. The guard is present in app.rs between
`Task::parse` and `push_undo_entry` in the Adding arm.
  </done>
</task>

<task type="auto">
  <name>Task 2: Add three tests verifying auto_creation_date behavior</name>
  <files>crates/todotxt-tui/src/app.rs</files>
  <action>
Append the following three tests inside the existing `#[cfg(test)]` module in `app.rs`,
grouped together with a section comment. Place them near the existing `add_undo_round_trip`
test (around line 6115) for locality.

```rust
// ── Quick 260508-fuq: auto_creation_date tests ──────────────────────────

#[test]
fn save_and_exit_adding_injects_creation_date_when_enabled() {
    // T-ACD-01: auto_creation_date = true, no date in input → today's date injected.
    let mut cfg = TuiConfig::default();
    cfg.auto_creation_date = true;
    let mut app = make_app_with_config(&[], cfg);
    app.mode = AppMode::Adding;
    app.editor = {
        let mut ta = tui_textarea::TextArea::default();
        ta.insert_str("buy milk");
        ta
    };
    app.save_and_exit().unwrap();
    let task = &app.task_list.tasks()[0];
    assert_eq!(
        task.creation_date,
        Some(Local::now().date_naive()),
        "T-ACD-01: creation_date should be today when auto_creation_date=true"
    );
}

#[test]
fn save_and_exit_adding_preserves_explicit_creation_date() {
    // T-ACD-02: auto_creation_date = true, user typed a date → preserve user date.
    let mut cfg = TuiConfig::default();
    cfg.auto_creation_date = true;
    let mut app = make_app_with_config(&[], cfg);
    app.mode = AppMode::Adding;
    app.editor = {
        let mut ta = tui_textarea::TextArea::default();
        ta.insert_str("2026-06-01 buy milk");
        ta
    };
    app.save_and_exit().unwrap();
    let task = &app.task_list.tasks()[0];
    assert_eq!(
        task.creation_date,
        Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()),
        "T-ACD-02: user-typed date must not be overwritten"
    );
}

#[test]
fn save_and_exit_adding_no_date_when_disabled() {
    // T-ACD-03: auto_creation_date = false (default) → no date injected.
    let mut app = make_app_with_tasks(&[]);
    app.mode = AppMode::Adding;
    app.editor = {
        let mut ta = tui_textarea::TextArea::default();
        ta.insert_str("buy milk");
        ta
    };
    app.save_and_exit().unwrap();
    let task = &app.task_list.tasks()[0];
    assert!(
        task.creation_date.is_none(),
        "T-ACD-03: creation_date must be None when auto_creation_date=false"
    );
}
```

No new imports required — `TuiConfig`, `AppMode`, `Local`, `tui_textarea::TextArea`,
`make_app_with_config`, `make_app_with_tasks` are all already in scope within the test module.
  </action>
  <verify>
    <automated>cd crates/todotxt-tui &amp;&amp; cargo test --lib save_and_exit_adding 2>&amp;1</automated>
  </verify>
  <done>
`cargo test --lib save_and_exit_adding` reports 3 tests passed:
- `save_and_exit_adding_injects_creation_date_when_enabled`
- `save_and_exit_adding_preserves_explicit_creation_date`
- `save_and_exit_adding_no_date_when_disabled`

Full lib test suite still passes: `cargo test --lib` exits 0.
  </done>
</task>

<task type="auto">
  <name>Task 3: Add behavioral tests for normalize_edit and normalize_append config options</name>
  <files>crates/todotxt-tui/src/app.rs</files>
  <action>
<!-- Discovery: grep of app.rs test section confirmed zero existing tests for normalize_edit or
     normalize_append (searched for fn.*normalize, normalize.*test, normalize_append, normalize_edit
     in #[cfg(test)] module — no matches). Adding four tests per CONTEXT.md: "add tests if missing,
     do not refactor working code". -->

Append the following four tests inside the `#[cfg(test)]` module in `app.rs`, grouped with a
section comment. Place them immediately after the Task 2 creation-date group:

```rust
// ── Quick 260508-fuq: normalize_edit / normalize_append tests ─────────────

#[test]
fn save_and_exit_editing_normalize_edit_true_lifts_inline_priority() {
    // T-NE-01: normalize_edit = true → "buy milk (A)" → priority field = Some('A').
    // normalize_line scans body for stray "(X)" and lifts it; Task::parse alone would leave it in body.
    let mut cfg = TuiConfig::default();
    cfg.normalize_edit = true;
    let mut app = make_app_with_config(&["buy milk"], cfg);
    app.mode = AppMode::Editing { original_idx: 0 };
    app.editor = {
        let mut ta = tui_textarea::TextArea::default();
        ta.insert_str("buy milk (A)");
        ta
    };
    app.save_and_exit().unwrap();
    let task = &app.task_list.tasks()[0];
    assert_eq!(
        task.priority,
        Some('A'),
        "T-NE-01: normalize_edit=true must lift inline (A) to priority field"
    );
    assert!(
        !task.body.contains("(A)"),
        "T-NE-01: (A) must be removed from body after lifting"
    );
}

#[test]
fn save_and_exit_editing_normalize_edit_false_keeps_inline_priority_in_body() {
    // T-NE-02: normalize_edit = false → Task::parse only; "(A)" stays in body, priority is None.
    let mut cfg = TuiConfig::default();
    cfg.normalize_edit = false;
    let mut app = make_app_with_config(&["buy milk"], cfg);
    app.mode = AppMode::Editing { original_idx: 0 };
    app.editor = {
        let mut ta = tui_textarea::TextArea::default();
        ta.insert_str("buy milk (A)");
        ta
    };
    app.save_and_exit().unwrap();
    let task = &app.task_list.tasks()[0];
    assert!(
        task.priority.is_none(),
        "T-NE-02: normalize_edit=false must not lift priority; got {:?}",
        task.priority
    );
}

#[test]
fn append_text_normalize_append_true_merges_project_token() {
    // T-NA-01: normalize_append = true → appending "+work" merges into task's projects field.
    use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
    let mut cfg = TuiConfig::default();
    cfg.normalize_append = true;
    let mut app = make_app_with_config(&["buy milk"], cfg);
    app.mode = AppMode::AppendText;
    app.selected_tasks.insert(0);
    app.editor = {
        let mut ta = tui_textarea::TextArea::default();
        ta.insert_str("+work");
        ta
    };
    app.handle_append_text_key(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    }).unwrap();
    let task = &app.task_list.tasks()[0];
    assert!(
        task.projects.iter().any(|p| p == "work"),
        "T-NA-01: normalize_append=true must add 'work' to projects; got {:?}",
        task.projects
    );
}

#[test]
fn append_text_normalize_append_false_raw_concatenates() {
    // T-NA-02: normalize_append = false → raw concat fallback; "+work" appended as-is to raw.
    use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
    let mut cfg = TuiConfig::default();
    cfg.normalize_append = false;
    let mut app = make_app_with_config(&["buy milk"], cfg);
    app.mode = AppMode::AppendText;
    app.selected_tasks.insert(0);
    app.editor = {
        let mut ta = tui_textarea::TextArea::default();
        ta.insert_str("+work");
        ta
    };
    app.handle_append_text_key(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::NONE,
    }).unwrap();
    let task = &app.task_list.tasks()[0];
    // Raw concat still ends up in raw; Task::parse may or may not populate projects
    // depending on parse behavior — what matters is the raw contains the text verbatim.
    assert!(
        task.to_raw().contains("+work"),
        "T-NA-02: normalize_append=false must append +work verbatim in raw; got '{}'",
        task.to_raw()
    );
}
```

No new imports required — `TuiConfig`, `AppMode`, `KeyCode`, `make_app_with_config` are all
already in scope within the test module.
  </action>
  <verify>
    <automated>cd crates/todotxt-tui &amp;&amp; cargo test --lib normalize 2>&amp;1</automated>
  </verify>
  <done>
`cargo test --lib normalize` reports 4 tests passed:
- `save_and_exit_editing_normalize_edit_true_lifts_inline_priority`
- `save_and_exit_editing_normalize_edit_false_keeps_inline_priority_in_body`
- `append_text_normalize_append_true_merges_project_token`
- `append_text_normalize_append_false_raw_concatenates`

Full lib suite still passes: `cargo test --lib` exits 0.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| user input → Task::parse | Text typed in TUI editor; trusted (local user) |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-fuq-01 | Tampering | creation_date injection | accept | Date sourced from `Local::now()` (system clock); local-only app, no external input vector |
</threat_model>

<verification>
Run full lib suite after both tasks:

```
cd crates/todotxt-tui && cargo test --lib
```

All existing tests must still pass. The three new `save_and_exit_adding_*` tests must pass.
</verification>

<success_criteria>
- `auto_creation_date = true` in config.toml causes new tasks added via TUI to receive today's date
- Explicit user-typed dates are never overwritten
- Default (`auto_creation_date = false`) injects nothing — backward compatible
- `cargo test --lib` exits 0 with 3 new tests passing
</success_criteria>

<output>
After completion, create `.planning/quick/260508-fuq-fix-auto-creation-date-and-validate-othe/260508-fuq-SUMMARY.md`
</output>

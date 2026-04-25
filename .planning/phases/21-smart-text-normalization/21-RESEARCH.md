# Phase 21: Smart Text Normalization — Research

**Researched:** 2026-04-25
**Domain:** Rust / todotxt-core API design, TUI config, CLI flag extension
**Confidence:** HIGH (all findings verified from codebase — no external dependencies)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** parse-then-merge strategy — parse appended text into tokens, merge into Task fields, rebuild via `rebuild_raw()`. Do NOT use append-then-reparse (naive concat absorbs tokens into body).
- **D-02:** Unknown/unrecognized tokens (e.g., `rec:+1w`, `foo:bar`) preserved verbatim in `body`. No `custom_fields` map needed.
- **D-03:** Appended priority wins — replaces original when both present.
- **D-04:** Appended priority with no original priority → token becomes the task's priority.
- **D-05:** Edit normalization is configurable via `normalize_edit` in `config.toml`. When disabled, edit saves via `Task::parse()` (current behavior).
- **D-06:** Default `normalize_edit = true`.
- **D-07:** Separate toggle `normalize_append = true/false`. Default: `true`.
- **D-08:** `normalize_append = false` falls back to Phase 20 raw concatenation. CLI `--normalize` is additive (explicit flag).
- **D-09:** CLI append `--normalize` flag calls `normalize_append()`. Without flag, existing raw-concat behavior unchanged.
- **D-10:** CLI does NOT read `normalize_append` from config automatically — explicit `--normalize` only.
- **D-11:** Shared helper: `pub fn normalize_append(task: &Task, append_text: &str) -> Task` in `todotxt-core`.
- **D-12:** Lives in `task.rs` or new `normalize.rs` — planner decides.
- **D-13:** Publicly exported from `todotxt-core`.

### Agent's Discretion

- Whether `normalize_append` lives in `task.rs` or a new `normalize.rs`.
- Default values for config flags when a `config.toml` section already defines normalization-adjacent settings — align with existing section conventions.
- How TUI reads new config toggles (flat fields vs. nested `[normalization]` table) — based on existing config structure.

### Deferred Ideas (OUT OF SCOPE)

- Phase 22: Keymap and help parity.
- Phase 23: Final UAT.
- `custom_fields` map on Task struct.
- CLI reading normalize config automatically.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| NORM-01 | Priority token in appended/edited text → canonical prefix position | D-01/D-03/D-11: `normalize_append` merges priority, appended wins |
| NORM-02 | `+project` metadata → placed after body in stable form | D-01/D-11: projects field merged via union, `rebuild_raw` places them after body |
| NORM-03 | `@context` metadata → preserved without requiring manual repositioning | Same as NORM-02 for contexts field |
| NORM-04 | `due:` / `t:` metadata → normalized consistently without discarding surrounding text | D-01: due_date/threshold_date fields merged; body words preserved separately |
| NORM-05 | Plain text / unrecognized metadata preserved in saved task | D-02: unknown tokens stay in `body` per existing `rebuild_raw` behavior |
| NORM-06 | Shared core logic in `todotxt-core`, no TUI duplication | D-11/D-13: `pub fn normalize_append` exported from `todotxt-core` |
</phase_requirements>

---

## Research Summary

Phase 21 adds a single new public function to `todotxt-core` and wires it into three call sites: the TUI append handler, the TUI save-and-exit handler, and the CLI append command. The codebase infrastructure is almost entirely ready — `rebuild_raw()`, `Task::parse()`, and `extract_tags()` together already implement the parse-then-merge pipeline; `normalize_append` is the thin composition layer on top.

The primary technical decision the planner must resolve is **module placement**: `normalize_append` needs access to `rebuild_raw()`, which is currently a module-private function in `task.rs`. Placing the new function in `task.rs` (same module) avoids any visibility changes. A new `normalize.rs` would require making `rebuild_raw` `pub(crate)` — one extra line, but extra churn on a well-tested file.

Config additions follow the existing `auto_creation_date` precedent (flat `bool` fields on `TuiConfig`), not a new `[normalization]` TOML subsection. The `[tui]` section is reserved for display settings (theme); behavioral toggles live at the top level.

**Primary recommendation:** Place `normalize_append` in `task.rs` (same module as `rebuild_raw`); add `normalize_append: bool` and `normalize_edit: bool` as flat fields on `TuiConfig`; use `rstest`-driven tests in `crates/todotxt-core/tests/normalize_tests.rs`.

---

## Existing Code Analysis

### Task Struct Fields and Builders

**File:** `crates/todotxt-core/src/task.rs`

```
Task {
    raw: String,               // private — canonical serialization; never set directly
    pub completed: bool,
    pub priority: Option<char>,
    pub creation_date: Option<NaiveDate>,
    pub completion_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub threshold_date: Option<NaiveDate>,
    pub projects: Vec<String>, // sorted, deduplicated (BTreeSet during parse → Vec)
    pub contexts: Vec<String>, // sorted, deduplicated
    pub body: String,          // remaining words after tags extracted
}
```

All `with_*` builders follow this exact pattern:
```rust
pub fn with_priority(self, priority: Option<char>) -> Self {
    let new_task = Task { priority, ..self };
    let new_raw = rebuild_raw(&new_task);  // ← module-private fn
    Task::parse(&new_raw)                  // ← re-parses to sync all fields
}
```
`normalize_append` must follow the same two-step close: `rebuild_raw` → `Task::parse`.

**Existing builders:** `with_completed`, `with_priority`, `with_creation_date`, `with_due_date`, `with_threshold_date`, `with_text_prepended`. All live in `task.rs`. None of these do multi-field merges — `normalize_append` is the first multi-field operation.

### rebuild_raw() Signature and Behavior

```rust
fn rebuild_raw(task: &Task) -> String  // module-private
```

**Output order (canonical todo.txt format):**
```
[x YYYY-MM-DD ] [(P) ] [YYYY-MM-DD ] BODY [+proj...] [@ctx...] [due:YYYY-MM-DD] [t:YYYY-MM-DD]
```

Key behaviors verified from source:
- Completion marker + completion_date before priority
- Priority before creation_date
- Body first among suffix tokens
- All projects appended alphabetically (BTreeSet in parse)
- All contexts appended alphabetically
- `due:` before `t:`
- Empty body + no suffix → no trailing space (`.trim_end()` applied)

`rebuild_raw` does NOT know about unknown `key:value` tokens — those survive only because they are never extracted from `body` during `Task::parse`. The `body` field retains them verbatim. This is exactly what NORM-05/D-02 require.

### extract_tags() — Private Helper

```rust
fn extract_tags(body: &str) -> (String, Vec<String>, Vec<String>, Option<NaiveDate>, Option<NaiveDate>)
```

Splits whitespace, recognizes `+proj`, `@ctx`, `due:YYYY-MM-DD`, `t:YYYY-MM-DD`. Unknown tokens (e.g., `rec:+1w`) are NOT recognized and stay in `body_words`. This is the exact parsing `normalize_append` should replicate for the append text — or it can call `Task::parse(append_text)` which invokes this function internally.

**IMPORTANT:** `normalize_append` does NOT need to call `extract_tags` directly. Calling `Task::parse(append_text)` is simpler and reuses all existing logic. The parsed `Task` fields then contain the tokens to merge.

### handle_append_text_key() Current Implementation

**File:** `crates/todotxt-tui/src/app.rs`, line 936

Current raw-concat path (Phase 20):
```rust
// Enter key handler, line ~970
let replacements: Vec<(usize, Task)> = sorted_indices
    .iter()
    .filter_map(|&idx| {
        tasks.get(idx).map(|t| {
            let new_raw = format!("{} {}", t.to_raw().trim_end(), text);
            let new_task = Task::parse(&new_raw);   // ← Phase 20 raw concat
            (idx, new_task)
        })
    })
    .collect();
```

**Phase 21 change:** Branch on `self.config.normalize_append`:
- `true`  → replace `Task::parse(&new_raw)` with `normalize_append(t, &text)`
- `false` → keep current raw-concat (D-08)

The config value is accessible via `self.config` (a `TuiConfig` instance already stored on `App`).

### save_and_exit() Current Implementation

**File:** `crates/todotxt-tui/src/app.rs`, line 1009

```rust
fn save_and_exit(&mut self) -> color_eyre::Result<()> {
    let text = self.editor.lines().first().cloned().unwrap_or_default();
    let task = Task::parse(&text);                      // ← normalize_edit replaces this
    let mode = self.mode;
    match mode {
        AppMode::Adding => {
            self.task_list.add(task)...
        }
        AppMode::Editing { original_idx } => {
            self.task_list.update(original_idx, task)...
        }
        ...
    }
}
```

**Phase 21 change for `AppMode::Editing`:** When `self.config.normalize_edit` is true, instead of `Task::parse(&text)`, use:
```rust
let original = self.task_list.tasks()[original_idx].clone();
let task = normalize_append(&original, &text);
```

**Note:** `AppMode::Adding` is NOT subject to normalize_edit (user is creating a new task, there is no "original" to merge into). Only `AppMode::Editing { original_idx }` is affected.

**Open question for planner:** In edit mode, the user replaces the ENTIRE task line. `normalize_append(&original, &edited_text)` would merge edited_text tokens on top of the original, concatenating bodies. This may produce doubled body text. Consider whether `normalize_edit` should use `normalize_append(&Task::default(), &edited_text)` to normalize as standalone text (pure token extraction + rebuild), or a different call site pattern. See Risks section below.

### CLI append Subcommand Current Structure

**File:** `crates/todotxt-cli/src/commands/append.rs`

```rust
pub fn run(todo_path: &Path, id: usize, text: &str, renderer: &Renderer) -> Result<(), CliError> {
    let task = list.tasks()[idx].clone();
    let updated = Task::parse(&format!("{} {}", task.to_raw(), text));  // ← raw concat
    list.update(idx, updated.clone())?;
}
```

**File:** `crates/todotxt-cli/src/cli.rs`, line ~101

```rust
Append {
    /// 1-based task ID
    id: usize,
    /// Text to append (leading space added automatically)
    text: String,
},
```

**Phase 21 changes:**
1. Add `#[arg(long)]` `normalize: bool` field to `Append` variant in `cli.rs`
2. Pass `normalize` flag to `commands::append::run()`
3. Branch in `run()`: if normalize → call `todotxt_core::normalize_append(&task, text)`; else keep raw concat

### TUI Config Struct Current Shape

**File:** `crates/todotxt-tui/src/config.rs`

```rust
pub struct TuiConfig {
    pub todo_file: Option<PathBuf>,
    pub done_file: Option<PathBuf>,
    pub auto_creation_date: bool,          // ← precedent for behavioral bool flags
    pub presets: HashMap<String, TuiPreset>,
    pub tui: TuiSection,                   // ← display settings (theme)
}

pub struct TuiSection {
    pub theme: String,
}
```

**TOML structure today:**
```toml
todo_file = "~/todo.txt"
auto_creation_date = true

[tui]
theme = "default"

[presets.work]
filter = "+work"
```

`TuiSection` is for display/visual settings. Behavioral flags like `auto_creation_date` live flat on `TuiConfig`. Adding `normalize_append` and `normalize_edit` flat on `TuiConfig` matches this precedent.

---

## Integration Points

### Exact Function Signatures to Call/Modify

| Item | Location | Current | After Phase 21 |
|------|----------|---------|----------------|
| `normalize_append` | NEW: `todotxt-core/src/task.rs` or `normalize.rs` | — | `pub fn normalize_append(task: &Task, append_text: &str) -> Task` |
| `rebuild_raw` | `todotxt-core/src/task.rs:305` | `fn rebuild_raw` (private) | unchanged if in task.rs; `pub(crate)` if in normalize.rs |
| `handle_append_text_key` | `todotxt-tui/src/app.rs:936` | raw concat | branch on `self.config.normalize_append` |
| `save_and_exit` | `todotxt-tui/src/app.rs:1009` | `Task::parse(&text)` | branch on `self.config.normalize_edit` for Editing arm |
| `commands::append::run` | `todotxt-cli/src/commands/append.rs:1` | raw concat | branch on `normalize: bool` arg |
| `Commands::Append` | `todotxt-cli/src/cli.rs:101` | `{ id, text }` | `{ id, text, normalize: bool }` |
| `TuiConfig` | `todotxt-tui/src/config.rs:37` | no normalize fields | +`normalize_append: bool`, +`normalize_edit: bool` |

### Exact File Paths to Create/Modify

| Action | Path |
|--------|------|
| MODIFY (add function) | `crates/todotxt-core/src/task.rs` |
| MODIFY (export) | `crates/todotxt-core/src/lib.rs` — add `pub use task::normalize_append` |
| MODIFY (config fields) | `crates/todotxt-tui/src/config.rs` |
| MODIFY (append branch) | `crates/todotxt-tui/src/app.rs` |
| MODIFY (CLI flag + branch) | `crates/todotxt-cli/src/cli.rs` |
| MODIFY (CLI run fn) | `crates/todotxt-cli/src/commands/append.rs` |
| CREATE (tests) | `crates/todotxt-core/tests/normalize_tests.rs` |

**If normalize.rs path chosen:** also CREATE `crates/todotxt-core/src/normalize.rs` and add `pub mod normalize;` to `lib.rs`.

---

## Implementation Approach

### Recommended Placement: task.rs (not normalize.rs)

**Rationale:** `normalize_append` calls `rebuild_raw()` as its last step before `Task::parse`. `rebuild_raw` is module-private in `task.rs`. Placing `normalize_append` in the same module:
- Zero visibility changes to existing code
- Follows the established `with_*` builder pattern precisely
- `task.rs` is 324 lines — adding ~30 lines keeps it manageable
- `normalize.rs` would require `pub(crate) fn rebuild_raw` — extra churn on a well-tested, stable file

**When to prefer normalize.rs:** only if the merge logic grows substantially beyond ~40 lines (e.g., if complex deduplication rules are needed). For the current scope it is not warranted.

### normalize_append Implementation Pattern

```rust
/// Parse `append_text` for recognized todo.txt tokens, merge into `task`'s fields,
/// and return a new Task rebuilt via `rebuild_raw()`.
///
/// Merge rules:
/// - priority: appended wins if present; original preserved if not (D-03, D-04)
/// - projects, contexts: union (BTreeSet dedup in parse handles sorting)
/// - due_date, threshold_date: appended wins if present
/// - body: original body + " " + appended plain-text body (non-token words)
/// - completed, creation_date, completion_date: always from original (not in append)
pub fn normalize_append(task: &Task, append_text: &str) -> Task {
    let appended = Task::parse(append_text);

    // Merge projects: union
    let mut projects = task.projects.clone();
    for p in &appended.projects {
        if !projects.contains(p) {
            projects.push(p.clone());
        }
    }
    projects.sort();  // maintain alphabetical order

    // Merge contexts: union
    let mut contexts = task.contexts.clone();
    for c in &appended.contexts {
        if !contexts.contains(c) {
            contexts.push(c.clone());
        }
    }
    contexts.sort();

    // Merge body: concatenate (original first, appended body second)
    let new_body = if appended.body.is_empty() {
        task.body.clone()
    } else if task.body.is_empty() {
        appended.body.clone()
    } else {
        format!("{} {}", task.body, appended.body)
    };

    let merged = Task {
        raw: String::new(),           // will be set by rebuild_raw + Task::parse
        completed: task.completed,
        priority: appended.priority.or(task.priority),   // appended wins (D-03)
        creation_date: task.creation_date,
        completion_date: task.completion_date,
        due_date: appended.due_date.or(task.due_date),   // appended wins
        threshold_date: appended.threshold_date.or(task.threshold_date),
        projects,
        contexts,
        body: new_body,
    };

    let new_raw = rebuild_raw(&merged);
    Task::parse(&new_raw)
}
```

**Note:** `Task { raw: String::new(), ..merged }` pattern cannot be used directly since `raw` is private — but since we're implementing the function inside `task.rs`, the private field is directly accessible.

### Config Struct Additions

Add flat `bool` fields to `TuiConfig` (following `auto_creation_date` precedent):

```rust
pub struct TuiConfig {
    // ... existing fields ...
    /// Normalize tokens in bulk-append text (extract priority/projects/etc. into fields).
    #[serde(default = "default_true")]
    pub normalize_append: bool,
    /// Normalize tokens when saving an edited task.
    #[serde(default = "default_true")]
    pub normalize_edit: bool,
}

fn default_true() -> bool { true }
```

**TOML representation:**
```toml
normalize_append = true   # optional; defaults to true
normalize_edit = true     # optional; defaults to true
```

**Why not `[normalization]` table:** The `[tui]` subsection holds display settings only (theme). Behavioral toggles live flat at top level. Consistency matters for users reading config docs.

**Serde note:** `#[serde(default)]` alone would use `bool::default()` = `false`, which contradicts D-06/D-07 (defaults should be `true`). Must use `#[serde(default = "default_true")]` with a helper fn.

### Test Strategy for normalize_append

Tests live in `crates/todotxt-core/tests/normalize_tests.rs`, following the existing pattern in `task_tests.rs`. Use `rstest` for parametric cases.

**Test categories:**

| Category | Cases |
|----------|-------|
| Priority merge | No original + append has priority → appended; original has priority + append has priority → appended wins; original has priority + append has no priority → original kept |
| Project merge | No overlap → union; overlap → deduplicated; empty append → original unchanged |
| Context merge | Same as project |
| due_date merge | Append has date → wins; append empty → original kept; both have date → appended wins |
| threshold_date merge | Same as due_date |
| Body concatenation | Plain words append; empty append body → original only; empty original body → appended only |
| Unknown tokens in append | `rec:+1w` in append text → stays in body verbatim |
| Round-trip | `normalize_append(task, text).to_raw()` produces canonical todo.txt line |
| Completed task append | `completed=true` preserved; completion_date preserved |
| Creation date preserved | `creation_date` from original, not affected by append |

**Test file registration:** Add `mod normalize_tests;` to... actually in Rust integration tests under `tests/`, each file is automatically discovered by cargo. No explicit `mod` registration needed.

---

## Risks and Edge Cases

### 1. Edit Mode: Body Doubling (HIGH RISK)

**What goes wrong:** In `save_and_exit()`, when `normalize_edit = true` and mode is `AppMode::Editing { original_idx }`, calling `normalize_append(&original_task, &edited_text)` treats `edited_text` as an append. The edited text already contains the full task content. If original.body = "fix bug" and edited_text = "fix bug +work", then:
- `appended.body` = "fix bug" (non-tag words)
- Final body = "fix bug fix bug" (doubled!)

**Why it happens:** `normalize_append` concatenates bodies by design (for the append use case). In the edit use case the "append text" is actually a replacement of the whole line.

**Resolution options for planner:**
1. For edit mode, use `normalize_append(&Task::default(), &edited_text)` — normalizes token placement in edited text without merging against original body. Preserves original's `creation_date` and `completed` status separately.
2. Implement a second function `normalize_line(edited_text: &str, creation_date: Option<NaiveDate>, completed: bool, ...) -> Task` that normalizes standalone text while preserving non-content fields.
3. For edit mode, strip the body from the original before calling: `normalize_append(&original.with_body(""), &edited_text)` (requires a `with_body` builder).
4. **Simplest/recommended:** for `AppMode::Editing`, call `normalize_append` with the original task where `body` is set to empty — i.e., create a "structural shell" from original (preserving dates/completion) and merge the edited text as if appending. This is `normalize_append(&Task { body: String::new(), ..original.clone() }, &edited_text)`.

**Note:** Option 4 requires accessing the private `body` field, which is fine inside `task.rs`. Or expose a `with_body` builder.

### 2. Priority Token Position in Append Text

**What goes wrong:** User appends `"done (B) +work"`. After `Task::parse("done (B) +work")`, the result is `priority: None, body: "done (B)"` because `(B)` is not at position 0 of the parsed text.

**Why it happens:** `parse_priority_prefix` only matches `(X) ` at the very start. The appended text `"done (B) +work"` starts with `"done"`, so priority is not parsed.

**Impact:** The normalization cannot "find" a priority in the middle of append text — it only works if the user types `"(B) done +work"` (priority first).

**Mitigation:** This is acceptable per D-01 (parse-then-merge uses `Task::parse` which follows standard todo.txt format). Document the behavior in phase UAT: users must place priority at the start of append text for it to be recognized. If the planner wants more liberal priority detection, that is a separate concern (not in scope).

### 3. Duplicate Due/Threshold Date Handling

**What goes wrong:** Original task has `due:2026-05-01`, user appends `"due:2026-06-01"`. With "appended wins" logic, original due date is silently replaced.

**Why it matters:** Could be unexpected if user only intended to append text and accidentally typed a due date pattern.

**Resolution:** This is the defined behavior (D-03 analog for dates: appended wins). Document in UAT. No special handling needed.

### 4. Config Default Alignment (serde `default`)

**What goes wrong:** `#[serde(default)]` on a `bool` field uses `bool::default()` = `false`. Both `normalize_append` and `normalize_edit` should default to `true` (D-06, D-07). Using plain `#[serde(default)]` silently inverts the defaults.

**Prevention:** Use `#[serde(default = "default_true")]` with a named function returning `true`.

**Verify:** Existing `auto_creation_date: bool` uses `#[serde(default)]` — meaning it defaults to `false` when absent. This pattern is consistent with its semantic (opt-in date auto-prefix). But for normalize toggles, opt-out is the design intent, so the named-fn pattern is required.

### 5. projects/contexts Vec Ordering After Union Merge

**What goes wrong:** `normalize_append` merges two `Vec<String>` by pushing new entries and sorting. But `rebuild_raw` emits projects in `task.projects` order (Vec iteration). If the sort is not stable or projects contain mixed case, ordering could differ from what `Task::parse` produces (BTreeSet → alphabetical).

**Prevention:** After merging, sort the `projects` and `contexts` vecs before constructing the merged `Task`. Alternatively, collect into a `BTreeSet` during merge and convert to `Vec` — this exactly mirrors what `extract_tags` does and guarantees alphabetical order.

### 6. CLI Append — 1-based ID Convention

The CLI `append` command uses 1-based IDs publicly but 0-based internally (validated by `validate_id`). The `normalize: bool` flag does not affect this — just note that `normalize_append` receives the raw `&Task` reference, not an ID. No risk here, just documenting for planner.

### 7. Phase 20 Compatibility (D-08)

When `normalize_append = false` in config, `handle_append_text_key` must fall back to the exact Phase 20 code path:
```rust
let new_raw = format!("{} {}", t.to_raw().trim_end(), text);
Task::parse(&new_raw)
```
Ensure the else-branch does not deviate from Phase 20 behavior.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `rstest 0.26` |
| Config file | `Cargo.toml` `[dev-dependencies]` (already configured) |
| Quick run command | `cargo test -p todotxt-core normalize` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| NORM-01 | Priority token in append lands in priority field | unit | `cargo test -p todotxt-core normalize::priority` | ❌ Wave 0 |
| NORM-01 | Appended priority replaces original (D-03) | unit | `cargo test -p todotxt-core normalize::priority_conflict` | ❌ Wave 0 |
| NORM-02 | +project tags merged into projects field | unit | `cargo test -p todotxt-core normalize::projects` | ❌ Wave 0 |
| NORM-03 | @context tags merged into contexts field | unit | `cargo test -p todotxt-core normalize::contexts` | ❌ Wave 0 |
| NORM-04 | due: and t: tokens normalized into date fields | unit | `cargo test -p todotxt-core normalize::dates` | ❌ Wave 0 |
| NORM-05 | Unknown tokens (rec:+1w) preserved in body | unit | `cargo test -p todotxt-core normalize::unknown_tokens` | ❌ Wave 0 |
| NORM-06 | TUI uses todotxt_core::normalize_append (no local logic) | integration | `cargo test -p todotxt-tui` (compile check) | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p todotxt-core normalize`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `crates/todotxt-core/tests/normalize_tests.rs` — covers NORM-01 through NORM-05
- [ ] No framework install needed — `rstest` already in `[dev-dependencies]`
- [ ] No new test config files needed

---

## Sources

All findings are **[VERIFIED: codebase]** — read directly from workspace files:

- `crates/todotxt-core/src/task.rs` — Task struct, rebuild_raw, extract_tags, with_* builders
- `crates/todotxt-core/src/lib.rs` — public API surface
- `crates/todotxt-tui/src/app.rs` — handle_append_text_key (line 936), save_and_exit (line 1009)
- `crates/todotxt-tui/src/config.rs` — TuiConfig, TuiSection
- `crates/todotxt-cli/src/commands/append.rs` — CLI run fn
- `crates/todotxt-cli/src/cli.rs` — Commands::Append variant
- `.planning/phases/21-smart-text-normalization/21-CONTEXT.md` — all decisions
- `.planning/REQUIREMENTS.md` — NORM-01 through NORM-06

## Assumptions Log

All claims in this research are [VERIFIED: codebase]. No assumptions.

---

## RESEARCH COMPLETE

**Phase:** 21 — Smart Text Normalization
**Confidence:** HIGH — pure codebase analysis, no external dependencies

### Key Findings

1. **`rebuild_raw` is module-private** in `task.rs` — placing `normalize_append` in the same file (`task.rs`) is the zero-friction path; `normalize.rs` requires making `rebuild_raw` `pub(crate)`.
2. **The parse-then-merge implementation is straightforward**: call `Task::parse(append_text)` to tokenize append text, union projects/contexts with BTreeSet sort, apply "appended wins" for priority/dates, concatenate bodies.
3. **Edit mode body-doubling risk is real**: calling `normalize_append(&original, &edited_whole_line)` doubles the body. The planner must use a structural-shell pattern (original with empty body) or a separate normalize-only function for the edit path.
4. **Config defaults require `#[serde(default = "default_true")]`**: plain `#[serde(default)]` gives `false`, which inverts the intended opt-out defaults for `normalize_append` and `normalize_edit`.
5. **CLI change is minimal**: add `#[arg(long)] normalize: bool` to `Append` variant and branch in `commands::append::run`.
6. **Test infrastructure is ready**: `rstest` already in dev-dependencies; `tests/normalize_tests.rs` follows established patterns in `tests/task_tests.rs`.

### File Created

`.planning/phases/21-smart-text-normalization/21-RESEARCH.md`

### Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| Standard stack (Rust internals) | HIGH | Read directly from source |
| Architecture (module placement) | HIGH | rebuild_raw visibility verified |
| Config shape | HIGH | TuiConfig struct verified |
| CLI integration | HIGH | Commands enum and run fn verified |
| Edit mode body-doubling risk | HIGH | Derived from verified merge logic |
| Test infrastructure | HIGH | Cargo.toml dev-deps and test file patterns verified |

### Open Questions

1. **Edit mode merge strategy** — Does the planner want `normalize_append(&structural_shell, &edited_text)` (preserve original dates/completion but use edited body), or a separate `normalize_line` function? Either is ~5 lines; the decision affects the public API surface.
2. **`with_body` builder** — If the structural-shell approach is chosen, it requires a new `pub fn with_body(self, body: String) -> Self` builder on `Task`. Consider whether to add it (useful beyond this phase) or use a different approach.

### Ready for Planning

Research complete. Planner can now create PLAN.md files for all implementation tasks.

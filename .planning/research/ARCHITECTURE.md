# Architecture Patterns: Rust todo.txt Port

**Domain:** CLI tool / cross-platform library + binary  
**Researched:** 2025-01-31  
**Confidence:** HIGH — direct port of a well-understood C# codebase, well-established Rust workspace patterns

---

## Recommended Architecture

A Cargo workspace with a strict two-layer separation mirroring the existing C#
`ToDoLib` / `Client` split — but structured for future TUI and GUI growth:

```
todotxt.net/                    ← git repo root (existing)
├── Cargo.toml                  ← workspace root
├── crates/
│   ├── todotxt-core/           ← library crate (= ToDoLib equivalent)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs          ← re-exports public API
│   │       ├── task.rs         ← Task struct, parser, serialiser
│   │       ├── task_list.rs    ← TaskList file I/O
│   │       ├── filter.rs       ← filter/search logic
│   │       ├── sort.rs         ← sort strategies
│   │       └── error.rs        ← TodoError enum (thiserror)
│   └── todotxt-cli/            ← binary crate (= Client equivalent, minus UI)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs         ← entry point; clap CLI setup
│           ├── commands/       ← one module per subcommand
│           │   ├── add.rs
│           │   ├── list.rs
│           │   ├── complete.rs
│           │   ├── delete.rs
│           │   ├── edit.rs
│           │   ├── archive.rs
│           │   └── mod.rs
│           ├── output.rs       ← human-readable vs JSON rendering
│           ├── config.rs       ← settings persistence (dirs crate)
│           └── watch.rs        ← file-watching daemon (notify crate)
├── Client/                     ← existing C# WPF app (untouched)
├── ToDoLib/                    ← existing C# library (untouched)
└── ...
```

### Why `crates/` subdirectory (not root-level)

Placing all Rust crates under `crates/` avoids Cargo.toml collisions with the
existing VS solution structure and makes the workspace root unambiguous.
The existing C# source directories (Client/, ToDoLib/, etc.) are unaffected.

---

## Cargo Workspace Root

```toml
# Cargo.toml (repo root)
[workspace]
members = [
    "crates/todotxt-core",
    "crates/todotxt-cli",
    # future: "crates/todotxt-tui",
    # future: "crates/todotxt-gui",
]
resolver = "2"

[workspace.dependencies]
# Pin shared dependency versions here — individual crates inherit via
# `dependency = { workspace = true }`
chrono      = { version = "0.4", features = ["serde"] }
regex       = "1"
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
thiserror   = "1"
```

---

## Component Boundaries

| Crate | Responsibility | Public API Exports | May NOT depend on |
|-------|---------------|-------------------|-------------------|
| `todotxt-core` | Parse, represent, persist, filter, sort todo.txt tasks | `Task`, `TaskList`, `Filter`, `SortType`, `TodoError` | Any UI crate, `clap`, `notify`, `dirs` |
| `todotxt-cli` | Command routing, output rendering, config, file watching | (binary — no public API) | TUI/GUI crates |
| `todotxt-tui` _(future)_ | Interactive terminal UI | (binary or library) | GUI crates |
| `todotxt-gui` _(future)_ | Native desktop GUI | (binary) | TUI crates |

**Rule:** `todotxt-core` is the only crate allowed to touch `Task` and `TaskList`
internals. All other crates consume the public API surface only.

---

## Core Library Public API (`todotxt-core`)

This is the direct Rust translation of `ToDoLib/Task.cs` and `ToDoLib/TaskList.cs`.
The `Raw`-as-canonical-string approach from C# maps perfectly to Rust.

### `Task` struct

```rust
// src/task.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Task {
    // Canonical representation — serialize/deserialize always goes through this
    pub raw: String,

    // Parsed fields (derived from raw; kept in sync)
    pub completed: bool,
    pub priority: Option<char>,          // 'A'–'Z', None if no priority
    pub creation_date: Option<NaiveDate>,
    pub completion_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub threshold_date: Option<NaiveDate>,
    pub projects: Vec<String>,           // sorted, deduplicated (+Project)
    pub contexts: Vec<String>,           // sorted, deduplicated (@Context)
    pub body: String,                    // remainder after stripping metadata
}

impl Task {
    /// Parse a single todo.txt line. Relative dates (today/tomorrow/weekday)
    /// are resolved to absolute dates at parse time, matching C# behaviour.
    pub fn parse(raw: &str) -> Result<Self, TodoError>;

    /// Serialise back to todo.txt line format.
    /// Prefers the stored `raw` field (preserves user's original layout)
    /// and mutates only completion status if changed — same as C# ToString().
    pub fn to_raw(&self) -> String;

    /// Mutation helpers — return new Task with updated raw string
    pub fn with_completed(self, completed: bool) -> Self;
    pub fn with_priority(self, p: Option<char>) -> Self;
    pub fn inc_priority(self) -> Self;
    pub fn dec_priority(self) -> Self;
    pub fn with_due_date(self, date: Option<NaiveDate>) -> Self;

    /// Due status (not-due / today / overdue)
    pub fn due_status(&self) -> DueStatus;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DueStatus { NotDue, Today, Overdue }
```

**Design note:** Mutation methods take `self` by value and return a new `Task`
(builder/consume-transform pattern). This is more idiomatic Rust than C#'s
mutable-fields approach and makes the CLI's "apply transform, write back"
pattern natural.

### `TaskList` struct

```rust
// src/task_list.rs
pub struct TaskList {
    path: PathBuf,
    pub tasks: Vec<Task>,
    pub projects: BTreeSet<String>,   // derived metadata
    pub contexts: BTreeSet<String>,
    pub priorities: BTreeSet<char>,
    preferred_line_ending: LineEnding,
}

impl TaskList {
    /// Load from file. Skips blank lines by default (preserve_whitespace = false).
    pub fn load(path: impl AsRef<Path>) -> Result<Self, TodoError>;

    /// Reload from disk (used after external file change).
    pub fn reload(&mut self) -> Result<(), TodoError>;

    /// Append a task to the file and in-memory list.
    pub fn add(&mut self, task: Task) -> Result<(), TodoError>;

    /// Remove a task by raw-string identity (matches C# Remove-by-Raw).
    pub fn delete(&mut self, raw: &str) -> Result<(), TodoError>;

    /// Replace one task with another (full-file-rewrite, matches C# Update).
    pub fn update(&mut self, old_raw: &str, new_task: Task) -> Result<(), TodoError>;

    /// Batch update — disables intermediate writes; single file write at end.
    /// This matches the C# ModifySelectedTasks pattern.
    pub fn batch_update<F>(&mut self, f: F) -> Result<(), TodoError>
    where F: FnMut(&Task) -> Option<Task>;  // None = leave unchanged

    /// Save current in-memory state to disk.
    pub fn save(&self) -> Result<(), TodoError>;

    /// Return derived metadata (projects/contexts/priorities are always current).
    pub fn metadata(&self) -> TaskListMetadata;
}
```

### `Filter` struct

```rust
// src/filter.rs
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Filter {
    pub text: Option<String>,
    pub projects: Vec<String>,
    pub contexts: Vec<String>,
    pub priority: Option<char>,
    pub due_before: Option<NaiveDate>,
    pub hide_completed: bool,
    pub hide_future: bool,   // threshold date in future
}

impl Filter {
    pub fn matches(&self, task: &Task) -> bool;
    pub fn apply<'a>(&self, tasks: &'a [Task]) -> Vec<&'a Task>;
}
```

### `SortType` enum

```rust
// src/sort.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SortType {
    Alphabetical,
    Priority,
    DueDate,
    Project,
    Context,
    None,  // preserve file order
}

pub fn sort_tasks(tasks: &mut Vec<&Task>, sort: SortType);
```

### Error type

```rust
// src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum TodoError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error on line {line}: {message}")]
    Parse { line: usize, message: String },
    #[error("Task not found: {raw}")]
    TaskNotFound { raw: String },
}
```

---

## CLI Binary Architecture (`todotxt-cli`)

### Command Structure (clap)

```
todotxt [OPTIONS] <SUBCOMMAND>

OPTIONS:
  --file <PATH>         todo.txt path (overrides config)
  --json                Output as JSON array
  --no-color            Disable ANSI colour

SUBCOMMANDS:
  add     <TEXT>        Add a task
  list    [FILTER]      List (and filter) tasks
  done    <ID|--all>    Mark complete
  undone  <ID>          Mark incomplete
  del     <ID>          Delete
  edit    <ID> <TEXT>   Replace task text
  archive               Move completed tasks to done.txt
  config                Show/set configuration
```

`ID` is the 1-based line index in the file (same as todo.sh convention).

### Output Layer (`output.rs`)

Two rendering paths, selected by `--json` flag:

```rust
pub enum OutputFormat { Human, Json }

pub fn render_tasks(tasks: &[&Task], format: OutputFormat);
pub fn render_task(task: &Task, index: usize, format: OutputFormat);
pub fn render_error(err: &TodoError, format: OutputFormat);
```

JSON output shape:
```json
[
  {
    "id": 1,
    "raw": "(A) 2024-01-15 Fix the bug +Project @Context due:2024-01-31",
    "completed": false,
    "priority": "A",
    "projects": ["+Project"],
    "contexts": ["@Context"],
    "due_date": "2024-01-31",
    "threshold_date": null,
    "creation_date": "2024-01-15",
    "body": "Fix the bug",
    "due_status": "overdue"
  }
]
```

### Config (`config.rs`)

Uses `directories` crate for cross-platform paths:
- **Normal mode:** `~/.config/todotxt/config.toml` (Linux/macOS), `%APPDATA%\todotxt\config.toml` (Windows)
- **Portable mode:** `config.toml` beside the binary (detected if file exists at that path)

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub todo_file: PathBuf,
    pub done_file: Option<PathBuf>,
    pub sort: SortType,
    pub filter_presets: Vec<(String, Filter)>,  // up to 9 named presets
    pub auto_creation_date: bool,
    pub preserve_whitespace: bool,
}
```

Config is loaded once at startup; individual `--file` overrides take precedence.

### File Watch (`watch.rs`)

Uses `notify` crate. Only relevant for daemon/watch mode (explicit `--watch` flag).
Not used by default single-shot CLI invocations.

---

## Data Flow

### Single-shot CLI command

```
argv
  │
  ▼
main.rs → clap parse args → Config::load()
  │
  ▼
TaskList::load(path)          ← reads file, constructs Vec<Task>
  │
  ▼
Filter::apply(&tasks)         ← if list/done command has filter args
  │
  ▼
sort_tasks(&mut tasks, sort)  ← applies sort from config or --sort flag
  │
  ▼
output::render_tasks(...)     ← human table or JSON array
  │
  ▼
exit(0)
```

### Mutation command (add / done / delete / edit)

```
TaskList::load(path)
  │
  ▼
task operation:
  add    → TaskList::add(Task::parse(text)?)
  done   → TaskList::batch_update(|t| Some(t.clone().with_completed(true)))
  delete → TaskList::delete(raw)
  edit   → TaskList::update(old_raw, Task::parse(new_text)?)
  │
  ▼
(TaskList::save called internally after each mutation)
  │
  ▼
output::render_tasks(affected_tasks, format)
```

**Key principle:** Every mutation is an autonomous read-modify-write on the file
(same as C# TaskList). No long-lived in-memory state between CLI invocations.
File is always the source of truth.

---

## Build Order (Phase Dependency Graph)

```
Phase 1: todotxt-core
  ├─ error.rs           (no deps within crate)
  ├─ task.rs            (depends on: error.rs, chrono, regex)
  ├─ task_list.rs       (depends on: task.rs, error.rs)
  ├─ filter.rs          (depends on: task.rs)
  └─ sort.rs            (depends on: task.rs)

Phase 2: todotxt-cli
  ├─ config.rs          (depends on: todotxt-core::SortType/Filter)
  ├─ output.rs          (depends on: todotxt-core::Task)
  ├─ commands/list.rs   (depends on: core, config, output)
  ├─ commands/add.rs    (depends on: core, config, output)
  ├─ commands/...       (depends on: core, config, output)
  ├─ watch.rs           (depends on: core, config; optional feature)
  └─ main.rs            (depends on: all commands, clap)

Phase 3 (future): todotxt-tui
  └─ depends on: todotxt-core (same API as CLI uses)

Phase 4 (future): todotxt-gui
  └─ depends on: todotxt-core (same API as CLI uses)
```

**Implication:** `todotxt-core` must reach a stable public API before `todotxt-cli`
can progress. Within `todotxt-core`, `Task` parsing must be solid before `TaskList`
and `Filter` can be built. Tests belong in `todotxt-core` — they test the domain
model in isolation, exactly as `ToDoTests/` tests `ToDoLib` in isolation.

---

## TUI / GUI Extensibility

Future UI crates need **nothing added** to `todotxt-core` to support basic use.
They consume the same `TaskList`, `Task`, `Filter`, and `SortType` types as the CLI.

What they will add:
- **TUI** (`todotxt-tui`): event loop, keybindings, ratatui widgets; `TaskList`
  held in app state rather than loaded per-command; `FileWatcher` from `notify`
  used for live reload. `Task` mutation still goes through `TaskList::update`.
- **GUI** (`todotxt-gui`): same as TUI but with a windowing framework (egui/iced/slint).
  Settings would share the same `Config` struct from CLI or a new GUI-specific
  extension of it.

The only core API addition TUI/GUI will likely need is **change notification** — a
callback or channel when `TaskList` mutates. This is the Rust equivalent of the C#
`TaskList.Modified` event. Design recommendation: add a `notify_tx:
Option<Sender<()>>` field to `TaskList` in a future phase, or use a simple
`Arc<Mutex<TaskList>>` with the UI polling/subscribing externally.

---

## Key Patterns to Follow

### Pattern 1: Raw-as-canonical (from C# `Task.Raw`)

The `raw` field is the ground truth. All serialisation goes through `to_raw()`.
Parsed fields are derived views. When a task is "mutated", the mutation rebuilds
`raw` using regex substitution on the original string — preserving the user's
original layout (project/context position in the line), exactly as C# does.

```rust
// Good — preserves original raw, mutates only the changed part
pub fn with_completed(self, completed: bool) -> Self {
    let new_raw = if completed {
        let without_priority = PRIORITY_RE.replace(&self.raw, "");
        format!("x {} {}", today_str(), without_priority.trim())
    } else {
        COMPLETED_RE.replace(&self.raw, "").trim().to_string()
    };
    Self::parse(&new_raw).unwrap_or(self)
}
```

### Pattern 2: Autonomous file operations (from C# TaskList)

Every public `TaskList` mutation method reads the file first (via `reload`) if
the file may have changed, applies the mutation in memory, then writes the full
file. This is identical to the C# pattern and minimises concurrent conflict risk
with external editors.

### Pattern 3: Batch update for multi-task operations

```rust
// CLI: mark all filtered tasks complete
task_list.batch_update(|t| {
    if filter.matches(t) {
        Some(t.clone().with_completed(true))
    } else {
        None
    }
})?;
```

`batch_update` does a single file read + single file write for the entire
batch — critical for correctness when completing many tasks at once.

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Fat binary — business logic in `todotxt-cli`

**What:** Putting filter logic, sort logic, or task mutation in CLI command handlers.  
**Why bad:** TUI/GUI crates would have to duplicate or re-import from the CLI.  
**Instead:** All logic lives in `todotxt-core`. CLI handlers are thin translators
from `clap::ArgMatches` to core API calls + output rendering.

### Anti-Pattern 2: Mutable `Task` fields without updating `raw`

**What:** Setting `task.priority = Some('B')` directly without regenerating `raw`.  
**Why bad:** `raw` and fields diverge; serialisation produces wrong output.  
**Instead:** All mutations go through `with_*` builder methods that rebuild `raw`.

### Anti-Pattern 3: ID-by-index as stable identity

**What:** Storing the 1-based file-line index as a task's stable identifier.  
**Why bad:** Any add/delete operation shifts all subsequent IDs. C# used raw-string
equality for identity; this is the right model.  
**Instead:** Use `raw` string as identity for update/delete operations internally.
The 1-based index is a display/UX affordance only — regenerated at each `list`.
Document this clearly in the CLI help.

### Anti-Pattern 4: Global mutable state in core

**What:** `static mut` or `lazy_static` task storage in `todotxt-core`.  
**Why bad:** Prevents multiple simultaneous `TaskList` instances (tests, TUI tabs).  
**Instead:** `TaskList` owns its state; callers hold the instance.

---

## Scalability Considerations

| Concern | Small files (<1K tasks) | Large files (10K+ tasks) | Notes |
|---------|------------------------|--------------------------|-------|
| Parse speed | Trivial | Linear scan, ~ms | Regex compilation should use `lazy_static`/`once_cell` |
| File write | Full rewrite always | Still acceptable at 10K | todo.txt files rarely exceed a few thousand tasks |
| Filter | In-memory scan | In-memory scan | No indexing needed at this scale |
| File watching | `notify` crate | Same | Debounce needed (same 1s delay as C#) |

todo.txt files in practice top out at hundreds to low thousands of tasks. Full
in-memory + full file rewrite is the correct model here — no database needed.

---

## Sources

- C# reference: `ToDoLib/Task.cs`, `ToDoLib/TaskList.cs` (analyzed 2025-01-31)
- C# architecture: `.planning/codebase/ARCHITECTURE.md`
- Cargo workspace docs: https://doc.rust-lang.org/cargo/reference/workspaces.html
- todo.txt format spec: https://github.com/todotxt/todo.txt
- Rust `thiserror`: https://docs.rs/thiserror
- Rust `notify` (file watching): https://docs.rs/notify
- Rust `directories` (config paths): https://docs.rs/directories
- Rust `clap` v4 (CLI): https://docs.rs/clap

---

*Architecture research: 2025-01-31*

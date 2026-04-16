# Phase 2 — Pattern Map

**Phase:** 2 — Core Library Completion
**Date:** 2026-04-15

Maps existing codebase patterns to the new files being created. Executor reads this to understand the conventions before writing code.

---

## Analog Files

| New File | Closest Analog | Role |
|----------|---------------|------|
| `src/filter.rs` | `src/task.rs` | Pure data type + logic, no I/O |
| `src/sort.rs` | `src/task.rs` | Pure enum + impl, no I/O |
| `src/watcher.rs` | `src/task_list.rs` | Stateful struct with fallible constructor |
| `src/portable.rs` | `src/error.rs` | Tiny module, single public function |
| `src/task_list.rs` (additions) | existing `task_list.rs` | Extend existing file |

---

## Existing Public API (Executor: use these directly)

### `src/error.rs`
```rust
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum TodoError {
    #[error("I/O error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("task not found at index {index}")]
    NotFound { index: usize },

    #[error("index {index} out of bounds (task count: {count})")]
    IndexOutOfBounds { index: usize, count: usize },
    // Phase 2 adds: Watch variant (cfg(feature = "watching"))
}
```

### `src/task.rs` (key public surface)
```rust
pub struct Task {
    // raw: String — private, accessed via to_raw()
    pub completed: bool,
    pub priority: Option<char>,
    pub creation_date: Option<NaiveDate>,
    pub completion_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub threshold_date: Option<NaiveDate>,
    pub projects: Vec<String>,   // sorted, deduplicated, without "+"
    pub contexts: Vec<String>,   // sorted, deduplicated, without "@"
    pub body: String,
}

impl Task {
    pub fn parse(line: &str) -> Self;    // infallible
    pub fn to_raw(&self) -> &str;        // canonical serialization
    pub fn due_status(&self) -> DueStatus;

    // Builder methods (value-consuming):
    pub fn with_completed(self, completed: bool) -> Self;
    pub fn with_priority(self, priority: Option<char>) -> Self;
    pub fn with_creation_date(self, date: Option<NaiveDate>) -> Self;
    pub fn with_due_date(self, date: Option<NaiveDate>) -> Self;
    pub fn with_threshold_date(self, date: Option<NaiveDate>) -> Self;
}

pub enum DueStatus { NotDue, Today, Overdue }
```

### `src/task_list.rs` (key public surface)
```rust
pub struct TaskList { /* private fields */ }

impl TaskList {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, TodoError>;
    pub fn load_with_options(path: impl AsRef<Path>, preserve_whitespace: bool) -> Result<Self, TodoError>;
    pub fn save(&self) -> Result<(), TodoError>;
    pub fn reload(&mut self) -> Result<(), TodoError>;

    // CRUD — all call save() internally:
    pub fn add(&mut self, task: Task) -> Result<(), TodoError>;
    pub fn update(&mut self, index: usize, new_task: Task) -> Result<(), TodoError>;
    pub fn delete(&mut self, index: usize) -> Result<(), TodoError>;

    // Getters:
    pub fn tasks(&self) -> &[Task];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn path(&self) -> &Path;
    pub fn line_ending(&self) -> LineEnding;

    // Phase 2 adds:
    // pub fn batch_update(&mut self, replacements: Vec<(usize, Task)>) -> Result<(), TodoError>;
    // pub fn filter(&self, filter: &Filter) -> Vec<(usize, &Task)>;
    // pub fn sort(&mut self, order: SortOrder);
}

pub enum LineEnding { Lf, CrLf }
```

### `src/lib.rs` (current exports)
```rust
pub mod error;
pub mod task;
pub mod task_list;

pub use error::TodoError;
pub use task::{DueStatus, Task};
pub use task_list::{LineEnding, TaskList};

// Phase 2 adds:
// pub mod filter; pub mod sort; pub mod portable;
// pub use filter::{Filter, FilterTerm};
// pub use sort::SortOrder;
// pub use portable::resolve_config_path;
// #[cfg(feature = "watching")] pub mod watcher;
// #[cfg(feature = "watching")] pub use watcher::FileWatcher;
```

---

## Code Conventions (from existing code)

### Error Handling
- All fallible public methods return `Result<_, TodoError>`
- Error variants use `thiserror` `#[error("...")]` and `#[source]` attributes
- I/O errors always include `path: PathBuf` for context

### Module Style
- Each module is a single file in `src/`
- Private helpers go below a `// ── Private helpers ───` separator comment
- Public types are grouped with `// ── TypeName ───` separator comments

### Builder Pattern (Task)
- Value-consuming builders `fn with_x(self, x: ...) -> Self` — not `&mut self`
- Each builder rebuilds `raw` via `rebuild_raw()` then re-parses (round-trip)

### Tests
- Integration tests in `crates/todotxt-core/tests/*.rs` (separate files per feature)
- Unit tests can also be in `#[cfg(test)] mod tests` within the module file
- Phase 1 pattern: separate test files (`task_tests.rs`, `task_list_tests.rs`)
- `rstest` with `#[rstest]` + `#[case(...)]` for parametrized tests
- `insta` with `assert_snapshot!` for snapshot tests

### Cargo Features
- No features currently in `todotxt-core`
- Phase 2 adds: `watching = ["dep:notify-debouncer-mini"]`
- Feature-gated code uses `#[cfg(feature = "watching")]`

### Struct Visibility
- Structs are `pub`; internal implementation fields are private
- `#[non_exhaustive]` on public enums that may grow (e.g., `DueStatus`, `SortOrder`)

---

## ## PATTERN MAPPING COMPLETE

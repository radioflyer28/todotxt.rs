# Phase 1 Research: Workspace Bootstrap + Core Library Foundation

**Phase:** 1 — Workspace Bootstrap + Core Library Foundation
**Researched:** 2026-04-15
**Confidence:** HIGH — all crate versions verified; C# source fully inspected; todo.txt spec reviewed

---

## Standard Stack

All crate versions from `.planning/research/STACK.md` (verified against crates.io):

| Crate | Version | Purpose in Phase 1 |
|-------|---------|---------------------|
| `winnow` | 1.0.1 | Single-pass todo.txt line parser |
| `chrono` | 0.4 | `NaiveDate` for all date fields |
| `thiserror` | 2.0 | `TodoError` enum with `#[error]` derives |
| `serde` | 1.0 (+ `derive` feature) | `Serialize`/`Deserialize` on `Task` |
| `serde_json` | 1.0 | JSON serialization (re-exported for downstream) |
| `regex` | 1.x | Supplemental pattern matching (filter engine in Phase 2; static patterns in Phase 1 for relative date replacement) |
| `tempfile` | 3.27 | `NamedTempFile::persist()` for atomic writes |
| `rstest` | 0.26 | Parameterized test tables |
| `insta` | 1.47 | Snapshot round-trip tests |

### Crates NOT needed in Phase 1

| Crate | Why deferred |
|-------|-------------|
| `notify` / `notify-debouncer-mini` | File watching is Phase 2 (CORE-04) |
| `clap` / `anyhow` / `owo-colors` / `comfy-table` | CLI is Phase 3+ |
| `directories` / `toml` | Config is Phase 3 (CFG-01) |

---

## Architecture Patterns

### Workspace Layout (from CONTEXT.md Decision 1)

```
todotxt.net/                  ← git root (C# stays untouched)
├── Cargo.toml                ← workspace root [workspace] members = ["crates/*"]
├── crates/
│   ├── todotxt-core/         ← Phase 1 library crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs        ← public re-exports
│   │       ├── task.rs       ← Task struct + winnow parser + builder methods
│   │       ├── task_list.rs  ← TaskList + atomic file I/O
│   │       └── error.rs      ← TodoError enum (thiserror 2.0)
│   └── todotxt-cli/          ← scaffold only (Cargo.toml + src/main.rs stub)
├── Client/                   ← existing C# (untouched)
├── ToDoLib/                  ← existing C# (untouched)
└── ToDo.Net.sln
```

Key decisions:
- `Cargo.toml` at repo root, Rust crates under `crates/`
- Add `target/` to root `.gitignore`
- `todotxt-cli` scaffolded as empty stub so workspace compiles; no CLI logic until Phase 3

### Module Boundaries

| Module | Responsibility | Public exports |
|--------|---------------|----------------|
| `error.rs` | Error types | `TodoError` |
| `task.rs` | Parse + represent + serialize one todo.txt line | `Task`, `DueStatus`, `LineEnding` |
| `task_list.rs` | File I/O, CRUD, BOM/CRLF handling | `TaskList` |
| `lib.rs` | Re-exports | All of the above |

### No `parser.rs` Separate Module

The `winnow` parser is internal to `task.rs` — it parses a single line into a `Task`. There is no separate `parser.rs` module because:
1. The parser is tightly coupled to `Task` field construction
2. It's a single function (`Task::parse`) that returns a `Task` directly
3. Keeping it in `task.rs` avoids circular dependencies between parser output and `Task` struct

---

## Key Implementation Details

### Task::parse — Infallible, Single-Pass (CONTEXT.md Decision 2)

```rust
impl Task {
    pub fn parse(line: &str) -> Self { ... }  // NEVER returns Err
}
```

The C# parser uses sequential regex matching and mutation (`raw = reg.Replace(raw, "")`) in a specific order:
1. Completed status (`^X\s(\d{4}-\d{2}-\d{2})?`)
2. Priority (`^\([A-Z]\)\s`)
3. Due date (`due:YYYY-MM-DD`)
4. Threshold date (`t:YYYY-MM-DD`)
5. Creation date (`YYYY-MM-DD` after priority)
6. Projects (`\+\S+`)
7. Contexts (`@\S+`)
8. Remainder = body

The Rust `winnow` parser does this in a single pass. The key insight: the todo.txt format is positional at the start (completed marker, then priority, then creation date) but tag-based in the body (projects, contexts, due/threshold dates can appear anywhere).

**Parser strategy:**
1. Parse optional `x ` prefix + optional completion date (positional)
2. Parse optional `(A) ` priority (positional, uppercase only — fixes C# bug M-6)
3. Parse optional creation date `YYYY-MM-DD` (positional, immediately after priority)
4. Remainder is the body — scan for `+project`, `@context`, `due:YYYY-MM-DD`, `t:YYYY-MM-DD` tags
5. Store the original line as `raw` (private) for round-trip fidelity

**Relative date handling:** Before parsing, replace `due:today`, `due:tomorrow`, `due:monday` etc. with `due:YYYY-MM-DD` in the raw string (matches C# `ParseDate` behavior). Same for `t:today` etc.

### Task Struct (CONTEXT.md Decision 3)

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub struct Task {
    // Private — enforces invariant that raw and parsed fields are in sync
    #[serde(skip)]
    raw: String,

    pub completed: bool,
    pub priority: Option<char>,           // 'A'-'Z' only
    pub creation_date: Option<NaiveDate>,
    pub completion_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub threshold_date: Option<NaiveDate>,
    pub projects: Vec<String>,            // sorted, deduplicated, WITHOUT '+' prefix
    pub contexts: Vec<String>,            // sorted, deduplicated, WITHOUT '@' prefix
    pub body: String,                     // remainder text after stripping metadata
}
```

- `#[non_exhaustive]` prevents external struct literal construction
- `raw` is private with `pub fn to_raw(&self) -> &str` getter
- `Display` impl delegates to `to_raw()`
- `serde(skip)` on `raw` so JSON output shows parsed fields only

### Builder Mutation Pattern (CONTEXT.md Decision 4)

```rust
impl Task {
    pub fn with_completed(self, completed: bool) -> Self { ... }
    pub fn with_priority(self, priority: Option<char>) -> Self { ... }
    pub fn with_due_date(self, date: Option<NaiveDate>) -> Self { ... }
    pub fn with_creation_date(self, date: Option<NaiveDate>) -> Self { ... }
    pub fn with_threshold_date(self, date: Option<NaiveDate>) -> Self { ... }
}
```

Each `with_*` method:
1. Takes `self` by value (consumes the task)
2. Modifies the relevant field
3. Rebuilds `raw` from all fields
4. Returns the new `Task`

**Rebuild `raw` strategy:** When completion status changes, follow the C# `ToString()` logic:
- If marking complete: strip priority from raw, prepend `x YYYY-MM-DD `
- If marking incomplete: strip `x YYYY-MM-DD ` prefix
- For other mutations: reconstruct the full line from parsed fields

### TaskList — Atomic File I/O (Pitfall C-2)

```rust
pub struct TaskList {
    path: PathBuf,
    tasks: Vec<Task>,
    line_ending: LineEnding,    // detected on load
    has_bom: bool,              // detected on load (stripped, never re-written)
    preserve_whitespace: bool,  // default false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
}
```

**Load flow:**
1. Read entire file as bytes
2. Strip UTF-8 BOM (`\u{FEFF}`) from first line if present (Pitfall C-3)
3. Detect line ending by scanning first 4000 bytes (matches C# `GetPreferredFileLineEndingFromFile`)
4. Split by detected line ending
5. Parse each non-empty line as `Task::parse(line)` (or keep blank lines if `preserve_whitespace: true`)
6. Store `LineEnding` for use on save

**Save flow (atomic — Pitfall C-2):**
1. Create `tempfile::NamedTempFile` in same directory as target
2. Write all tasks, joining with detected `LineEnding`
3. `flush()` + `sync_all()`
4. `persist()` (atomic rename on POSIX; best-effort on Windows)

**CRUD operations:**
- `add(task: Task)` — append to `self.tasks`, then `save()`
- `update(index: usize, new_task: Task)` — replace at index, then `save()` (Pitfall C-1: index-based, not raw-string)
- `delete(index: usize)` — remove at index, then `save()` (Pitfall C-1: index-based)
- `reload()` — re-read file from disk into `self.tasks`

### BOM/CRLF Handling (CORE-07)

| Scenario | Load Behavior | Save Behavior |
|----------|--------------|---------------|
| File has UTF-8 BOM | Strip `\u{FEFF}` from first line | Never write BOM |
| File has CRLF endings | Detect, store `LineEnding::CrLf` | Write CRLF between lines |
| File has LF endings | Detect, store `LineEnding::Lf` | Write LF between lines |
| File has no newlines | Default to platform line ending | Use default |

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum TodoError {
    #[error("I/O error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("task not found at index {index}")]
    NotFound { index: usize },

    #[error("index {index} out of bounds (task count: {count})")]
    IndexOutOfBounds { index: usize, count: usize },
}
```

Note: No `Parse` variant needed because `Task::parse` is infallible (Decision 2). `TodoError` is only for I/O and index validation.

---

## C# Bugs Addressed in Phase 1

| Bug | C# Code | Rust Fix |
|-----|---------|----------|
| C-1: Raw-string identity | `t.Raw == task.Raw` in Delete/Update | `usize` index for all mutations |
| C-2: Non-atomic writes | `StreamWriter` directly over file | `tempfile::NamedTempFile::persist()` |
| C-3: BOM breaks parsing | No BOM handling | `strip_prefix('\u{FEFF}')` on first line |
| C-4: CRLF not preserved | Inconsistent line ending handling | Detect on load, preserve on save |
| C-6: Regex recompilation | `new Regex()` per task in constructor | `winnow` parser (no regex in hot path); `LazyLock<Regex>` for any supplemental patterns |
| m-5: Sequential regex mutation | `raw = reg.Replace(raw, "")` chain | `winnow` single-pass extraction |

---

## Don't Hand-Roll

| Component | Use This | Not This |
|-----------|----------|----------|
| Date types | `chrono::NaiveDate` | Custom date parsing |
| Error types | `thiserror` 2.0 derive macros | Manual `impl Display + Error` |
| Atomic writes | `tempfile::NamedTempFile::persist()` | Manual temp file + `fs::rename` |
| Parser | `winnow` 1.0.1 combinators | Regex sequential replacement |
| Snapshot tests | `insta` 1.47 | Manual golden file comparison |
| Parameterized tests | `rstest` 0.26 | Copy-paste test methods |

---

## Test Strategy

### Test Framework
- `rstest` for parameterized/table-driven tests
- `insta` for snapshot round-trip tests
- `tempfile::TempDir` for file I/O tests (not real paths)

### Must-Have Tests (Phase 1)

| Test | What it proves | Type |
|------|---------------|------|
| Round-trip fidelity | `Task::parse(line).to_string() == line` for every line in fixture | `insta` snapshot |
| All field parsing | Priority, projects, contexts, dates, body from a complex line | `rstest` parameterized |
| Completed task parsing | `x 2024-01-15 Task text` → `completed: true`, correct date | Unit |
| Priority case-sensitivity | `(A)` parsed, `(a)` treated as body text (spec compliance) | Unit |
| Duplicate task deletion | Two identical tasks, `delete(0)` removes first only | Unit (Pitfall C-1) |
| BOM stripping | File with `\u{FEFF}` header loads cleanly, first task parsed correctly | Integration |
| CRLF round-trip | CRLF file loads → saves as CRLF; LF file loads → saves as LF | Integration |
| Atomic write safety | Write succeeds with `NamedTempFile::persist()` | Integration |
| Blank line handling | `preserve_whitespace: true` keeps blanks; `false` drops them | Unit |
| Builder mutation | `task.with_completed(true).to_raw()` produces correct `x` prefix | Unit |
| Empty/whitespace lines | Parser handles gracefully (raw task with empty body) | Unit |
| Relative date replacement | `due:today` → `due:YYYY-MM-DD` before parsing | Unit |
| Projects/contexts extraction | `+proj1 +proj2 @ctx` → sorted, deduplicated lists | `rstest` |

### Test Fixtures

Create `crates/todotxt-core/tests/fixtures/todo.txt` with representative lines:
```
(A) 2024-01-15 Call dentist +Health @phone due:2024-01-31
x 2024-01-10 2024-01-05 Pay bills +Finance @home
(B) Write report +Work @office t:2024-02-01
Buy groceries +Personal @errands
x completed without date
A line with no structured data at all
(C) Task with multiple +proj1 +proj2 @ctx1 @ctx2 due:2024-03-15
```

---

## Common Pitfalls (Phase 1 Specific)

1. **Don't use `regex` in the parser hot path** — use `winnow` combinators exclusively for `Task::parse`. `LazyLock<Regex>` is acceptable for relative date pre-processing (runs once per line, before parsing).

2. **Don't forget `#[non_exhaustive]`** on `Task` and enums — prevents downstream crates from constructing/matching exhaustively (Pitfall M-1).

3. **Don't expose `Vec<Task>` in the API** — use `&[Task]` for getters, `impl Iterator` for iteration.

4. **Don't use `unwrap()` in library code** — propagate errors with `?`. Reserve `unwrap()` for test code only.

5. **`serde(rename_all = "snake_case")`** on `Task` — lock in the JSON field naming convention (Pitfall C-5, carry-forward decision).

6. **`winnow` parser must handle trailing whitespace** — the C# parser `Trim()`s at the end; the winnow parser should consume trailing whitespace but preserve it in `raw`.

---

## Validation Architecture

### Dimension Coverage

| Dimension | Applicable in Phase 1 | How |
|-----------|----------------------|-----|
| Correctness | Yes | Round-trip tests, field parsing tests |
| Robustness | Yes | BOM, CRLF, blank lines, malformed input |
| Data integrity | Yes | Atomic writes, duplicate deletion, index-based identity |
| Performance | Minimal | No regex in hot path (winnow); defer benchmarks to Phase 2 |
| Security | Minimal | No user input from network; file path validation only |
| Compatibility | Yes | C# interop: same file format, same line endings |
| API stability | Yes | `#[non_exhaustive]`, `&[Task]` not `&Vec<Task>` |

---

## RESEARCH COMPLETE

Phase 1 research covers all CORE-01, CORE-02, CORE-03, CORE-07 requirements with verified crate versions, architecture patterns from CONTEXT.md decisions, and C# bug fixes. Ready for planning.

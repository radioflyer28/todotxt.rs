# Phase 1 Context: Workspace Bootstrap + Core Library Foundation

**Phase:** 1 — Workspace Bootstrap + Core Library Foundation
**Created:** 2026-04-15
**Status:** Ready for planning

## Domain Boundary

Set up the Cargo workspace and implement the `todotxt-core` library crate: parser, Task model, TaskList CRUD with atomic writes, and BOM/CRLF handling. No CLI code in this phase. The library must be self-contained with a passing test suite before Phase 2 begins.

## Decisions

### 1. Workspace structure

**Decision:** `Cargo.toml` lives at the **repository root** alongside the existing C# files (`ToDo.Net.sln`, `Client/`, `ToDoLib/`). Rust crates are under `crates/`:

```
todotxt.net/
├── Cargo.toml              ← workspace root (new)
├── crates/
│   ├── todotxt-core/       ← Phase 1 library crate
│   └── todotxt-cli/        ← Phase 3 binary crate (scaffold only in Phase 1 if needed)
├── Client/                 ← existing C# WPF app (untouched)
├── ToDoLib/                ← existing C# library (untouched)
├── ToDoTests/              ← existing C# tests (untouched)
└── ToDo.Net.sln
```

**Rationale:** Standard Cargo workspace placement; rust-analyzer and `cargo` work from any directory without `cd`; C# solution is unaffected (VS doesn't open Cargo.toml).

**Implication for planner:** Add a root `.gitignore` entry for `target/` (Cargo build dir) and ensure `.gitignore` already excludes it. The `todotxt-cli` crate directory can be scaffolded (just `Cargo.toml` + `src/main.rs` stub) in Phase 1 so the workspace is complete, but no CLI logic is implemented until Phase 3.

### 2. Parser error policy

**Decision:** **Lenient** — `Task::parse()` never returns `Err` for a line it can't fully structure. Any line that can't be parsed as a standard todo.txt task is wrapped as a "raw task" with all structured fields defaulting to `None`/`false`/empty. Blank lines are governed separately by the `preserve_whitespace` setting on `TaskList`.

```rust
// Every line always produces a Task — no parse failure path
impl Task {
    pub fn parse(line: &str) -> Self { ... }  // infallible
}
```

**Rationale:** Never lose user data; round-trip fidelity is paramount. Matches todo.sh behavior. Existing todo.txt files from the C# app should load without errors even if they contain non-standard lines.

**Implication for planner:**
- `Task::parse` signature is `fn parse(line: &str) -> Task` (not `Result<Task, TodoError>`) — it never fails
- `TaskList::load()` returns `Result<TaskList, TodoError>` for IO errors only (file not found, permission denied, etc.) — NOT for parse errors
- Blank-line handling: `TaskList` stores blank lines as `Task` entries with `raw: ""` when `preserve_whitespace: true`; skips them when `preserve_whitespace: false` (default)
- `TodoError` still has a `Parse` variant — reserved for future strict-mode or for internal validation errors that indicate a programmer bug, not user data issues

### 3. Task field visibility

**Decision:** `raw` field is **private**; parsed fields (`completed`, `priority`, `creation_date`, etc.) are **public**. A `to_raw()` getter exposes the canonical string read-only.

```rust
pub struct Task {
    raw: String,                              // private — enforce invariant
    pub completed: bool,
    pub priority: Option<char>,
    pub creation_date: Option<NaiveDate>,
    pub completion_date: Option<NaiveDate>,
    pub due_date: Option<NaiveDate>,
    pub threshold_date: Option<NaiveDate>,
    pub projects: Vec<String>,
    pub contexts: Vec<String>,
    pub body: String,
}

impl Task {
    pub fn to_raw(&self) -> &str { &self.raw }
}
```

**Rationale:** Prevents callers from writing `task.raw = "bad data"` which would break the invariant that `raw` and parsed fields are in sync. All mutations must go through builder methods which rebuild `raw` correctly.

**Implication for planner:** `#[non_exhaustive]` on `Task` struct to prevent external struct literal construction. Implement `Display` for `Task` as an alias for `to_raw()` so `println!("{}", task)` works naturally.

### 4. Mutation pattern

**Decision:** **Builder/value-consuming** — mutation methods take `self` by value and return a new `Task`. Chainable, immutable by default.

```rust
impl Task {
    pub fn with_completed(self, completed: bool) -> Self { ... }
    pub fn with_priority(self, priority: Option<char>) -> Self { ... }
    pub fn with_due_date(self, date: Option<NaiveDate>) -> Self { ... }
    pub fn with_creation_date(self, date: Option<NaiveDate>) -> Self { ... }
    // etc.
}

// Usage in CLI commands:
let updated = task.with_completed(true);  // task consumed, updated is new Task
task_list.update(idx, updated)?;
```

**Rationale:** Invariant preservation — every `with_*` method rebuilds `raw` from the mutated fields before returning, so raw and parsed fields are always in sync. Immutable semantics make it impossible to forget to save.

**Implication for planner:** `TaskList::update(idx: usize, new_task: Task)` replaces the task at `idx` in the in-memory Vec and immediately triggers a save. All CLI mutation commands follow the pattern: `load → transform via with_*() → list.update(idx, transformed) → save`.

## Canonical Refs

These documents define the authoritative requirements and design for this phase. Downstream agents (researcher, planner) MUST read all of them:

- `.planning/ROADMAP.md` — Phase 1 requirements, deliverables, UAT criteria, verification steps
- `.planning/REQUIREMENTS.md` — CORE-01, CORE-02, CORE-03, CORE-07 requirement descriptions
- `.planning/research/STACK.md` — Verified crate versions: `winnow` 1.0.1, `thiserror` 2.0, `tempfile`, `rstest`, `insta`, `chrono` 0.4, `regex` 1.x
- `.planning/research/ARCHITECTURE.md` — Core library public API surface, component boundaries, data flow
- `.planning/research/PITFALLS.md` — Critical pitfalls C-1 through C-6 and m-5: must be addressed in this phase
- `.planning/codebase/ARCHITECTURE.md` — Existing C# architecture (reference for what Rust is porting)

## Carry-Forward Decisions (from project-level context)

These were decided before Phase 1 and are not subject to re-discussion:

| Decision | Value |
|----------|-------|
| Parser engine | `winnow` 1.0.1 (single-pass) |
| Error library | `thiserror` 2.0 |
| Task identity (internal) | Vec index (`usize`) — not raw-string comparison |
| Task identity (display) | 1-based line number — documented as shifting on delete |
| Atomic writes | `tempfile::NamedTempFile::persist()` then rename |
| BOM handling | Strip `\u{FEFF}` from first line on load; never write BOM |
| CRLF handling | Detect line ending on load; preserve on save |
| JSON field naming | `snake_case` |
| Crate names | `todotxt-core` (library), `todotxt-cli` (binary) |

## C# Bugs to Fix in Phase 1

All of these must be addressed in the implementation, not deferred:

| Bug ID | Bug | Fix |
|--------|-----|-----|
| C-1 | Raw-string identity causes silent deletion of duplicate tasks | Use Vec index (`usize`) for all delete/update operations |
| C-2 | Non-atomic file writes corrupt todo.txt on crash | `tempfile::NamedTempFile::persist()` + rename before any CLI code |
| C-3 | UTF-8 BOM on first line breaks parsing | `strip_prefix('\u{FEFF}')` on first line at load time |
| C-4 | CRLF line endings not preserved on save | Detect `LineEnding` on load; use it in `TaskList::save()` |
| C-6 | Regex recompilation per task tanks performance | `OnceLock<Regex>` / `LazyLock` for all compiled patterns |
| m-5 | Sequential regex mutation pattern (C# porting trap) | Do NOT port; `winnow` single-pass extracts all fields in one pass |

## Test Strategy

- **Framework:** `rstest` (parameterized tests) + `insta` (snapshot tests)
- **Must-have tests in Phase 1:**
  - Round-trip: every line in a `tests/fixtures/todo.txt` parses and serializes back byte-for-byte
  - Duplicate task deletion: TaskList with two identical tasks — `delete(0)` removes first, not both
  - BOM stripping: file with BOM header loads cleanly
  - CRLF round-trip: CRLF file loads and saves as CRLF
  - Atomic write: write succeeds even when `temp` dir is on the same filesystem as `todo.txt`
  - Blank line handling: `preserve_whitespace: true` keeps blank lines; `false` drops them

## Deferred Ideas

None raised during discussion.

# Phase 2 Context: Core Library Completion

**Phase:** 2 — Core Library Completion
**Created:** 2026-04-15
**Status:** Ready for planning

## Domain Boundary

Complete the `todotxt-core` public API with a filter engine, sort engine, file watcher, batch mutations, and portable mode. No CLI code in this phase. All work is inside `crates/todotxt-core/`.

## Prior Decisions (Locked from Phase 1)

The following decisions from Phase 1 context apply here and must not be revisited:

1. **Infallible parse** — `Task::parse()` never returns `Err`; blank lines handled by `preserve_whitespace`.
2. **Private `raw` field** — mutations go through builder methods (`with_*`) which rebuild `raw`.
3. **Index-based identity** — all CRUD (including batch) uses Vec index, not raw-string matching.
4. **`thiserror` 2.0 error types** — all public `Result` errors use `TodoError` variants.
5. **`winnow` 1.0 API** — `ModalResult` (not `PResult`), `prelude::*` for `Parser` trait.

## Decisions

### 1. Filter query format — space-separated tokens

**Decision:** `Filter::from_query(q: &str)` splits on ASCII whitespace. Each token is parsed independently. Multiple tokens are AND-combined (all must match — same as C# behavior).

```
Filter::from_query("@home -DONE due:today")
// → [Include("@home"), NotDone, DueToday]  — AND of all three
```

**Rationale:** C# used newline-separated tokens because of a multi-line text box GUI. A library/CLI should use space-separated tokens (todo.sh style, ergonomic for CLI args). AND semantics match C# exactly.

**Filter token types (exhaustive — matches C# `FilterList` logic):**

| Token | Behavior |
|-------|----------|
| `DONE` | Case-sensitive. Include only completed tasks |
| `-DONE` | Case-sensitive. Include only incomplete tasks |
| `due:today` | Case-insensitive. Include only tasks with due = today |
| `due:past` | Case-insensitive. Include only tasks overdue (due < today) |
| `due:future` | Case-insensitive. Include only tasks with due > today |
| `due:active` | Case-insensitive. Include tasks with due ≤ today (today + past) |
| `-due:today` | Case-insensitive. Exclude tasks with due = today |
| `-due:past` | Case-insensitive. Exclude overdue tasks |
| `-due:future` | Case-insensitive. Exclude future-due tasks |
| `-due:active` | Case-insensitive. Exclude tasks with due ≤ today |
| `-term` | Case-insensitive substring. Exclude if task.to_raw() contains `term` |
| `term` | Case-insensitive substring. Include only if task.to_raw() contains `term` |

**Note:** `DONE` and `-DONE` are case-sensitive by design (matches C# `StringComparison.Ordinal`). All other tokens are case-insensitive. The rationale: "done" can appear naturally in task body text; only uppercase `DONE` triggers the special completion filter.

**Implication for planner:**

```rust
pub struct Filter {
    pub terms: Vec<FilterTerm>,
    pub suppress_hidden: bool,            // default: true
    pub suppress_future_threshold: bool,  // default: true
}

pub enum FilterTerm {
    Include(String),
    Exclude(String),
    Done,
    NotDone,
    DueToday, DuePast, DueFuture, DueActive,
    NegDueToday, NegDuePast, NegDueFuture, NegDueActive,
}

impl Filter {
    pub fn from_query(q: &str) -> Self { ... }
    pub fn new() -> Self { ... }   // suppress_hidden: true, suppress_future_threshold: true
}
```

`TaskList::filter(&self, filter: &Filter) -> Vec<&Task>` — returns slice of references, does NOT mutate or save.

### 2. `h:1` and threshold suppression — filter-level flags, default on

**Decision:** Both `h:1` suppression and future-threshold suppression are flags on the `Filter` struct, defaulting to `true`. They are evaluated BEFORE token matching (same order as C# code).

```rust
// Default behavior: hidden and future-threshold tasks are excluded
let filter = Filter::from_query("@home");       // suppress_hidden: true, suppress_future_threshold: true

// Opt-out: see all tasks including hidden/future-threshold
let filter = Filter { suppress_hidden: false, suppress_future_threshold: false, ..Filter::from_query("@home") };
```

**Rationale:** C# applies these as global settings (`ShowHidenTasks`, `FilterFutureTasks`). Making them per-filter flags is more flexible — the library consumer (CLI config, TUI settings) can store the preference and pass it in. Default-on preserves the C# user experience.

**`h:1` suppression rule:** A task whose `to_raw()` contains `h:1` is excluded when `suppress_hidden: true`, regardless of other filter tokens.

**Threshold suppression rule:** A task with `threshold_date > today` is excluded when `suppress_future_threshold: true`, regardless of other filter tokens.

### 3. Sort orders — 5 per ROADMAP

**Decision:** `SortOrder` enum has exactly 5 variants matching ROADMAP requirements:

```rust
pub enum SortOrder {
    Priority,      // (A) before (B) before unprioritized; stable among ties
    DueDate,       // earliest first; tasks with no due date last; stable among ties
    Alphabetical,  // case-insensitive; stable among ties
    Project,       // first +Project token alphabetical; tasks with no project last
    Context,       // first @Context token alphabetical; tasks with no context last
}
```

**Stability:** All sort orders use Rust's stable sort (`slice::sort_by`). Tasks that compare equal preserve their original order. This matches LINQ's `OrderBy` behavior in C#.

**Backlog:** C# also has `Completed`, `CreatedDate`, and `None` (file order). These are intentionally deferred — they can be added to `SortOrder` in a later phase without breaking changes since the enum is `#[non_exhaustive]`.

**Implication for planner:** `TaskList::sort(&mut self, order: SortOrder)` sorts `self.tasks` in-place, does NOT save to disk. Caller decides when to save.

### 4. File watcher callback — simple `Fn()` signal

**Decision:** The watcher callback receives no arguments. The caller calls `task_list.reload()` to pick up changes. This keeps the watcher API simple and stateless.

```rust
pub struct FileWatcher {
    // internal debouncer state
}

impl FileWatcher {
    pub fn new(path: impl AsRef<Path>, callback: Arc<dyn Fn() + Send + Sync + 'static>) -> Result<Self, TodoError>;
    pub fn stop(self);
}
```

**Rationale:** `TaskList::reload()` already exists (from Phase 1). A richer `Fn(Vec<Task>)` callback would require the watcher to snapshot the file state, adding complexity for no clear benefit. Simple signal = caller reloads = correct.

**Feature flag:** `FileWatcher` and all `notify`/`notify-debouncer-mini` dependencies are gated behind Cargo feature `watching`. The crate compiles without file watching by default (no additional transitive deps).

**Debounce:** 1 second (matches ROADMAP and C# `FileChangeObserver` behavior).

### 5. Batch mutations API

**Decision:** `batch_update` takes a vec of `(index, Task)` replacement pairs. This is consistent with the Phase 1 `update(index, task)` pattern and preserves index-based identity.

```rust
impl TaskList {
    /// Replace multiple tasks atomically — one save() call at the end.
    pub fn batch_update(&mut self, replacements: Vec<(usize, Task)>) -> Result<(), TodoError>;
}
```

**Behavior:**
- Validate ALL indices before applying any replacement (fail-fast: if any index is out of bounds, return `IndexOutOfBounds` immediately, no partial update)
- Apply all replacements in a single pass
- Call `save()` once at the end

**Rationale:** Avoids N disk writes for N tasks (avoids write amplification). Atomic in the sense that either all replacements apply or none do (fail-fast validation).

**Note:** `reload()` was already implemented in Phase 1. The ROADMAP's mention of it as a Phase 2 deliverable is already satisfied.

## C# Reference Notes

Key behavioral details extracted from `Client/MainWindowViewModel.cs:FilterList()`:

- Multi-token filter uses **AND** logic — all tokens must be satisfied
- `h:1` suppression runs **before** token evaluation (pre-filter, not a token)
- Threshold suppression runs **before** token evaluation (pre-filter, not a token)
- `due:active` = tasks where `DueDate` is NOT null AND NOT > today (i.e. today + past)
- `DONE`/`-DONE` use `StringComparison.Ordinal` (case-sensitive)
- All other substring filters use `StringComparison.InvariantCultureIgnoreCase` (case-insensitive)
- C# sort types: Alphabetical, Completed, Context, DueDate, Priority, Project, None, Created — Rust gets 5 (Priority, DueDate, Alphabetical, Project, Context)

## Deferred Ideas

*(none — user skipped discussion, no scope-creep suggestions)*

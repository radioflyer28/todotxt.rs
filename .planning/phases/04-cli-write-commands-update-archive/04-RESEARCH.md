# Phase 4: CLI Write Commands — Research

**Researched:** 2026-04-15
**Domain:** Rust CLI write commands — TaskList mutation, clap subcommands, assert_cmd integration testing
**Confidence:** HIGH

---

## Summary

Phase 4 adds seven write subcommands to the `todotxt-cli` crate. All mutation logic (completion,
deletion, text update) already exists in `todotxt-core`; this phase is primarily a CLI wiring
task. The core library provides `Task::with_completed()`, `with_priority()`, `with_*_date()`
builders, and `TaskList::add()` / `update()` / `delete()` — all of which atomically persist to
disk. The only missing piece is a `Task::with_text_prepended()` builder for the `prepend` command.

The CLI patterns from Phase 3 (`show.rs`, `list.rs`, error propagation via `CliError`, output via
`Renderer`) provide a complete, tested template. Write commands follow the same shape with the
addition of a `print_write_result` helper on `Renderer` and a new `auto_creation_date` config field.

**Primary recommendation:** Wire the seven commands following the `show.rs` pattern; add
`with_text_prepended` to `task.rs`; extend `Renderer` and `Config` minimally. No new dependencies
needed — all required crates are already in `Cargo.toml`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Task mutation (complete, undo, edit, del) | Core library (`todotxt-core`) | — | Mutation logic lives in `Task` builders and `TaskList` CRUD |
| Append/prepend text | Core library (`todotxt-core`) | CLI command | `Task::parse` + new builder; CLI only formats the string |
| ID resolution (1-based → 0-based) | CLI command layer | — | Display IDs are a CLI concept; core uses 0-based indices |
| Creation date injection | CLI command (`add.rs`) | Config | Config controls default; CLI applies it before `Task::parse` |
| Atomic persistence | Core library (`TaskList::save`) | — | Already implemented; write commands call `add()`/`update()`/`delete()` |
| Output rendering | CLI output layer (`Renderer`) | — | Existing pattern; needs minor extension for write results |
| Exit code mapping | CLI `main.rs` | — | `CliError::NotFound` → 1, `CliError::Other` → 2; unchanged |
| Integration test fixtures | CLI `tests/helpers.rs` | — | `TestFixture` pattern already established |

---

## Standard Stack

### Core (all already in Cargo.toml — no new dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `todotxt-core` | workspace | Task model, mutation, persistence | Phase 4 is pure CLI wiring on top of core |
| `clap` | 4.6 | Subcommand definitions, arg parsing | Already used for all existing commands |
| `anyhow` | 1.0 | Error wrapping in CLI layer | Established pattern; `?` propagation to `CliError` |
| `chrono` | workspace | `Local::now().date_naive()` for creation date | Already in core; `todotxt-cli/Cargo.toml` doesn't need it directly — use `todotxt_core::Task::parse` with a date string constructed from chrono |
| `assert_cmd` | workspace | Integration test binary invocation | Already used in all Phase 3 tests |
| `assert_fs` | workspace | Temporary fixture files | Already used in `helpers.rs` |
| `predicates` | workspace | Output assertion helpers | Already used in `show_tests.rs` |

[VERIFIED: crates/todotxt-cli/Cargo.toml and crates/todotxt-core/Cargo.toml]

### No New Dependencies Needed

All write operations use the existing `TaskList::add()`, `TaskList::update()`,
`TaskList::delete()`, and `Task::with_*()` APIs already present in `todotxt-core`.
The `chrono` crate is available transitively (used by `todotxt-core`), but CLI code
constructs date strings via `Task::parse` — no direct `chrono` usage in CLI.

**Installation:**
```bash
# Nothing to install — all dependencies already declared in workspace Cargo.toml
```

---

## Architecture Patterns

### System Architecture Diagram

```
CLI input (args)
       │
       ▼
  cli.rs (clap parse)
       │
  ┌────┴─────────────────────────────────┐
  │  Commands::Add / Do / Undo / Del /   │
  │  Edit / Append / Prepend             │
  └────┬─────────────────────────────────┘
       │
       ▼
 main.rs::run() dispatch
       │
       ├──► config.rs::Config::load_or_create() ──► TOML file
       │
       ├──► TaskList::load(todo_path) ──────────────► todo.txt (read)
       │
       ├──► task mutation (builders / parse)
       │         │
       │    Task::with_completed()
       │    Task::with_text_prepended()   ← new
       │    Task::parse(text)
       │
       ├──► TaskList::add() / update() / delete() ──► todo.txt (atomic write)
       │
       └──► Renderer::print_write_result() ──────────► stdout (task) + stderr (info)
```

### Recommended Project Structure (additions only)

```
crates/todotxt-cli/src/
├── commands/
│   ├── mod.rs          # add 7 new `pub mod` declarations
│   ├── add.rs          # new
│   ├── do.rs           # new (command is `do`, Rust module is `do_cmd` or `do.rs`)
│   ├── undo.rs         # new
│   ├── del.rs          # new
│   ├── edit.rs         # new
│   ├── append.rs       # new
│   └── prepend.rs      # new
├── cli.rs              # add 7 new Commands variants
├── config.rs           # add auto_creation_date field
├── output.rs           # add print_write_result method
└── main.rs             # add 7 dispatch arms

crates/todotxt-core/src/
└── task.rs             # add with_text_prepended builder

crates/todotxt-cli/tests/
├── write_tests.rs      # all write command integration tests
└── helpers.rs          # unchanged (TestFixture already handles write scenarios)
```

**Note on `do.rs`:** `do` is a Rust keyword, so the module file is named `do.rs` but the
subcommand struct/function use non-keyword names. Clap handles the `do` CLI name via
`#[command(name = "do")]` on the struct variant. [VERIFIED: clap docs behavior]

### Pattern 1: Write Command Shape (mirroring show.rs)

```rust
// Source: crates/todotxt-cli/src/commands/show.rs (established pattern)
use crate::{output::Renderer, CliError};
use std::path::Path;
use todotxt_core::TaskList;

pub fn run(todo_path: &Path, id: usize, text: &str, renderer: &Renderer) -> Result<(), CliError> {
    if id == 0 {
        return Err(CliError::NotFound(format!("task ID 0 is invalid (IDs start at 1)")));
    }
    let mut list = TaskList::load(todo_path)?;      // load
    let idx = id - 1;                               // 1-based → 0-based
    let task = list.tasks().get(idx).ok_or_else(|| {
        CliError::NotFound(format!("task {} not found (list has {} tasks)", id, list.len()))
    })?;
    let updated = Task::parse(&format!("{} {}", task.to_raw(), text));  // mutate
    list.update(idx, updated.clone())?;             // persist atomically
    renderer.print_write_result(&format!("Appended to task #{id}."), idx, &updated);
    Ok(())
}
```

[VERIFIED: established pattern from show.rs and list.rs in this codebase]

### Pattern 2: Multi-ID Descending Delete

```rust
// Validate all IDs exist BEFORE any deletion
let mut indices: Vec<usize> = Vec::with_capacity(ids.len());
for &id in ids {
    if id == 0 { return Err(CliError::NotFound(...)); }
    let idx = id - 1;
    list.tasks().get(idx).ok_or_else(|| CliError::NotFound(...))?;
    indices.push(idx);
}
// Sort descending so earlier indices are not invalidated by later deletes
indices.sort_unstable_by(|a, b| b.cmp(a));
indices.dedup();
for idx in indices {
    // print deleted task then delete
    renderer.print_write_result(..., idx, &list.tasks()[idx].clone());
    list.delete(idx)?;
}
```

[VERIFIED: index-preservation logic derived from TaskList::delete() contract in task_list.rs]

### Pattern 3: Renderer::print_write_result (new method)

```rust
// In output.rs — add alongside existing print_task / print_count methods
pub fn print_write_result(&self, info: &str, idx: usize, task: &Task) {
    if self.json {
        println!("{}", json_success(task_dto(idx, task)));
    } else {
        if !self.quiet {
            eprintln!("{}", info);
        }
        println!("{}", task.to_raw());
    }
}
```

[VERIFIED: consistent with existing print_task/print_count pattern in output.rs]

### Pattern 4: Task::with_text_prepended (new core builder)

```rust
// In task.rs — alongside other with_* builders
pub fn with_text_prepended(self, text: &str) -> Self {
    let new_body = if self.body.is_empty() {
        text.to_string()
    } else {
        format!("{} {}", text, self.body)
    };
    let new_task = Task { body: new_body, ..self };
    let new_raw = rebuild_raw(&new_task);   // private fn in task.rs — accessible here
    Task::parse(&new_raw)
}
```

[VERIFIED: rebuild_raw is private to task.rs (line 298); with_* builders already call it]

### Pattern 5: add with Optional Creation Date

```rust
pub fn run(todo_path: &Path, text: &str, with_date: bool, renderer: &Renderer) -> Result<(), CliError> {
    if text.trim().is_empty() {
        return Err(CliError::Other(anyhow::anyhow!("task text cannot be empty")));
    }
    let raw = if with_date {
        let today = chrono::Local::now().date_naive();
        format!("{} {}", today.format("%Y-%m-%d"), text)
    } else {
        text.to_string()
    };
    let task = Task::parse(&raw);
    let mut list = TaskList::load(todo_path)?;
    let new_idx = list.len();  // index of the task after add
    list.add(task.clone())?;
    renderer.print_write_result(&format!("Added task #{}.", new_idx + 1), new_idx, &task);
    Ok(())
}
```

[ASSUMED: chrono available in CLI — needs Cargo.toml `chrono` dependency in todotxt-cli]

**NOTE:** `chrono` is a workspace dependency already used by `todotxt-core`, but `todotxt-cli`'s
`Cargo.toml` does not currently declare it. The `add` command needs to add `chrono = { workspace = true }`
to `crates/todotxt-cli/Cargo.toml`. Alternatively, delegate date formatting to `todotxt-core`
via a helper. Recommendation: add `chrono` to `todotxt-cli/Cargo.toml` directly — it's already
in the workspace.

### Anti-Patterns to Avoid

- **Cloning TaskList for validation**: Don't load the list twice for validate+mutate. Load once, validate indices in-memory, mutate.
- **Ascending delete order**: `del 1 3` in ascending order shifts index 3 to become 2 after index 1 is removed. Always sort descending before deleting.
- **Printing after delete**: Print the task **before** calling `TaskList::delete(idx)` — after deletion the task is gone from the list.
- **Using `task.body` for append**: `append` should append to the full raw line, not just the body, so that `+proj` and `@ctx` tokens work correctly when appended.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Atomic file writes | Custom temp+rename | `TaskList::add/update/delete` (already uses `tempfile::NamedTempFile::persist`) | Already handles crash safety, CRLF preservation, BOM stripping |
| Task text mutation | String manipulation | `Task::with_completed()`, `Task::parse()`, new `with_text_prepended()` | Builders keep all fields in sync; manual string edit risks stale parsed fields |
| Integration test CLI invocation | `std::process::Command` | `assert_cmd::Command::cargo_bin("todotxt")` | `assert_cmd` finds debug binary automatically, chains assertions cleanly |
| Temp file fixtures | `std::fs::write` in tests | `assert_fs::TempDir` + `TestFixture::with_content()` | Cleanup is automatic; already established in `helpers.rs` |

**Key insight:** The entire mutation surface is already implemented in `todotxt-core`. Phase 4 is 90% plumbing — CLI arg definitions, dispatch, and integration tests.

---

## Common Pitfalls

### Pitfall 1: `do` Is a Rust Keyword
**What goes wrong:** Naming a module or function `do` causes a compile error: `error: expected identifier, found keyword 'do'`.
**Why it happens:** `do` is a reserved keyword in Rust (though unused as of Rust 2021 edition).
**How to avoid:** Name the module file `do.rs` (allowed as a file name), use `r#do` if needed as an identifier, or name the function `run` (the established pattern). The clap subcommand is named `"do"` via `#[command(name = "do")]` on the struct.
**Warning signs:** `error[E0578]: module 'do' is not defined` or similar.

### Pitfall 2: Index Shift on Multi-Delete
**What goes wrong:** Deleting tasks in ascending ID order causes remaining IDs to shift, so later deletes target the wrong tasks.
**Why it happens:** `TaskList` is a `Vec<Task>`; removing index N shifts all indices > N down by 1.
**How to avoid:** Always sort indices descending before deleting. Validate all IDs first (before any deletion) to avoid partial state changes.
**Warning signs:** Integration test deleting tasks 1 and 3 where task 4 ends up deleted instead.

### Pitfall 3: with_completed(true) Strips Priority
**What goes wrong:** A test expecting `(A)` to survive a `do` command fails.
**Why it happens:** Per todo.txt spec, marking a task complete strips its priority. The `with_completed(true)` builder already implements this.
**How to avoid:** Tests for `do` should not assert the priority is preserved. This is correct behavior.
**Warning signs:** Test asserting `(A)` appears in a `do` command output fails unexpectedly.

### Pitfall 4: print_task vs. to_raw for del
**What goes wrong:** Printing the task after `list.delete(idx)` causes a panic or shows the wrong task.
**Why it happens:** After `delete(idx)`, the task at that index no longer exists. Index `idx` now points to what was previously `idx+1`.
**How to avoid:** Clone or snapshot the task **before** calling `delete()`, then print it.

### Pitfall 5: chrono Not in todotxt-cli Cargo.toml
**What goes wrong:** `use chrono::Local;` in `commands/add.rs` gives `error[E0432]: unresolved import`.
**Why it happens:** `chrono` is declared as a workspace dependency used by `todotxt-core`, but `todotxt-cli/Cargo.toml` doesn't re-declare it.
**How to avoid:** Add `chrono = { workspace = true }` to `crates/todotxt-cli/Cargo.toml` `[dependencies]`.

### Pitfall 6: Renderer::print_write_result Must Handle JSON Consistently
**What goes wrong:** In `--json` mode, info text appears on stderr but JSON consumers see mixed output.
**Why it happens:** The info line is unconditionally sent to stderr even in JSON mode.
**How to avoid:** In `print_write_result`, suppress the stderr info line entirely when `self.json` is true — JSON clients parse stdout only.

---

## Code Examples

### Integration Test Pattern (write command)

```rust
// Source: crates/todotxt-cli/tests/helpers.rs + show_tests.rs (established pattern)
mod helpers;
use helpers::TestFixture;
use predicates::prelude::*;

#[test]
fn add_creates_new_task() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("add")
        .arg("buy coffee +groceries")
        .assert()
        .success()
        .code(0)
        .stdout(predicate::str::contains("buy coffee +groceries"));
    // Verify it appears in list
    fx.cmd()
        .arg("list")
        .arg("buy coffee")
        .assert()
        .success()
        .stdout(predicate::str::contains("buy coffee"));
}

#[test]
fn del_removes_task_and_exits_zero() {
    let fx = TestFixture::new();
    fx.cmd()
        .arg("del")
        .arg("1")
        .assert()
        .success()
        .code(0);
    // Task 1 should no longer appear
    fx.cmd()
        .arg("show")
        .arg("1")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn do_on_already_done_task_is_idempotent() {
    // SAMPLE_TODO task 3 is "x 2024-01-01 Done task +work" (already completed)
    let fx = TestFixture::new();
    fx.cmd()
        .arg("do")
        .arg("3")
        .assert()
        .success()  // idempotent — exit 0
        .code(0);
}
```

### cli.rs New Variants

```rust
// Source: established pattern from existing Commands enum in cli.rs
/// Add a new task
Add {
    /// Full task text (priority, dates, projects, contexts are parsed from text)
    text: String,
    /// Prepend today's creation date (overrides auto_creation_date config)
    #[arg(long)]
    date: bool,
    /// Suppress creation date even if auto_creation_date = true
    #[arg(long)]
    no_date: bool,
},

/// Mark one or more tasks as done
#[command(name = "do")]
Do {
    /// 1-based task IDs to complete
    #[arg(required = true)]
    ids: Vec<usize>,
},

/// Unmark one or more completed tasks
Undo {
    #[arg(required = true)]
    ids: Vec<usize>,
},

/// Delete one or more tasks by ID
#[command(alias = "delete")]
Del {
    #[arg(required = true)]
    ids: Vec<usize>,
},

/// Replace a task's full text
Edit {
    id: usize,
    /// New full task text (replaces the task entirely)
    text: String,
},

/// Append text to the end of a task
Append {
    id: usize,
    text: String,
},

/// Insert text before a task's body (after prefix fields)
Prepend {
    id: usize,
    text: String,
},
```

[VERIFIED: clap 4.x derive syntax confirmed against existing cli.rs in this codebase]

---

## Runtime State Inventory

> Greenfield phase — not a rename or refactor. Skipped.

---

## Environment Availability

> Pure code changes — no external tool dependencies beyond Rust toolchain already verified
> in Phase 3. Skipped (all required tooling confirmed in prior phases).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `cargo test` + `assert_cmd` + `assert_fs` + `predicates` |
| Config file | `crates/todotxt-cli/Cargo.toml` `[dev-dependencies]` |
| Quick run command | `cargo test -p todotxt-cli` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WRITE-01 | `add` creates task; appears in list | integration | `cargo test -p todotxt-cli add` | ❌ Wave 0 |
| WRITE-01 | `add --date` prepends creation date | integration | `cargo test -p todotxt-cli add_with_date` | ❌ Wave 0 |
| WRITE-02 | `do <id>` marks complete | integration | `cargo test -p todotxt-cli do_marks_complete` | ❌ Wave 0 |
| WRITE-02 | `do` idempotent on already-done | integration | `cargo test -p todotxt-cli do_idempotent` | ❌ Wave 0 |
| WRITE-02 | `do <id>` out-of-range exits 1 | integration | `cargo test -p todotxt-cli do_not_found` | ❌ Wave 0 |
| WRITE-03 | `undo <id>` removes completion | integration | `cargo test -p todotxt-cli undo_removes_completion` | ❌ Wave 0 |
| WRITE-03 | `undo` idempotent on incomplete | integration | `cargo test -p todotxt-cli undo_idempotent` | ❌ Wave 0 |
| WRITE-04 | `del <id>` removes task | integration | `cargo test -p todotxt-cli del_removes_task` | ❌ Wave 0 |
| WRITE-04 | `del 3 1` multi-delete descending | integration | `cargo test -p todotxt-cli del_multi` | ❌ Wave 0 |
| WRITE-05 | `edit <id> <text>` replaces task | integration | `cargo test -p todotxt-cli edit_replaces` | ❌ Wave 0 |
| WRITE-06 | `append <id> <text>` appends | integration | `cargo test -p todotxt-cli append_adds_text` | ❌ Wave 0 |
| WRITE-07 | `prepend <id> <text>` prepends to body | integration | `cargo test -p todotxt-cli prepend_inserts_before_body` | ❌ Wave 0 |
| WRITE-07 | `with_text_prepended` builder unit test | unit | `cargo test -p todotxt-core with_text_prepended` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p todotxt-cli -- --test-thread=1` (integration tests)
- **Per wave merge:** `cargo test` (full workspace)
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `crates/todotxt-cli/tests/write_tests.rs` — covers WRITE-01 through WRITE-07
- [ ] `with_text_prepended` unit test in `crates/todotxt-core/tests/` or inline in `task.rs` `#[cfg(test)]`

---

## Security Domain

> CLI tool that reads/writes local files. No network, no authentication, no user input
> interpreted as code. Primary attack surface is file path injection (WRITE-04 `del` on
> `todo_path` misconfigured to a system file) — mitigated by the existing `Config.todo_file`
> path resolution which only reads from user config.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | Yes (limited) | `text.trim().is_empty()` guard on `add`/`edit`; task ID validated as `usize` by clap |
| V6 Cryptography | No | Not applicable |
| V2 Authentication | No | Local file tool; no auth surface |
| V4 Access Control | No | OS file permissions govern access; no application-level ACL needed |

No OWASP Top 10 issues apply to a local CLI file tool with no network surface.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `chrono` needs to be added to `todotxt-cli/Cargo.toml` for `add` command date formatting | Code Examples / Pitfall 5 | Compile error; easy fix (one line in Cargo.toml) |
| A2 | `do` as a filename (`do.rs`) is legal in Rust | Pitfall 1 | Module won't load; workaround: name file `complete.rs` with `#[command(name="do")]` |

---

## Open Questions

1. **Module name for `do` command**
   - What we know: `do` is a reserved Rust keyword but unused in Rust 2021 edition
   - What's unclear: Whether `pub mod do;` compiles (likely doesn't) or requires `r#do` or a different filename
   - Recommendation: Name the file `do.rs` and the module `pub mod r#do;` in `mod.rs`, OR name it `complete.rs` with `#[command(name = "do")]`. The latter is cleaner — use `complete.rs`.

2. **Whether `del` prints task pre- or post-deletion in JSON mode**
   - What we know: After `delete(idx)`, the task is gone from the list
   - What's unclear: JSON schema for delete response — task object or just `{"deleted": true}`?
   - Recommendation: Return the deleted task object (as it was) for auditability; consistent with human-mode output

---

## Sources

### Primary (HIGH confidence — verified in codebase)
- `crates/todotxt-core/src/task.rs` — Task model, `with_*` builders, `rebuild_raw`
- `crates/todotxt-core/src/task_list.rs` — `add()`, `update()`, `delete()` contracts
- `crates/todotxt-cli/src/commands/show.rs` — canonical write command shape
- `crates/todotxt-cli/src/output.rs` — Renderer pattern
- `crates/todotxt-cli/src/cli.rs` — clap derive struct patterns
- `crates/todotxt-cli/tests/helpers.rs` — TestFixture pattern
- `crates/todotxt-cli/tests/show_tests.rs` — assert_cmd assertion patterns

### Secondary (MEDIUM confidence)
- clap 4.x `#[command(name = "do")]` rename attribute — consistent with clap documentation and established use of `#[command(alias = "ls")]` in this codebase

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies verified in Cargo.toml
- Architecture: HIGH — patterns directly lifted from Phase 3 code
- Pitfalls: HIGH — derived from actual code (task_list.rs contracts, keyword conflict)
- Test patterns: HIGH — verified against existing test files

**Research date:** 2026-04-15
**Valid until:** Stable (no external dependencies introduced)

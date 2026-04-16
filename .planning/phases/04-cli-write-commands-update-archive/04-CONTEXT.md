# Phase 4: CLI Write Commands — Context

**Phase:** 04 — CLI Write Commands
**Milestone:** v1.0 — Core Library + CLI
**Status:** Ready for Planning
**Date:** 2026-04-15

---

## Scope

Phase 4 implements the full task lifecycle write commands for the CLI.

**In scope (WRITE-01 through WRITE-07):**
- `add <text>` — create a new task
- `do <id>...` — mark one or more tasks complete
- `undo <id>...` — unmark one or more completed tasks
- `del <id>...` — delete one or more tasks
- `edit <id> <new text>` — replace a task's full text
- `append <id> <text>` — append text to the end of a task
- `prepend <id> <text>` — prepend text before a task's body

**Out of scope (deferred):**
- `pri <id> <A-Z>` / `depri <id>` — priority enrichment → Phase 5
- `due <id> <date>` / `postpone <id> <N>` — due date enrichment → Phase 5
- `archive` / `del-done` — bulk operations → Phase 5

---

## Decisions

### D-01: Creation Date Auto-Prepend
`add` auto-prepends today's creation date (`YYYY-MM-DD`) when `auto_creation_date = true` in config (default: **false**). Overridable per-invocation with `--date` (force prepend) and `--no-date` (suppress even if config is true).

**Rationale:** Agent use cases need explicit control. Default off avoids unexpected mutation of user text. Callers can pass `--date` when the date is needed.

### D-02: Multi-ID Support for do / undo / del
`do`, `undo`, and `del` accept one or more 1-based IDs: `todotxt do 1 3 5`. When multiple IDs are given, they are processed in **descending index order** to preserve correctness during deletion (higher indices first so lower indices don't shift).

**Rationale:** Batch operations are the primary use case for agent automation. Descending order prevents index-shift bugs on multi-delete.

### D-03: Idempotent do / undo Semantics
- `do <id>` on an already-completed task: print info to stderr, skip, **exit 0** (idempotent)
- `undo <id>` on an already-incomplete task: print info to stderr, skip, **exit 0** (idempotent)
- Any ID out of range: **exit 1** (NotFound), matching `show` behavior

**Rationale:** Idempotent writes are retry-safe for agent automation. Exit 0 signals "the desired state is already in effect," not "nothing happened."

### D-04: Output After Write
Every write command:
1. Prints the resulting task (post-mutation) to **stdout** via `renderer.print_task(idx, task)`
2. Prints a brief info line to **stderr** (e.g., "Added task #5.") unless `--quiet`
3. In JSON mode: returns the resulting task in the standard `{"schema_version":1,"data":...}` envelope; no stderr info line

For `del`, the **deleted** task is printed (as it was before deletion) to stdout.

**Rationale:** Consistent with read commands; lets agents capture the result for verification.

### D-05: append Semantics
`append <id> <text>` appends ` <text>` to the **end of the raw task line**, then re-parses via `Task::parse()`. Any `+project`, `@context`, `due:`, `t:` tags in the appended text are structured normally.

**Implementation:** `Task::parse(&format!("{} {}", task.to_raw(), text))`

### D-06: prepend Semantics
`prepend <id> <text>` inserts `<text> ` **before the body** (after completion marker, priority, and creation/completion date prefixes). Tags in the prepended text are parsed and sorted with existing tags.

**Implementation:** A new `Task::with_text_prepended(text: &str) -> Self` builder in `todotxt-core` that prepends to the `body` field and calls `rebuild_raw` + `Task::parse`.

### D-07: edit Semantics
`edit <id> <new text>` replaces the task's raw line entirely: `Task::parse(new_text)`. No fields are preserved from the original. The caller provides the complete replacement text including any priority, dates, or tags they want to keep.

**Rationale:** Full replacement gives agents predictable, complete control. Partial field preservation would require complex merge logic with ambiguous semantics.

### D-08: del Does Not Confirm
`del` deletes immediately without any confirmation prompt. This is intentional.

**Rationale:** Interactive prompts are anti-features for scriptable/agent-driven CLIs.

### D-09: Renderer Extension for Write Results
Add `Renderer::print_write_result(info: &str, idx: usize, task: &Task)` to `output.rs`:
- Human mode: print `info` to stderr (unless `--quiet`), then print task to stdout
- JSON mode: print task in JSON envelope; no info line

### D-10: Config Field for auto_creation_date
Add `auto_creation_date: bool` (default `false`) to `Config` struct with `#[serde(default)]`. No migration needed — TOML defaults to `false` when field is absent.

---

## Claude's Discretion

- Exact wording of info messages ("Added task #N." vs "TODO: task #N added.")
- Whether `del` prints the task pre- or post-deletion to stdout (recommend: print as it was — useful as a confirmation)
- Whether `do` accepts `done` as a subcommand alias
- Integration test file organization (one file per command vs. grouped write-commands file)
- Whether to add `delete` as an alias for `del`

---

## Deferred Ideas (OUT OF SCOPE)

- Priority commands (`pri`, `depri`) → Phase 5
- Due date commands (`due`, `postpone`) → Phase 5
- `archive` and `del-done` bulk operations → Phase 5
- Interactive confirmation/undo-redo history → REJECTED (not in requirements, anti-feature)
- Transactional multi-command rollback → REJECTED (atomic writes per-operation are sufficient)

---

## Requirements

### W-01: add Command

**Spec:** `todotxt add "<task text>"`

Behavior:
- Accepts one positional argument: the full task text
- Optionally auto-prepends today's date per D-01
- Calls `TaskList::add(Task::parse(text))`
- Prints the added task to stdout (its 1-based ID = `list.len()` after add)
- Prints "Added task #N." to stderr unless `--quiet`

Flags:
- `--date` — force-prepend creation date (overrides config)
- `--no-date` — suppress creation date even if `auto_creation_date = true`

Edge cases:
- Empty text string: exit 2 with error
- Text containing tags (`+proj`, `@ctx`, `due:2026-04-15`): parsed normally by `Task::parse`
- Text that already starts with a valid priority `(A) `: accepted as-is

### W-02: do Command

**Spec:** `todotxt do <id> [<id>...]`

Behavior:
- Accepts 1+ 1-based IDs as positional arguments
- Processes IDs in descending order
- For each ID:
  - Out of range → `CliError::NotFound`, exit 1
  - Already completed → print info to stderr, skip (idempotent)
  - Incomplete → `task.with_completed(true)`, `TaskList::update(idx, mutated)`, print result
- Multi-ID: loads TaskList once, applies all mutations, saves after each (atomic per-mutation)

### W-03: undo Command

**Spec:** `todotxt undo <id> [<id>...]`

Behavior:
- Accepts 1+ 1-based IDs
- For each ID:
  - Out of range → `CliError::NotFound`, exit 1
  - Already incomplete → print info to stderr, skip (idempotent)
  - Completed → `task.with_completed(false)`, `TaskList::update(idx, mutated)`, print result

### W-04: del Command

**Spec:** `todotxt del <id> [<id>...]`

Behavior:
- Accepts 1+ 1-based IDs
- Validates all IDs exist before deleting any (fail fast with exit 1 if any ID invalid)
- Processes in descending index order
- For each ID: print deleted task to stdout, then `TaskList::delete(idx)`

Note: Validate-then-delete avoids partial deletes on bad input.

### W-05: edit Command

**Spec:** `todotxt edit <id> "<new text>"`

Behavior:
- Accepts task ID and new text as positional arguments
- `TaskList::update(idx, Task::parse(new_text))`
- Prints updated task to stdout; prints "Edited task #N." to stderr unless `--quiet`
- Empty new text: exit 2

### W-06: append Command

**Spec:** `todotxt append <id> "<text>"`

Behavior:
- `let updated = Task::parse(&format!("{} {}", task.to_raw(), text))`
- `TaskList::update(idx, updated)`
- Prints updated task to stdout; prints "Appended to task #N." to stderr unless `--quiet`

### W-07: prepend Command

**Spec:** `todotxt prepend <id> "<text>"`

Behavior:
- `let updated = task.with_text_prepended(text)` (new core builder)
- `TaskList::update(idx, updated)`
- Prints updated task to stdout; prints "Prepended to task #N." to stderr unless `--quiet`

---

## Core Library Changes Required

### New: Task::with_text_prepended

```rust
/// Insert `text` before the task body (after all prefix fields).
///
/// Tags in `text` (`+proj`, `@ctx`, `due:`, `t:`) are parsed and merged with
/// existing tags. The raw line is rebuilt and re-parsed for full consistency.
pub fn with_text_prepended(self, text: &str) -> Self {
    let new_body = if self.body.is_empty() {
        text.to_string()
    } else {
        format!("{} {}", text, self.body)
    };
    let new_task = Task { body: new_body, ..self };
    let new_raw = rebuild_raw(&new_task);
    Task::parse(&new_raw)
}
```

(`rebuild_raw` is private to `task.rs` — this method lives there.)

### Config Addition: auto_creation_date

```rust
/// Automatically prepend today's creation date when adding tasks.
/// Can be overridden per-invocation with --date / --no-date flags.
#[serde(default)]
pub auto_creation_date: bool,
```

---

## New CLI Files Required

| File | Command | Core API Used |
|------|---------|---------------|
| `commands/add.rs` | `add` | `Task::parse`, `TaskList::add` |
| `commands/do.rs` | `do` | `Task::with_completed(true)`, `TaskList::update` |
| `commands/undo.rs` | `undo` | `Task::with_completed(false)`, `TaskList::update` |
| `commands/del.rs` | `del` | `TaskList::delete` |
| `commands/edit.rs` | `edit` | `Task::parse`, `TaskList::update` |
| `commands/append.rs` | `append` | `Task::parse` (format!), `TaskList::update` |
| `commands/prepend.rs` | `prepend` | `Task::with_text_prepended`, `TaskList::update` |

**Modified files:**
- `crates/todotxt-core/src/task.rs` — add `with_text_prepended` builder
- `crates/todotxt-cli/src/cli.rs` — add 7 new subcommand variants
- `crates/todotxt-cli/src/main.rs` — add 7 dispatch arms
- `crates/todotxt-cli/src/config.rs` — add `auto_creation_date` field
- `crates/todotxt-cli/src/output.rs` — add `print_write_result` method
- `crates/todotxt-cli/src/commands/mod.rs` — declare 7 new modules
- `crates/todotxt-cli/tests/` — integration tests for all write commands

---

## Success Criteria

### Functional
- [ ] `todotxt add "buy milk"` creates a new task; `list` shows it
- [ ] `todotxt add "buy milk" --date` creates task with today's `YYYY-MM-DD` creation date
- [ ] `todotxt do 1` marks task 1 complete: `x YYYY-MM-DD` prefix added, priority stripped
- [ ] `todotxt do 1 2 3` marks all three tasks complete in one invocation
- [ ] `todotxt do 1` on already-done task: exit 0, no change
- [ ] `todotxt undo 1` removes `x YYYY-MM-DD` prefix from a completed task
- [ ] `todotxt undo 1` on already-incomplete task: exit 0, no change
- [ ] `todotxt del 1` removes task 1; subsequent `list` does not show it
- [ ] `todotxt del 3 1` removes both tasks correctly (no index-shift corruption)
- [ ] `todotxt edit 1 "new text"` replaces task 1 entirely
- [ ] `todotxt append 1 "+work"` appends `+work` to task 1
- [ ] `todotxt prepend 1 "URGENT:"` inserts before task 1's body, after any dates/priority
- [ ] All write commands respect `--json`, `--no-color`, `--quiet` global flags
- [ ] All write commands print resulting task to stdout
- [ ] Exit code 1 for out-of-range IDs; exit code 2 for file errors

### Technical
- [ ] `cargo test -p todotxt-core` passes (new `with_text_prepended` tests)
- [ ] `cargo test -p todotxt-cli` passes (write command integration tests)
- [ ] `cargo clippy -- -D warnings` passes (both crates)
- [ ] All new command files follow the `show.rs` / `list.rs` pattern

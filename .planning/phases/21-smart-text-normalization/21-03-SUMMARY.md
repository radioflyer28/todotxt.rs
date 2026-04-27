# Plan 21-03 Summary: TUI Edit Wiring + CLI --normalize Flag

**Status:** ✅ COMPLETE

**Commit:** fc8db88

**Changes:** 5 files, 148 insertions, 7 deletions

## Implementation Details

### Task 1: Wire save_and_exit Editing arm (COMPLETE)
**Location:** `crates/todotxt-tui/src/app.rs`

Updated imports to include `normalize_line`:
```rust
use todotxt_core::{Filter, SortOrder, Task, TaskList, normalize_append, normalize_line};
```

Restructured `save_and_exit` to move `let task = Task::parse(&text)` into per-arm construction:
- **`AppMode::Adding` arm:** Always uses `Task::parse(&text)` — T-21-07 (normalize_edit must not affect new task creation; no "original" to merge into)
- **`AppMode::Editing` arm:** Branches on `self.config.normalize_edit`:
  - When `true` (default): calls `normalize_line(&text)` — lifts inline priority tokens to canonical field position
  - When `false`: falls back to `Task::parse(&text)` — current behavior preserved
- T-21-06 mitigation: `normalize_line` (not `normalize_append`) used for editing — operates on the full replacement line, avoids body-doubling

### Task 2: CLI --normalize flag (COMPLETE)

**`crates/todotxt-cli/src/cli.rs`:**
Added `normalize: bool` field to `Commands::Append` variant:
```rust
/// Parse and normalize todo.txt tokens in TEXT (priority, +project, @context,
/// due:, t:) into canonical field positions instead of raw string concat.
/// Without this flag, existing behavior (raw append) is unchanged.
#[arg(long)]
normalize: bool,
```

**`crates/todotxt-cli/src/commands/append.rs`:**
- Added `normalize_append` to imports: `use todotxt_core::{normalize_append, Task, TaskList};`
- Updated `pub fn run` signature: added `normalize: bool` parameter
- Replaced raw concat with branch:
  ```rust
  let updated = if normalize {
      normalize_append(&task, text)
  } else {
      Task::parse(&format!("{} {}", task.to_raw(), text))
  };
  ```

**`crates/todotxt-cli/src/main.rs`:**
Updated Append dispatch to destructure and forward normalize flag:
```rust
Commands::Append { id, text, normalize } => {
    commands::append::run(&todo_path, *id, text, *normalize, &renderer)?
}
```

## Testing Results

**Build:** `cargo build -p todotxt-tui` — ✅ clean
**Build:** `cargo build -p todotxt-cli` — ✅ clean
**Workspace tests:** 380 passing, 0 failures
**Help text:** `cargo run -p todotxt-cli -- append --help` shows `--normalize` flag

## Phase 21 Requirements Coverage

| Requirement | Status | Delivered By |
|-------------|--------|--------------|
| NORM-01: Priority normalization during append | ✅ | Plan 21-01 (normalize_append) |
| NORM-02: Project/context deduplication | ✅ | Plan 21-01 (normalize_append BTreeSet) |
| NORM-03: Date field precedence | ✅ | Plan 21-01 (normalize_append) |
| NORM-04: Inline priority lifting during edit | ✅ | Plans 21-01 + 21-03 (normalize_line wired) |
| NORM-05: Unknown token preservation | ✅ | Plan 21-01 (body passthrough) |
| NORM-06: Config toggle for opt-out | ✅ | Plans 21-02 + 21-03 (config fields wired) |

All three normalization call sites complete:
- **TUI append:** wired in Plan 21-02 via `normalize_append`
- **TUI edit:** wired in Plan 21-03 via `normalize_line`
- **CLI append:** wired in Plan 21-03 via `normalize_append` + `--normalize` flag

---
phase: 01-workspace-bootstrap-core-library-foundation
plan: 02
status: complete
commit: 32a4eb5
tests_added: 13
tests_total: 46
---

# Plan 01-02 Execution Summary

## Objective

Implement `TaskList` with atomic file I/O, BOM/CRLF handling, and index-based CRUD — completing the core library's data layer.

## Tasks Completed

### Task 1: TaskList struct with atomic file I/O and CRUD

**Files created/modified:**
- `crates/todotxt-core/src/task_list.rs` — new (218 lines)
- `crates/todotxt-core/src/lib.rs` — updated to export `TaskList` and `LineEnding`
- `crates/todotxt-core/Cargo.toml` — moved `tempfile` from dev-dependencies to regular dependencies

**Key implementation decisions:**

| Decision | Rationale |
|----------|-----------|
| `tempfile::NamedTempFile::persist()` for save | Fixes C-2: atomic write prevents partial file on crash or kill |
| Index-based delete/update | Fixes C-1: `delete(0)` removes only the first task when two identical tasks exist |
| BOM strip on load, never write BOM | Fixes C-3: `strip_prefix('\u{FEFF}')` on first load; save never emits BOM |
| CRLF/LF detection from first 4000 bytes | Fixes C-4: matches C# `GetPreferredFileLineEndingFromFile()` algorithm exactly |
| `tasks(&self) -> &[Task]` returns slice | Per RESEARCH.md: API contract uses slice, not `&Vec<Task>` |

**C# bugs fixed:**

- **C-1 (Raw-string identity)**: C# `Delete` used `Tasks.First(t => t.Raw == task.Raw)` — silently removes wrong task when duplicates exist. Rust uses `Vec::remove(index)` — always removes exactly the indexed task.
- **C-2 (Non-atomic write)**: C# `WriteAllTasksToFile()` opened with `StreamWriter`, truncating the file before writing. Rust uses `NamedTempFile::persist()` for guaranteed atomic replacement.
- **C-3 (BOM)**: C# `StreamReader` transparently stripped BOM but `StreamWriter` could re-add it. Rust strips on load and never writes BOM.
- **C-4 (CRLF)**: Rust explicitly detects and preserves line endings. `str::lines()` was avoided — it normalises endings silently.

**Deviation from plan:**
- `tempfile` was originally only a dev-dependency. Moved to regular dependencies because `task_list.rs` uses `NamedTempFile` in library code.

### Task 2: TaskList integration tests

**File created:**
- `crates/todotxt-core/tests/task_list_tests.rs` — new (185 lines, 13 tests)

**All 13 tests pass:**

| Test | What it verifies |
|------|-----------------|
| `load_and_parse` | Parse 3 tasks, field values correct |
| `load_bom_stripping` | UTF-8 BOM stripped on load; save writes no BOM |
| `crlf_round_trip` | CRLF detected and preserved on save |
| `lf_round_trip` | LF preserved, no CRLF introduced |
| `add_task` | Append to list, persisted to disk |
| `delete_by_index` | Remove task[1], surrounding tasks intact |
| `update_by_index` | Replace task[1], other tasks unchanged |
| `duplicate_task_deletion` | **C-1 proof**: two identical tasks, `delete(0)` removes only first |
| `delete_out_of_bounds` | Returns `TodoError::IndexOutOfBounds { index: 5, count: 3 }` |
| `update_out_of_bounds` | Returns `TodoError::IndexOutOfBounds { index: 10, count: 2 }` |
| `atomic_write_creates_file` | File exists with correct content after save |
| `preserve_whitespace` | `false` drops blanks (3 tasks); `true` keeps them (5 tasks) |
| `reload_picks_up_external_changes` | External write + reload returns new content |

All tests use `tempfile::TempDir` for isolation — no test writes to real filesystem paths.

## Verification

```
cargo test -p todotxt-core    → 46/46 pass (33 parser + 13 TaskList)
cargo clippy -p todotxt-core  → 0 warnings
```

## Self-Check

- [x] `task_list.rs` contains `pub struct TaskList`
- [x] `task_list.rs` contains `pub fn load(`
- [x] `task_list.rs` contains `pub fn save(&self)`
- [x] `task_list.rs` contains `pub fn add(&mut self`
- [x] `task_list.rs` contains `pub fn update(&mut self, index: usize`
- [x] `task_list.rs` contains `pub fn delete(&mut self, index: usize`
- [x] `task_list.rs` contains `NamedTempFile` (atomic write)
- [x] `task_list.rs` contains `\u{FEFF}` / `strip_prefix` (BOM handling)
- [x] `task_list.rs` contains `pub fn tasks(&self) -> &[Task]`
- [x] `lib.rs` contains `pub mod task_list`
- [x] `lib.rs` contains `pub use task_list::TaskList`
- [x] `cargo check -p todotxt-core` exits 0
- [x] `cargo test -p todotxt-core` exits 0 (46/46)
- [x] `cargo clippy -p todotxt-core -- -D warnings` exits 0

---
phase: 01-workspace-bootstrap-core-library-foundation
plan: 01-01
type: summary
status: complete
completed: 2026-04-15
commits:
  - hash: 4a0829c
    description: "feat(01-01): scaffold cargo workspace, error types, task struct with winnow parser"
  - hash: f3c2a6f
    description: "feat(01-01): add Task tests, fixtures, and insta snapshots"
---

# Plan 01-01 Execution Summary

**Phase:** 01 — Workspace Bootstrap + Core Library Foundation
**Plan:** 01-01 — Wave 1: Cargo workspace + Task struct with winnow parser
**Status:** Complete
**Requirements addressed:** CORE-01, CORE-02

---

## What Was Built

### Task 1: Cargo Workspace Scaffold

Created the Rust workspace at the repository root, alongside the existing C# files:

- `Cargo.toml` — workspace root with `[workspace.dependencies]` shared across crates (winnow 1.0.1, chrono 0.4, thiserror 2.0, serde 1.0, tempfile 3)
- `crates/todotxt-core/Cargo.toml` — core library crate with winnow, chrono, thiserror, serde + dev-deps rstest/insta/tempfile
- `crates/todotxt-core/src/error.rs` — `TodoError` enum with `Io`, `NotFound`, `IndexOutOfBounds` variants via `thiserror` 2.0
- `crates/todotxt-core/src/lib.rs` — public re-exports for `Task`, `DueStatus`, `TodoError`
- `crates/todotxt-cli/Cargo.toml` — CLI stub crate, not yet implemented
- `crates/todotxt-cli/src/main.rs` — placeholder `main()` function
- `.gitignore` — appended `/target/` Rust build directory

### Task 2: Task Struct with Winnow Parser

Implemented `crates/todotxt-core/src/task.rs`:

**`Task` struct** — `#[non_exhaustive]` with private `raw` field and public parsed fields:
- `completed: bool`, `priority: Option<char>`, `creation_date`, `completion_date`, `due_date`, `threshold_date: Option<NaiveDate>`
- `projects: Vec<String>`, `contexts: Vec<String>` (sorted, deduplicated via BTreeSet)
- `body: String`

**`Task::parse(line: &str) -> Self`** — infallible single-pass parser using winnow combinators:
1. Completed marker (`x ` prefix — lowercase only, case-sensitive)
2. Completion date (YYYY-MM-DD prefix, only when completed)
3. Priority (`(A)` pattern, uppercase only [A-Z], not matched for lowercase)
4. Creation date (YYYY-MM-DD prefix)
5. Body tag extraction (+proj, @ctx, due:YYYY-MM-DD, t:YYYY-MM-DD)

**Builder methods** (value-consuming per CONTEXT.md Decision 4):
- `with_completed(self, bool)` — strips priority on completion, sets today's date
- `with_priority(self, Option<char>)` — adds/removes/changes `(X) ` prefix
- `with_due_date(self, Option<NaiveDate>)` — adds/removes/updates `due:YYYY-MM-DD`
- `with_creation_date(self, Option<NaiveDate>)` — adds/removes/updates creation date
- `with_threshold_date(self, Option<NaiveDate>)` — adds/removes/updates `t:YYYY-MM-DD`

**`due_status(&self) -> DueStatus`** — returns `Overdue`, `Today`, or `NotDue`

**`Display for Task`** — delegates to `to_raw()` for transparent string representation

**Test suite** (`crates/todotxt-core/tests/task_tests.rs`):
- 33 tests total, all passing
- Round-trip property test (all 10 fixture lines survive parse→to_string)
- Field extraction tests for all field types
- `#[rstest]` parameterized priority/completion case-sensitivity tests
- Builder mutation tests for all `with_*` methods
- `DueStatus` tests
- 2 insta snapshot tests (display + JSON)

---

## Deviations

| # | Area | Planned | Actual | Reason |
|---|------|---------|--------|--------|
| 1 | winnow imports | `PResult`, `Parser` | `ModalResult` (winnow 1.0), `prelude::*` | winnow 1.0.1 renamed `PResult` to `ModalResult` — updated imports accordingly |
| 2 | `Display` impl | `impl std::fmt::Display` | `impl fmt::Display` (via `use std::fmt`) | Idiomatic Rust — identical semantics |
| 3 | insta snapshots | First-run auto-accept | Used `INSTA_UPDATE=always` env var | `cargo-insta` CLI not installed; env var achieves same result |

---

## Key Files Created

| File | Lines | Purpose |
|------|-------|---------|
| `Cargo.toml` | 12 | Workspace root with shared dependencies |
| `crates/todotxt-core/Cargo.toml` | 17 | Core crate manifest |
| `crates/todotxt-core/src/error.rs` | 18 | TodoError enum |
| `crates/todotxt-core/src/lib.rs` | 5 | Public API re-exports |
| `crates/todotxt-core/src/task.rs` | 300+ | Task struct, parser, builders, Display |
| `crates/todotxt-core/tests/task_tests.rs` | 250+ | Test suite (33 tests) |
| `crates/todotxt-core/tests/fixtures/todo.txt` | 10 | Test fixture lines |
| `crates/todotxt-core/tests/snapshots/*.snap` | — | Insta snapshot baselines |

---

## Verification Results

```
cargo check --workspace        → ✓ passes (0 errors, 0 warnings)
cargo test -p todotxt-core     → ✓ 33/33 tests pass
cargo clippy -p todotxt-core -- -D warnings  → ✓ 0 warnings
```

Round-trip property: all 10 fixture lines satisfy `Task::parse(line).to_string() == line`.

---

## Self-Check: PASSED

All acceptance criteria from both tasks verified:
- Cargo.toml contains `[workspace]` ✓
- `TodoError` defined with all three variants ✓
- `Task` with `#[non_exhaustive]` and `pub fn parse` ✓
- `to_raw()`, `with_completed()`, `with_priority()`, `due_status()` ✓
- `Display for Task` impl ✓
- 10-line test fixture ✓
- 33 tests pass, 0 clippy warnings ✓

## Next Phase Readiness

Plan 01-02 can proceed — `Task::parse()`, `to_raw()`, `TodoError` are all available for `TaskList` to consume.

Types exported from `todotxt-core`:
- `Task` — parse, to_raw, builders, due_status
- `DueStatus` — NotDue, Today, Overdue
- `TodoError` — Io, NotFound, IndexOutOfBounds

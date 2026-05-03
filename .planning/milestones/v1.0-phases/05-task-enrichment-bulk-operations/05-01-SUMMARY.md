---
phase: 05
plan: 01
subsystem: CLI Infrastructure
status: COMPLETE
tags:
  - Config extension
  - CLI wiring
  - Command module stubs
  - Foundation work
dependencies:
  requires: []
  provides:
    - Config.done_file field
    - CLI enum with 6 new commands
    - Command dispatch wiring in main.rs
    - Compilable stubs for pri/depri/due/postpone/archive/del-done
  affects:
    - 05-02 (Date parsing)
    - 05-03/04 (Priority & due-date commands)
    - 05-05 (Archive & cleanup commands)
    - 05-06 (Integration tests)
tech_stack:
  added:
    - Config extension pattern (done_file: Option<PathBuf>)
    - Multi-ID command pattern (pri, depri with Vec<usize>)
    - Single-ID command pattern (due, postpone with usize)
    - No-arg command pattern (archive, del-done)
  patterns:
    - todo!() stubs for future implementation
    - #[serde(default)] for optional config fields
    - #[command(name = "...")] for CLI name overrides
key_files:
  created:
    - crates/todotxt-cli/src/commands/priority.rs (run_pri, run_depri)
    - crates/todotxt-cli/src/commands/due.rs (run_due, run_postpone)
    - crates/todotxt-cli/src/commands/archive.rs (run_archive)
    - crates/todotxt-cli/src/commands/del_done.rs (run_del_done)
  modified:
    - crates/todotxt-cli/src/config.rs (added done_file field)
    - crates/todotxt-cli/src/cli.rs (added 6 command variants)
    - crates/todotxt-cli/src/commands/mod.rs (added 4 mod declarations)
    - crates/todotxt-cli/src/main.rs (added dispatch logic for 6 commands)
duration: ~15 minutes
completed_date: 2026-04-16

---

# Phase 05 Plan 01: CLI Infrastructure — SUMMARY

## One-Liner

Extended Config struct with `done_file` field, wired 6 new enrichment/bulk commands into CLI dispatcher, created compilable command module stubs.

## Objective

Establish the foundation for Phase 5 — prepare CLI infrastructure (Config extension, command enum, dispatch logic) to support priority manipulation, due-date management, and bulk archive/cleanup commands. All stubs compile without errors.

## Execution Summary

### Task 1: Extend Config with done_file field ✓

**Status:** COMPLETE

- Added `done_file: Option<PathBuf>` field to `Config` struct with `#[serde(default)]` attribute
- Located after `auto_creation_date` field (line 24 in config.rs)
- Initialized to `None` in all 3 Config literal constructors:
  - Line 69: `load_or_create()` auto-creation
  - Line 140: Test `resolve_todo_file_returns_err_when_none()`
  - Line 150: Test `resolve_todo_file_returns_path_when_set()`
- No compile errors introduced

**Verification:**
```
✓ grep -n "done_file.*Option.*PathBuf" → found at line 24
✓ grep -c "done_file: None" → 3 occurrences
✓ cargo build -p todotxt-cli → No errors
```

### Task 2: Wire 6 new command enum variants into CLI ✓

**Status:** COMPLETE

Added to `Commands` enum in cli.rs (lines 96–128):
1. `Pri { ids: Vec<usize>, priority: char }` — Set priority for multiple tasks
2. `Depri { ids: Vec<usize> }` — Remove priority from multiple tasks
3. `Due { id: usize, date: String }` — Set due date on a single task
4. `Postpone { id: usize, days: u32 }` — Move due date forward by N days
5. `Archive` — Archive all completed tasks to done.txt
6. `DelDone` (name="del-done") — Delete all completed tasks

**Design choices:**
- `Pri` and `Depri` accept `Vec<usize>` per D-01 (multi-ID support for bulk operations)
- `Due` and `Postpone` accept single `usize` (per D-01, applies only to pri/depri)
- `Due` takes `date: String` (parsed in implementation phase)
- `Archive` and `DelDone` take no arguments
- Used `#[command(name = "...")]` for `DelDone` to map Rust identifier to CLI name

**Verification:**
```
✓ All 6 variants compile without errors
✓ All variants appear in CLI help output (12 total mentions)
✓ cargo clippy -p todotxt-cli -- -D warnings → 0 warnings
```

### Task 3: Create command module stubs and wire dispatch ✓

**Status:** COMPLETE

**Module declarations added** (commands/mod.rs, alphabetically sorted):
```rust
pub mod priority;   // run_pri, run_depri
pub mod due;        // run_due, run_postpone
pub mod archive;    // run_archive
pub mod del_done;   // run_del_done
```

**Stub files created:**

1. **priority.rs** (2 functions)
   - `pub fn run_pri(_todo_path: &Path, ids: &[usize], _priority: char, _renderer: &Renderer) -> Result<(), CliError>`
   - `pub fn run_depri(_todo_path: &Path, ids: &[usize], _renderer: &Renderer) -> Result<(), CliError>`
   - Both validate `ids.is_empty()` and return `todo!()`

2. **due.rs** (2 functions)
   - `pub fn run_due(_todo_path: &Path, _id: usize, _date: &str, _renderer: &Renderer) -> Result<(), CliError>`
   - `pub fn run_postpone(_todo_path: &Path, _id: usize, _days: u32, _renderer: &Renderer) -> Result<(), CliError>`
   - Both return `todo!()`

3. **archive.rs** (1 function)
   - `pub fn run_archive(_todo_path: &Path, _cfg: &Config, _renderer: &Renderer) -> Result<(), CliError>`
   - Takes `&Config` (needed for done_file resolution per D-02)
   - Returns `todo!()`

4. **del_done.rs** (1 function)
   - `pub fn run_del_done(_todo_path: &Path, _renderer: &Renderer) -> Result<(), CliError>`
   - Returns `todo!()`

**Dispatch logic wired** (main.rs, lines 48–56):
```rust
Commands::Pri { ids, priority } => commands::priority::run_pri(&todo_path, ids, *priority, &renderer)?,
Commands::Depri { ids } => commands::priority::run_depri(&todo_path, ids, &renderer)?,
Commands::Due { id, date } => commands::due::run_due(&todo_path, *id, date, &renderer)?,
Commands::Postpone { id, days } => commands::due::run_postpone(&todo_path, *id, *days, &renderer)?,
Commands::Archive => commands::archive::run_archive(&todo_path, &cfg, &renderer)?,
Commands::DelDone => commands::del_done::run_del_done(&todo_path, &renderer)?,
```

**Verification:**
```
✓ All 4 module files created
✓ All 6 dispatch arms compile without errors
✓ Dispatch uses correct parameter passing (derefs for pointers, moves for scalars)
✓ config module is available in main.rs (already imported)
```

## Compilation & Verification

```
$ cargo build -p todotxt-cli 2>&1
   Compiling todotxt-cli v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 2.61s

$ cargo clippy -p todotxt-cli -- -D warnings 2>&1
    Checking todotxt-cli v0.1.0
    Finished check [unoptimized + debuginfo] target(s) in 1.08s
```

**Result:** ✓ No compilation errors, ✓ No clippy warnings

## Compliance

### Must-Haves

| Item | Status |
|------|--------|
| All 6 new enrichment/bulk commands are wired into CLI dispatcher | ✓ PASS |
| Config supports `done_file` field for custom done.txt path resolution | ✓ PASS |
| `Config` has `done_file: Option<PathBuf>` with `#[serde(default)]` | ✓ PASS |
| `Commands` enum has Pri/Depri/Due/Postpone/Archive/DelDone variants | ✓ PASS |
| `commands/mod.rs` declares all 4 new modules | ✓ PASS |
| Code compiles without errors | ✓ PASS |
| Code has no clippy warnings | ✓ PASS |

### Requirements Traceability

- [x] **ENRICH-01** — Pri command wired (implementation deferred to 05-03)
- [x] **ENRICH-02** — Depri command wired (implementation deferred to 05-03)
- [x] **ENRICH-03** — Due command wired (implementation deferred to 05-04)
- [x] **ENRICH-04** — Postpone command wired (implementation deferred to 05-04)
- [x] **BULK-01** — Archive command wired (implementation deferred to 05-05)
- [x] **BULK-02** — DelDone command wired (implementation deferred to 05-05)

## Deviations from Plan

None — plan executed exactly as written. All tasks completed atomically with no auto-fixes or scope adjustments needed.

## Known Stubs

All 4 command modules contain `todo!()` placeholders that will panic if executed:

| File | Functions | Status | Planned Implementation |
|------|-----------|--------|---|
| priority.rs | run_pri, run_depri | Stub | 05-03 |
| due.rs | run_due, run_postpone | Stub | 05-04 |
| archive.rs | run_archive | Stub | 05-05 |
| del_done.rs | run_del_done | Stub | 05-05 |

These are intentional — the plan goal is infrastructure wiring, not implementation. Each command will be fully implemented in subsequent plans per the wave structure.

## Key Decisions

- **D-01 carried forward:** Multi-ID support for `pri` and `depri` (matches Phase 4 pattern)
- **D-02 carried forward:** `done_file` field resolves to parent of todo_file if unset
- **Config struct pattern:** Followed existing `auto_creation_date` field pattern with `#[serde(default)]`
- **CLI enum pattern:** Matched existing subcommand patterns (e.g., `Do`, `Undo`, `Del`)
- **Dispatch pattern:** Followed Phase 4 write command dispatch structure

## Next Steps

Plan 05-02 (Date Parsing Utility) will implement `chrono` integration and date parsing logic. Plans 05-03 through 05-06 will implement each command group in parallel waves.

---

## Self-Check

- [x] all created files exist
- [x] all modified files verified
- [x] all compilation checks passed
- [x] all verification commands passed
- [x] dispatch logic compiles and type-checks
- [x] git status shows all expected changes

**READY FOR COMMIT**

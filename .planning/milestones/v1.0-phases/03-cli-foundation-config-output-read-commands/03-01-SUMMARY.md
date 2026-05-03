---
phase: 03-cli-foundation-config-output-read-commands
plan: 01
subsystem: cli
tags: [clap, anyhow, owo-colors, comfy-table, directories, toml, serde_json, config, output]

requires:
  - phase: 02-core-library-completion
    provides: "resolve_config_path (portable mode), Task/TaskList/Filter public API"

provides:
  - "Config struct with TOML load_or_create, portable mode, preset support"
  - "Renderer with json/quiet flags, print_tasks, print_count, info, error"
  - "json_success / json_error envelope helpers (schema_version: 1)"
  - "init_color() for --no-color and NO_COLOR env var"
  - "All Phase 3 workspace dependencies declared"

affects: [03-02, 03-03, all-subsequent-cli-plans]

tech-stack:
  added:
    - clap 4.6 (derive API)
    - clap_complete 4.6
    - anyhow 1.0
    - owo-colors 4 (supports-colors feature)
    - comfy-table 7 (NOTHING preset)
    - directories 6 (ProjectDirs, BaseDirs)
    - toml 0.8
    - assert_cmd 2
    - assert_fs 1
    - predicates 3
    - tempfile (workspace, CLI dev-dep)
  patterns:
    - "Binary crate: mod declarations in main.rs with #[allow(dead_code)] until wired in Plan 02"
    - "Config auto-create on first run with create_dir_all before write (P-08)"
    - "JSON envelope: {schema_version:1, data:T} and {schema_version:1, error:str}"
    - "Color: init_color() called once at startup; if_supports_color for per-string checks"
    - "TaskDto: separate DTO for JSON; not derived on core Task"

key-files:
  created:
    - crates/todotxt-cli/src/config.rs
    - crates/todotxt-cli/src/output.rs
    - crates/todotxt-cli/tests/config_tests.rs
  modified:
    - Cargo.toml
    - crates/todotxt-cli/Cargo.toml
    - crates/todotxt-cli/src/main.rs

key-decisions:
  - "Used #[allow(dead_code)] on mod declarations in main.rs; removes when Plan 02 wires them"
  - "Unit tests inside config.rs (tempfile); integration test in tests/ pending Plan 02 wiring"
  - "dirs_home() uses directories::BaseDirs (consistent with rest of config module)"
  - "Priority color: A=red, B=yellow, C=green, D+=white via owo-colors fg() types"

patterns-established:
  - "Config: load_or_create(path) is the single entry point; never construct Config directly in CLI"
  - "Output: always route through Renderer; never call println!/eprintln! directly in commands"
  - "JSON: use json_success/json_error helpers; never hand-build JSON strings"

requirements-completed: [CFG-01, CFG-02, READ-06, READ-07]

duration: 25min
completed: 2026-04-15
---

# Phase 03 Plan 01: Config + Output Foundation Summary

**TOML config with portable mode and auto-create, plus Renderer/JSON-envelope/comfy-table output layer — all Phase 3 workspace dependencies declared**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-04-15T22:25:00Z
- **Completed:** 2026-04-15T22:50:00Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Declared all Phase 3 workspace and CLI dependencies (`cargo check` passes clean)
- `Config` struct with `load_or_create` (auto-creates file + dirs on first run), portable mode via `resolve_config_path`, preset support
- `Renderer` with JSON/quiet flags, `print_tasks`, `print_count`, `info`, `error`; `json_success`/`json_error` envelope helpers; `init_color` for `--no-color` + `NO_COLOR`
- 10 unit tests passing (6 in config.rs, 4 in output.rs)

## Task Commits

Each task was committed atomically:

1. **Task 1: Update Cargo.toml files with all Phase 3 dependencies** — `b4aeba4` (feat)
2. **Task 2: Implement config.rs — Config struct, load_or_create, portable mode, presets** — `f69bbe1` (feat)
3. **Task 3: Implement output.rs with Renderer, JSON helpers, comfy-table** — `c0c8670` (feat)

**Plan metadata:** *(this commit)*

## Files Created/Modified

- `Cargo.toml` — added 10 new workspace dependencies (clap, anyhow, owo-colors, comfy-table, directories, toml, assert_cmd, assert_fs, predicates, clap_complete)
- `crates/todotxt-cli/Cargo.toml` — full replacement; all deps wired as workspace = true; dev-deps for testing
- `crates/todotxt-cli/src/main.rs` — added `mod config; mod output;` with `#[allow(dead_code)]`
- `crates/todotxt-cli/src/config.rs` — Config, PresetConfig, load_or_create, resolve_todo_file, resolve_path, portable mode
- `crates/todotxt-cli/src/output.rs` — Renderer, init_color, json_success, json_error, build_task_table, TaskDto
- `crates/todotxt-cli/tests/config_tests.rs` — integration test (pending Plan 02 binary wiring)

## Decisions Made

- `#[allow(dead_code)]` on `mod config;` / `mod output;` in main.rs: binary crate dead-code lint fires on `pub` items not called from `main()`. Attributes removed when Plan 02 wires the modules.
- Integration test in `config_tests.rs` uses `assert_cmd` to invoke the binary with `--config`/`--todo-file`/`list`. Test fails until Plan 02 implements CLI dispatch — this is accepted per plan notes.
- `dirs_home()` private helper uses `directories::BaseDirs` (consistent crate choice over the unmaintained `dirs` crate).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing critical functionality] Add `mod config;`/`mod output;` to main.rs**
- **Found during:** Task 2
- **Issue:** `src/config.rs` not compiled without a module declaration; unit tests and clippy can't run
- **Fix:** Added `#[allow(dead_code)] mod config;` and `#[allow(dead_code)] mod output;` to main.rs. Plan said "do not touch main.rs yet" but compilation requires module declaration
- **Files modified:** `crates/todotxt-cli/src/main.rs`
- **Committed in:** `f69bbe1`

**2. [Rule 1 - Bug] Fixed `format!("(A)")` → `"(A)"` in output.rs**
- **Found during:** Task 3 (clippy)
- **Issue:** `cargo clippy -D warnings` flagged useless `format!` calls on string literals
- **Fix:** Replaced `format!("(A)")` etc. with bare `"(A)"` string literals; kept `format!("({p})")` for the variable case
- **Files modified:** `crates/todotxt-cli/src/output.rs`
- **Committed in:** `c0c8670`

**3. [Rule 2 - Missing dev-dep] Added `tempfile` to CLI dev-dependencies**
- **Found during:** Task 2
- **Issue:** Unit tests in config.rs used `tempfile::tempdir()` but it wasn't in CLI's dev-dependencies
- **Fix:** Added `tempfile = { workspace = true }` to `[dev-dependencies]` in `crates/todotxt-cli/Cargo.toml`
- **Files modified:** `crates/todotxt-cli/Cargo.toml`
- **Committed in:** `f69bbe1`

---

**Total deviations:** 3 auto-fixed
**Impact on plan:** All auto-fixes necessary for correctness/compilation. No scope creep.

## Issues Encountered

None beyond the deviations documented above.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

Plan 02 can now:
- Import `config::Config` and call `Config::load_or_create()` 
- Import `output::{Renderer, init_color, json_success, json_error}`
- Wire clap CLI struct (`cli.rs`) and dispatch commands using Renderer for output
- The integration test in `config_tests.rs` will pass once `--config`/`--todo-file`/`list` are wired

---
*Phase: 03-cli-foundation-config-output-read-commands*
*Completed: 2026-04-15*

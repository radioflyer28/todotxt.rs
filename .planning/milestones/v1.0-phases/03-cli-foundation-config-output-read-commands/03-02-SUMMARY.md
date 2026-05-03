---
phase: 03-cli-foundation-config-output-read-commands
plan: 02
subsystem: cli
tags: [clap, cli, dispatch, commands, list, stats, projects, contexts, show]

requires:
  - phase: 03-cli-foundation-config-output-read-commands
    plan: 01
    provides: "Config, Renderer, init_color, json_success/json_error, output foundation"

provides:
  - "Cli struct with 5 global flags (todo_file, config, json, no_color, quiet)"
  - "Commands enum: List(ListArgs), Stats, Projects, Contexts, Show{id}, Completions{shell}"
  - "CliError: NotFound (exit 1) / Other (exit 2) with blanket From<E: Into<anyhow::Error>>"
  - "main.rs dispatch loop: init_color before dispatch, exit 0/1/2 mapping"
  - "list command: preset resolution (:name), Filter::from_query, print_count footer"
  - "stats command: 5 stat fields with chrono due-date comparison"
  - "projects/contexts commands: BTreeSet sorted +/@ prefixed output"
  - "show command: CliError::NotFound for id=0 and out-of-range IDs"
  - "output.rs: Stats struct, print_task, print_lines, print_stats methods"

affects: [03-03]

tech-stack:
  added:
    - chrono (workspace dep added to todotxt-cli — needed for stats due-date comparison)
  patterns:
    - "CliError blanket From<E: Into<anyhow::Error>> for ? operator ergonomics"
    - "Preset resolution: :name prefix strips colon, looks up cfg.presets HashMap"
    - "Filter composition: preset query parts + positional tokens + --filter joined then from_query"
    - "BTreeSet for deduplication + alphabetic sort of project/context tags"
    - "print_count() called after print_tasks() in list for task count footer"

key-files:
  created:
    - crates/todotxt-cli/src/cli.rs
    - crates/todotxt-cli/src/commands/mod.rs
    - crates/todotxt-cli/src/commands/completions.rs
    - crates/todotxt-cli/src/commands/list.rs
    - crates/todotxt-cli/src/commands/stats.rs
    - crates/todotxt-cli/src/commands/projects.rs
    - crates/todotxt-cli/src/commands/contexts.rs
    - crates/todotxt-cli/src/commands/show.rs
  modified:
    - crates/todotxt-cli/src/main.rs
    - crates/todotxt-cli/src/output.rs
    - crates/todotxt-cli/Cargo.toml

key-decisions:
  - "clap_complete::Shell (top-level re-export) used instead of clap_complete::aot::Shell — both work in 4.6.2"
  - "chrono added as direct CLI dep (not just transitive) for stats::run date_naive() call"
  - "info/error marked #[allow(dead_code)] — reserved for Plan 03 (config auto-create notice) and future plans"
  - "print_count() wired in list.rs — natural footer per D-14 (suppressed by --quiet)"
  - "Stats struct added to output.rs (was in Plan interfaces but not Plan 01 output)"

requirements-completed: [READ-01, READ-02, READ-03, READ-04, READ-05, READ-08]

duration: 25min
completed: 2026-04-15
---

# Phase 03 Plan 02: CLI Dispatch + Read Commands Summary

**clap derive CLI with global flags and exit-code dispatch wired to list, stats, projects, contexts, show commands — working todotxt binary**

## Performance

- **Duration:** ~25 min
- **Started:** 2026-04-15
- **Completed:** 2026-04-15
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- `cli.rs`: `Cli` struct with 5 global flags (`todo_file`, `config`, `json`, `no_color`, `quiet`) + `Commands` enum with all 6 subcommands; `ListArgs` with positional filters and `--filter` flag
- `main.rs`: full rewrite — `CliError` (NotFound/Other), `run()` dispatch, `main()` with `init_color` before dispatch and `process::exit(1/2)` error mapping
- `commands/mod.rs`: 6 module declarations + stub `completions.rs` (Plan 03)
- `list.rs`: preset resolution (`:name` prefix), `Filter::from_query` composition, `print_tasks` + `print_count`
- `stats.rs`: chrono date comparison for `due_today`/`overdue`, `Stats` struct via output.rs
- `projects.rs` / `contexts.rs`: `BTreeSet` dedup with `+`/`@` prefix, sorted output
- `show.rs`: 1-based ID validation, `CliError::NotFound` for `id=0` and out-of-range
- `output.rs`: added `Stats` struct + `print_task`, `print_lines`, `print_stats` methods
- All 11 tests passing, 0 clippy warnings (`-D warnings`)

## Task Commits

1. **Task 1: Create cli.rs** — `c90d4d0` (feat)
2. **Task 2: Rewrite main.rs + commands module** — `35dcb15` (feat)
3. **Task 3: Implement all 5 read commands** — `1d4d6a9` (feat)

## Files Created/Modified

- `crates/todotxt-cli/src/cli.rs` — Cli, Commands, ListArgs (created)
- `crates/todotxt-cli/src/main.rs` — full rewrite: CliError, run(), main()
- `crates/todotxt-cli/src/output.rs` — Stats struct, print_task, print_lines, print_stats added
- `crates/todotxt-cli/src/commands/mod.rs` — 6 pub mod declarations (created)
- `crates/todotxt-cli/src/commands/completions.rs` — stub for Plan 03 (created)
- `crates/todotxt-cli/src/commands/list.rs` — preset + filter composition (created)
- `crates/todotxt-cli/src/commands/stats.rs` — 5 stat fields with due-date logic (created)
- `crates/todotxt-cli/src/commands/projects.rs` — BTreeSet +prefix output (created)
- `crates/todotxt-cli/src/commands/contexts.rs` — BTreeSet @prefix output (created)
- `crates/todotxt-cli/src/commands/show.rs` — 1-based ID, NotFound handling (created)
- `crates/todotxt-cli/Cargo.toml` — added chrono dep

## Decisions Made

- `clap_complete::Shell` (top-level re-export) used instead of `clap_complete::aot::Shell` — both paths exist in 4.6.2; plan action used the simpler form and it compiled.
- `chrono` added as direct CLI dependency; it was in the workspace but not declared in `crates/todotxt-cli/Cargo.toml`, required for `stats.rs` `chrono::Local::now().date_naive()`.
- `#[allow(dead_code)]` on `info` and `error` in `output.rs` — these are reserved for config auto-create notices (D-01) and future error-message routing, not yet wired to any Plan 02 command.
- `print_count()` called in `list.rs` after `print_tasks()` to show task count footer (suppressed by `--quiet` per D-14).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing methods] Added Stats struct + print_task/print_lines/print_stats to output.rs**
- **Found during:** Task 2 (main.rs references these; commands reference Stats)
- **Issue:** Plan 01 output.rs had `print_tasks`, `print_count`, `info`, `error` but NOT `Stats`, `print_task`, `print_lines`, or `print_stats` — required by the plan's interfaces block
- **Fix:** Added `pub struct Stats` with 5 usize fields + 3 pub methods to output.rs
- **Files modified:** `crates/todotxt-cli/src/output.rs`
- **Committed in:** `35dcb15`

**2. [Rule 3 - Missing dep] Added chrono to todotxt-cli Cargo.toml**
- **Found during:** Task 3 (stats.rs wouldn't compile without it)
- **Issue:** `chrono` was in workspace deps and `todotxt-core` but not declared in the CLI crate
- **Fix:** Added `chrono = { workspace = true }` to `crates/todotxt-cli/Cargo.toml`
- **Files modified:** `crates/todotxt-cli/Cargo.toml`
- **Committed in:** `1d4d6a9`

**3. [Rule 1 - Bug] Added print_count() to list.rs and #[allow(dead_code)] to info/error**
- **Found during:** Task 3 clippy pass (`-D warnings` fails on dead methods)
- **Issue:** `info`, `error`, `print_count` unused → clippy error under `-D warnings`
- **Fix:** Wire `print_count` in list.rs (natural per D-14 footer behavior); suppress `info`/`error` with `#[allow(dead_code)]` since they're reserved for future plans
- **Files modified:** `crates/todotxt-cli/src/output.rs`, `crates/todotxt-cli/src/commands/list.rs`
- **Committed in:** `1d4d6a9`

---

**Total deviations:** 3 auto-fixed
**Impact on plan:** All necessary for compilation and correctness. No scope creep.

## Known Stubs

- `crates/todotxt-cli/src/commands/completions.rs` — intentional stub (Plan 03 implements shell completion generation)

## Threat Flags

None — no new network endpoints, auth paths, or trust boundaries beyond what was in the plan's threat model.

## Self-Check: PASSED

Files verified:
- `crates/todotxt-cli/src/cli.rs` ✓
- `crates/todotxt-cli/src/main.rs` ✓
- `crates/todotxt-cli/src/commands/mod.rs` ✓
- `crates/todotxt-cli/src/commands/list.rs` ✓
- `crates/todotxt-cli/src/commands/stats.rs` ✓
- `crates/todotxt-cli/src/commands/projects.rs` ✓
- `crates/todotxt-cli/src/commands/contexts.rs` ✓
- `crates/todotxt-cli/src/commands/show.rs` ✓

Commits verified: `c90d4d0`, `35dcb15`, `1d4d6a9` — all present in git log.

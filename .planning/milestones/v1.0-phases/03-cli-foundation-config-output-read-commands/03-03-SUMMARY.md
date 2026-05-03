---
phase: 03-cli-foundation-config-output-read-commands
plan: 03
subsystem: cli
tags: [clap_complete, integration-tests, assert_cmd, assert_fs, completions]

requires:
  - phase: 03-cli-foundation-config-output-read-commands
    plan: 02
    provides: "CLI dispatch, list/stats/show/completions commands, Renderer, json_success/json_error"

provides:
  - "Shell completions for bash/zsh/fish/PowerShell via clap_complete::generate"
  - "Integration test helpers: TestFixture with TempDir, todo.txt, config.toml fixture"
  - "list_tests: 5 tests covering success, empty filter (P-10), filter narrowing, JSON envelope, --no-color"
  - "stats_tests: 2 tests covering human output and JSON envelope with total key"
  - "show_tests: 4 tests covering id=1 success, id=9999 exit 1, id=0 exit 1, JSON error envelope"
  - "completions_tests: 2 tests covering bash and zsh produce non-empty output"

affects: []

tech-stack:
  added: []
  patterns:
    - "clap::CommandFactory + clap_complete::generate for shell completion generation"
    - "TestFixture struct with #[allow(dead_code)] to keep TempDir alive across test scope"
    - "config.toml written with {:?} Debug format for Windows path escaping"
    - "mod helpers; in each integration test file (each tests/ file is a separate crate root)"
    - "Raw output inspection via .output().unwrap() for --no-color ANSI check"

key-files:
  created:
    - crates/todotxt-cli/tests/helpers.rs
    - crates/todotxt-cli/tests/list_tests.rs
    - crates/todotxt-cli/tests/stats_tests.rs
    - crates/todotxt-cli/tests/show_tests.rs
    - crates/todotxt-cli/tests/completions_tests.rs
  modified:
    - crates/todotxt-cli/src/commands/completions.rs

key-decisions:
  - "clap::CommandFactory used (not clap_complete::generate with &Cli) — CommandFactory is the trait that provides Cli::command()"
  - "TestFixture.dir and .todo kept as public fields for TempDir lifetime management; suppressed dead_code warning at struct level"
  - "completions_tests.rs uses TestFixture (with config) not bare Command — avoids side-effect of creating real user config"
  - "show 0 correctly exits 1 — show.rs has explicit id==0 guard returning CliError::NotFound"

requirements-completed: [PLAT-01, READ-01, READ-02, READ-05, READ-06, READ-07, READ-08]

duration: 15min
completed: 2026-04-15
---

# Phase 03 Plan 03: Shell Completions + Integration Tests Summary

**clap_complete shell generation for bash/zsh/fish/PowerShell; 13 new integration tests covering list/stats/show/completions end-to-end**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-04-15
- **Completed:** 2026-04-15
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- `completions.rs`: Replaced `todo!()` stub with `Cli::command()` + `clap_complete::generate` — bash/zsh/fish/PowerShell/Elvish all produce output
- `tests/helpers.rs`: `TestFixture` with `TempDir`, pre-populated `todo.txt` (4 tasks), and `config.toml` using `{:?}` Debug format for cross-platform path escaping; `cmd()` returns a `Command` pre-configured with `--config`
- `tests/list_tests.rs`: 5 tests — list success, empty filter exits 0 (P-10), positional filter narrows, `--json` envelope, `--no-color` has no ANSI
- `tests/stats_tests.rs`: 2 tests — human output with "Total:/Complete:/Incomplete:", JSON with `"total"` key
- `tests/show_tests.rs`: 4 tests — show id=1 success, id=9999 exit 1, id=0 exit 1, `--json` error envelope with `"error"` key
- `tests/completions_tests.rs`: 2 tests — bash and zsh produce non-empty output

## Test Results

All 13 new integration tests pass. Full workspace: 120 tests, 0 failures.

```
list_tests:         5 passed
stats_tests:        2 passed
show_tests:         4 passed
completions_tests:  2 passed
config_tests:       1 passed (existing)
```

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check

- [x] `crates/todotxt-cli/src/commands/completions.rs` contains `clap_complete::generate` and `Cli::command()`
- [x] `cargo build -p todotxt-cli` exits 0
- [x] `todotxt completions bash` produces non-empty bash completion output
- [x] All 13 new integration tests pass
- [x] Full workspace test suite: 0 failures
- [x] Commits: afbceec (completions.rs), 100609f (integration tests)

## Self-Check: PASSED

---
phase: 03-cli-foundation-config-output-read-commands
plan: 05
subsystem: cli
tags: [default-filtering, DONE-term, list-semantics, regression-tests, unknown-preset, gap-closure]

# Dependency graph
requires:
  - phase: 03-cli-foundation-config-output-read-commands
    provides: "CLI list command implementation and JSON/human output rendering from waves 1-4"
provides:
  - "Default incomplete-only list filter (automatically excludes completed tasks unless completion is explicitly requested)"
  - "Preserved unknown-preset warning behavior with corrected output formatting"
  - "5 new integration regression tests ensuring default filtering, warning behavior, and output correctness"
affects:
  - "phase-04-cli-write-commands"
  - "phase-05-task-enrichment-bulk-operations"
  - "cli-filter-composition"
  - "preset-resolution-paths"

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Filter composition: implicit -DONE appended when no explicit completion term (DONE or -DONE) present in query"
    - "Completion term detection: scans effective query post-preset-merge to determine explicit user intent"
    - "Unknown preset behavior: warning emitted to stderr while preserving output formatting correctness"

key-files:
  created: []
  modified:
    - "crates/todotxt-cli/src/commands/list.rs"
    - "crates/todotxt-cli/tests/list_tests.rs"

key-decisions:
  - "Default -DONE appended at filter composition boundary (build_filter), not at query parsing, preserving D-08/D-09/D-10/D-11 token merge rules"
  - "Explicit completion detection checks effective query after preset merge, ensuring user intent is honored whether via positional arg, --filter flag, or preset"
  - "Unknown preset warning path preserved exactly (stderr message + return 0), only filter behavior changed"
  - "Regression test design: integration-level assertions on stdout/stderr behavior ensures end-to-end CLI correctness"

patterns-established:
  - "Implicit filter injection at composition boundary: build_filter responsible for applying platform/user-preference defaults"
  - "Completion term override semantics: explicit DONE/-DONE in any query form (positional, --filter, preset) takes precedence over default"
  - "CLI output regression coverage: human/--json/--no-color modes tested together to prevent cross-format bugs"

requirements-completed: [READ-01, READ-06, READ-07, CFG-02, READ-08]

# Metrics
duration: 12min
completed: 2026-04-15
---

# Phase 03: CLI Foundation Summary (Plan 05)

**Enforce default incomplete-only list filter and add CLI regression coverage for default exclusion, unknown-preset warning, and output correctness**

## Performance

- **Duration:** 12 min
- **Started:** 2026-04-15T20:30:00Z
- **Completed:** 2026-04-15T20:42:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- `build_filter` in list.rs now appends `-DONE` by default when no explicit completion term (`DONE` or `-DONE`) is detected in the effective query
- Preset merge and token composition logic unchanged — all decision constraints (D-08/D-09/D-10/D-11) preserved
- Unknown preset warning continues to emit to stderr and exit with code 0, while output formatting remains correct
- 5 new integration regression tests added to prevent regressions in default filtering, warning paths, and output modes (human/--json/--no-color)

## Task Commits

1. **Task 1: Enforce default incomplete-only list semantics while honoring explicit completion filters** - `1238163` (fix)
2. **Task 2: Add integration regressions for default exclusion, warning preservation, and clean output/JSON** - `1238163` (fix, combined)

**Plan metadata:** `1238163` (fix: cli default list excludes completed tasks; add gap-closure regressions)

## Files Created/Modified

- `crates/todotxt-cli/src/commands/list.rs` — `build_filter` function now:
  - Detects explicit completion terms (DONE or -DONE) in effective query after preset merge
  - Appends `-DONE` by default when no completion term found
  - Preserves all token composition rules and unknown preset warning behavior
- `crates/todotxt-cli/tests/list_tests.rs` — 5 new integration regression tests:
  - `list_default_excludes_completed_tasks` — verifies default list hides completed rows
  - `list_done_token_shows_completed_tasks` — confirms explicit DONE override works
  - `list_unknown_preset_warns_on_stderr_exits_zero` — preserves warning path + exit code 0
  - `list_json_no_cr_in_output` — ensures JSON raw fields contain no trailing `\r`
  - `list_no_color_no_cr_artifacts` — confirms --no-color output free of ANSI and CR corruption

## Decisions Made

- Default filter behavior injected at composition boundary, not at query parsing stage, to preserve preset/token merge semantics
- Completion detection checks effective query post-preset resolution, ensuring explicit user intent honored across all input forms
- Unknown preset warning path unchanged to maintain operator diagnosability per threat model T-03-05-02

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## Test Results

```
cargo test -p todotxt-cli:
- Total tests passed: 27/27 ✓
- List tests: 10/10 (5 existing + 5 new)
- Show tests: 4/4
- Stats tests: 2/2
- Projects/Contexts tests: 2/2
- Completions tests: 1/1
- Library/Config/Output tests: 8/8
- Execution time: 0.15s
- Status: ✓ ALL PASSED

cargo clippy -p todotxt-cli -- -D warnings:
- No warnings or errors detected
- Status: ✓ PASSED
```

## User Setup Required

None - no external service configuration required.

## Gap Closure Summary

This plan closes the following UAT gaps from phase-03-uat:

1. **UAT Issue 2** — Default list now excludes completed tasks (test: `list_default_excludes_completed_tasks`)
2. **UAT Issue 10** — Unknown preset `:nonexistent` preserves warning behavior while producing clean output (test: `list_unknown_preset_warns_on_stderr_exits_zero`)
3. **UAT Issue 12** — List output modes (human, --json, --no-color) remain free of control character corruption (tests: `list_json_no_cr_in_output`, `list_no_color_no_cr_artifacts`)

## Next Phase Readiness

- CLI list command now exhibits correct default behavior: incomplete-only filtering with explicit override support
- All 27 list/show/stats/projects/contexts/completions tests passing
- Phase 3 core requirements READ-01/READ-06/READ-07/CFG-02/READ-08 verified complete
- Foundation ready for Phase 04: CLI Write Commands (add, complete, undo, delete, edit, append, prepend)

---
*Phase: 03-cli-foundation-config-output-read-commands (Wave 5 - Gap Closure)*
*Completed: 2026-04-15*
*Commit: 1238163*

---
status: resolved
trigger: "Diagnose this UAT issue and return root cause only (no code changes). Phase 03 list command shows done task printed and mangled indentation."
created: 2026-04-15T00:00:00Z
updated: 2026-04-23T00:00:00Z
---

## Current Focus

hypothesis: `list` includes completed tasks by default, and CR characters preserved in task raw text cause terminal carriage-return mangling.
test: Verify filter construction in CLI list path and verify CR preservation from parse/load into rendered output.
expecting: `Filter::new()` in list path with no `-DONE` term; completed task JSON raw contains `\r`; table/no-color output shows overwritten/indented completed row.
next_action: Return root-cause-only JSON with artifacts and evidence.

## Symptoms

expected: Running todotxt list prints only incomplete tasks; completed tasks are hidden and output rows are aligned.
actual: User reported done task is printed, improperly indented/mangled.
errors: none reported
reproduction: .\\target\\debug\\todotxt.exe list
started: unknown

## Eliminated

## Evidence

- timestamp: 2026-04-15T00:05:00Z
	checked: .planning/phases/03-cli-foundation-config-output-read-commands/03-UAT.md
	found: UAT gaps for tests 2/9/10/12 all cite done task showing and mangled/CR output in list paths.
	implication: Symptoms are consistent across normal, --json, and --no-color list output.

- timestamp: 2026-04-15T00:10:00Z
	checked: crates/todotxt-cli/src/commands/list.rs
	found: `build_filter` returns `Filter::new()` when no filter tokens are supplied and comment says "no filter = show all".
	implication: `todotxt list` intentionally includes completed tasks unless caller supplies `-DONE`.

- timestamp: 2026-04-15T00:12:00Z
	checked: crates/todotxt-core/src/filter.rs
	found: `Filter::default/new` sets hidden/future-threshold suppression only; no default `NotDone` term.
	implication: Completed tasks pass default filter and appear in list output.

- timestamp: 2026-04-15T00:15:00Z
	checked: crates/todotxt-core/src/task.rs and crates/todotxt-core/src/task_list.rs
	found: `Task::parse` stores `raw = line.to_string()` before trimming trailing `\r`; parsing logic trims for structured fields only.
	implication: CRLF content can keep `\r` in `Task.raw`, which later leaks into rendered/JSON output.

- timestamp: 2026-04-15T00:18:00Z
	checked: reproduction command outputs (`todotxt list`, `--no-color list`, `--json list`)
	found: list shows 4 tasks including completed entry; no-color output shows mangled line (`024-01-01...`); JSON includes `"completed":true` and `"raw":"x 2024-01-01 Done task +work\r"`.
	implication: Both user-visible symptoms are reproduced and trace back to default filtering plus CR-preserved raw serialization.

## Resolution

root_cause: `todotxt list` uses an empty default filter (`Filter::new`) that does not exclude completed tasks, and completed task lines loaded with trailing CR preserve `\r` in `Task.raw` (because raw is captured before trim). Rendering/JSON uses `Task.raw`, so the completed row appears and the embedded carriage return mangles indentation/text.
fix: not applied (diagnose-only mode)
verification: Reproduced via `todotxt list`, `todotxt --no-color list`, and `todotxt --json list` showing completed task plus `\r` in raw field.
files_changed: []

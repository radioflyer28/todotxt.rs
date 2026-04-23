---
status: resolved
trigger: "Issue: `todotxt --json list` should return clean payloads without trailing CR and exclude completed tasks, but completed task `raw` includes trailing \\r"
created: 2026-04-15T19:44:42.5964556-04:00
updated: 2026-04-23T00:00:00Z
---

## Current Focus

hypothesis: Mixed line endings plus non-normalized `Task.raw` and no default `-DONE` filter cause both leaked `\r` in JSON `raw` and completed tasks appearing in `list`.
test: Correlate runtime output with parser/filter implementation and actual todo file bytes.
expecting: `--json list` includes completed entries; completed entry raw includes `\r`; file has LF first newline and CRLF on completed line; parser preserves raw before trimming CR.
next_action: Return diagnose-only root cause with evidence and affected artifacts.

## Symptoms

expected: Running `todotxt --json list` returns clean payloads without trailing CR and excludes completed tasks.
actual: JSON contains trailing `\\r` in completed task `raw` field.
errors: None reported.
reproduction: `.\\target\\debug\\todotxt.exe --json list`
started: Reported during Phase 03 UAT.

## Eliminated

## Evidence

- timestamp: 2026-04-15T19:46:00-04:00
	checked: crates/todotxt-cli/src/commands/list.rs
	found: list command builds `Filter::new()` for empty query and immediately calls `list.filter(&filter)`.
	implication: Default list path does not explicitly exclude completed tasks.

- timestamp: 2026-04-15T19:46:20-04:00
	checked: crates/todotxt-core/src/filter.rs
	found: `Filter::default()` only suppresses hidden/future-threshold; completion is filtered only when term `-DONE` exists.
	implication: Completed tasks are included by default in `list` and `--json list`.

- timestamp: 2026-04-15T19:47:10-04:00
	checked: runtime reproduction `.\\target\\debug\\todotxt.exe --json list`
	found: JSON contains 4 tasks, including one with `completed=true`; escaped raw is `x\ 2024-01-01\ Done\ task\ \+work\r`.
	implication: Reported UAT behavior is reproducible and not a parsing artifact in the test harness.

- timestamp: 2026-04-15T19:49:30-04:00
	checked: active todo file content at `C:\\Users\\akriz\\todo.txt`
	found: File has mixed line endings: first lines LF, completed line ends CRLF (`...Done task +work\r\n`).
	implication: A split-on-LF path will leave a trailing `\r` on that completed line.

- timestamp: 2026-04-15T19:49:50-04:00
	checked: crates/todotxt-core/src/task.rs and crates/todotxt-core/src/task_list.rs
	found: `Task::parse` sets `raw = line.to_string()` before trimming trailing `\r`; `task_list::detect_line_ending` picks one global separator from first newline.
	implication: With mixed line endings, CR can remain in `raw` for specific rows and then be emitted unchanged by JSON serializer.

## Resolution

root_cause:
Single design gap spanning parse+list defaults: `list` uses a default filter that does not exclude completed tasks, and task parsing preserves `raw` before CR normalization while line splitting assumes one global line-ending style. With a mixed-LF/CRLF todo file, the completed row ending in CRLF is split on LF and retains trailing `\r` in `raw`, which `--json list` serializes verbatim.
fix:
verification:
files_changed: []

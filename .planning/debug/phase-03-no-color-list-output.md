---
status: diagnosed
trigger: "Diagnose this UAT issue and return root cause only (no code changes). Phase 03 --no-color list shows done task + mangled year + broken indentation"
created: 2026-04-15T00:00:00Z
updated: 2026-04-15T00:45:00Z
---

## Current Focus
<!-- OVERWRITE on each update - reflects NOW -->

hypothesis: two independent defects produce the UAT symptom: list default filter includes completed tasks, and mixed line endings leak trailing CR into Task.raw which is rendered directly
test: correlate command behavior with list/filter implementation and raw todo.txt bytes
expecting: default list path has no NotDone guard; raw done line contains trailing \r and renderer uses task.to_raw()
next_action: return root-cause-only diagnosis JSON for test 10

## Symptoms
<!-- Written during gathering, then IMMUTABLE -->

expected: Running todotxt --no-color list prints clean rows, hides completed tasks, no mangled text
actual: done task shown; year appears mangled and row indentation broken
errors: none reported
reproduction: .\\target\\debug\\todotxt.exe --no-color list
started: reported during Phase 03 UAT

## Eliminated
<!-- APPEND only - prevents re-investigating -->

## Evidence
<!-- APPEND only - facts discovered -->

- timestamp: 2026-04-15T00:18:00Z
	checked: .planning/phases/03-cli-foundation-config-output-read-commands/03-UAT.md
	found: Test 10 reports "done task is improperly indented and mangled the year" under --no-color list
	implication: issue scope includes both completion filtering and text rendering artifacts

- timestamp: 2026-04-15T00:22:00Z
	checked: crates/todotxt-cli/src/commands/list.rs
	found: list command builds Filter::new() when no query is supplied and passes it directly to TaskList::filter
	implication: default list path does not enforce -DONE / incomplete-only behavior

- timestamp: 2026-04-15T00:24:00Z
	checked: crates/todotxt-core/src/filter.rs
	found: Filter::new defaults only suppress_hidden and suppress_future_threshold; there is no implicit NotDone term
	implication: completed tasks match default filter and appear in list output unless user supplies -DONE

- timestamp: 2026-04-15T00:30:00Z
	checked: runtime command .\\target\\debug\\todotxt.exe --json list
	found: JSON output includes raw field ending with escaped carriage return: "raw":"x 2024-01-01 Done task +work\\r"
	implication: source task raw text contains trailing CR that survives parse and is serialized/rendered

- timestamp: 2026-04-15T00:33:00Z
	checked: crates/todotxt-core/src/task.rs
	found: Task::parse stores raw = line.to_string() before trimming trailing '\\r' for parsed-field processing
	implication: Task.to_raw() retains CR when input line ends with CR

- timestamp: 2026-04-15T00:35:00Z
	checked: crates/todotxt-cli/src/output.rs
	found: table and JSON both emit task.to_raw() directly (Cell::new(task.to_raw().to_string()) and TaskDto.raw = task.to_raw())
	implication: retained CR propagates to output, causing carriage-return rendering artifacts (mangled/indented row text)

- timestamp: 2026-04-15T00:41:00Z
	checked: C:\Users\akriz\todo.txt raw bytes
	found: file has mixed line endings (HAS_CRLF=True, HAS_LONE_LF=True) and done line contains trailing \r
	implication: mixed-LF/CRLF input is the trigger for CR leakage into raw output on affected line

## Resolution
<!-- OVERWRITE as understanding evolves -->

root_cause: "Test 10 fails due to two defects in the list pipeline: (1) default list filtering does not exclude completed tasks (no implicit -DONE), so done entries are shown; (2) Task::parse preserves trailing CR in Task.raw for mixed-ending lines, and renderer prints/serializes to_raw() verbatim, producing carriage-return artifacts that mangle year/indentation."
fix: "not applied (diagnose-only mode)"
verification: "root cause validated by code-path inspection and runtime evidence from --json list plus raw todo.txt line-ending scan"
files_changed: []

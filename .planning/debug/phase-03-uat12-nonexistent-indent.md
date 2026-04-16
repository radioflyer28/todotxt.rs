---
status: diagnosed
trigger: "Phase 03 UAT test 12: list :nonexistent warns correctly, but done task still improperly indented in output"
created: 2026-04-15T00:00:00Z
updated: 2026-04-15T00:25:00Z
---

## Current Focus

hypothesis: mixed line endings in todo.txt cause `\r` to remain in Task.raw for CRLF lines, and rendering `to_raw()` emits carriage returns that corrupt table alignment
test: verify `\r` in runtime JSON raw output, inspect todo file EOL mix, and trace parsing/splitting logic in task_list/task/output path
expecting: completed task row carries trailing CR character and is printed verbatim in table/json
next_action: return structured root-cause report (no code changes)

## Symptoms

expected: Running todotxt list :nonexistent warns about unknown preset while preserving clean list output
actual: warning path works but done task still improperly indented in output
errors: none (warning expected)
reproduction: .\\target\\debug\\todotxt.exe list :nonexistent
started: reported during Phase 03 UAT

## Eliminated

## Evidence

- timestamp: 2026-04-15T00:10:00Z
	checked: CLI runtime output for `todotxt list :nonexistent`, `--no-color`, and `--json`
	found: unknown preset warning is emitted as expected; JSON payload contains `"raw":"x 2024-01-01 Done task +work\r"`; parsed JSON raw string ends with char code 13
	implication: done task raw text includes a carriage return character before rendering

- timestamp: 2026-04-15T00:18:00Z
	checked: active config and todo file bytes (`C:\Users\akriz\AppData\Roaming\todotxt\config\config.toml` -> `C:\Users\akriz\todo.txt`)
	found: todo file has mixed endings (`CRLF_COUNT=1`, `LF_ONLY_COUNT=3`); completed task line is CRLF (`... 2B 77 6F 72 6B 0D 0A`)
	implication: only some lines carry CRLF, matching symptom that one row is malformed

- timestamp: 2026-04-15T00:23:00Z
	checked: `crates/todotxt-core/src/task_list.rs`, `crates/todotxt-core/src/task.rs`, `crates/todotxt-cli/src/output.rs`
	found: `detect_line_ending()` chooses first newline style globally; with first newline LF, `split_lines(..., "\n")` leaves trailing `\r` on CRLF lines; `Task::parse()` strips `\r` only for parsed fields but stores `raw` before trim; renderer prints `task.to_raw()` in table/json
	implication: CR survives in `Task.raw` and is emitted directly, causing carriage-return display corruption (indentation/mangled row)

## Resolution

root_cause: "Mixed LF/CRLF input combined with global first-newline EOL detection leaves trailing `\r` on CRLF lines; because `Task.raw` is captured before CR trimming and renderer outputs `to_raw()` verbatim, the completed row includes a carriage return that breaks terminal table alignment."
fix: ""
verification: ""
files_changed: []

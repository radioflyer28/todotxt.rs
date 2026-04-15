---
status: complete
phase: 03-cli-foundation-config-output-read-commands
source: [03-01-SUMMARY.md, 03-02-SUMMARY.md, 03-03-SUMMARY.md]
started: 2026-04-15T00:00:00Z
updated: 2026-04-15T01:10:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Config auto-creates on first run
expected: With no existing config file, running todotxt for the first time automatically creates the config file and parent directories at the platform default path. The command runs without error and the config file exists afterward.
result: pass
note: Config created at %APPDATA%\todotxt\config\config.toml (directories crate adds config\ subdirectory on Windows — code comment says %APPDATA%\todotxt\config.toml, minor doc gap)

### 2. list shows all tasks
expected: Running `todotxt list` prints all incomplete tasks from the configured todo.txt file, one per line, with priority tasks shown with color (A=red, B=yellow, C=green), plus a task count footer (e.g. "3 of 5 tasks shown").
result: issue
reported: "it incorrectlyt indented the done task"
severity: major

### 3. list with filter narrows results
expected: Running `todotxt list groceries` prints only tasks containing "groceries". Tasks not matching the filter are hidden. The count footer reflects the filtered subset.
result: pass

### 4. stats shows task counts
expected: Running `todotxt stats` prints a summary with "Total:", "Complete:", and "Incomplete:" counts. The numbers add up correctly.
result: pass

### 5. projects lists all project tags
expected: Running `todotxt projects` prints all unique +project tags found in the todo.txt file, one per line, sorted alphabetically. Completed tasks' projects are excluded.
result: pass

### 6. contexts lists all context tags
expected: Running `todotxt contexts` prints all unique @context tags found in the todo.txt file, one per line, sorted alphabetically. Completed tasks' contexts are excluded.
result: pass

### 7. show displays a specific task
expected: Running `todotxt show 1` prints the raw content of the first task in todo.txt and exits with code 0.
result: pass

### 8. show with invalid ID exits with code 1
expected: Running `todotxt show 9999` (or any ID beyond the list length) prints a "not found" message and exits with code 1 (not 0, not 2).
result: pass

### 9. --json flag outputs JSON envelope
expected: Running `todotxt --json list` outputs a JSON object with `"schema_version": 1` and a `"data"` key containing the task array. No ANSI color codes appear in the output.
result: issue
reported: "I do see a special char \"\\r\" in there"
severity: major

### 10. --no-color suppresses ANSI color codes
expected: Running `todotxt --no-color list` with priority tasks in the list produces output with no ANSI escape sequences (no \x1b[ color codes). Tasks still appear, just without color.
result: issue
reported: "done task is improperly indented and mangled the year"
severity: major

### 11. completions generates shell script
expected: Running `todotxt completions bash` prints a non-empty bash completion script (starts with `_todotxt` function or similar) and exits with code 0. Running with `zsh` produces a non-empty zsh script.
result: pass

### 12. Unknown preset warns to stderr
expected: Running `todotxt list :nonexistent` (where `:nonexistent` is not defined in config presets) prints a warning to stderr like `warning: unknown preset ':nonexistent' — ignored` and still returns the full unfiltered task list (exit 0).
result: issue
reported: "yes, but the done task is still improperly indented"
severity: major

## Summary

total: 12
passed: 8
issues: 4
pending: 0
skipped: 0

## Gaps

- truth: "Running `todotxt list` prints only incomplete tasks; completed tasks are hidden and output rows are aligned"
  status: failed
  reason: "User reported: it incorrectlyt indented the done task"
  severity: major
  test: 2
  artifacts: []
  missing: []
- truth: "Running `todotxt --json list` returns clean task payloads without trailing CR characters and excludes completed tasks from list output"
  status: failed
  reason: "User reported: JSON includes trailing \"\\r\" in raw text for completed task entry"
  severity: major
  test: 9
  artifacts: []
  missing: []
- truth: "Running `todotxt --no-color list` prints cleanly formatted task rows; completed tasks are hidden and text is not mangled"
  status: failed
  reason: "User reported: done task is improperly indented and mangled the year"
  severity: major
  test: 10
  artifacts: []
  missing: []
- truth: "Running `todotxt list :nonexistent` warns about unknown preset while preserving clean, correctly formatted list output"
  status: failed
  reason: "User reported: done task is still improperly indented"
  severity: major
  test: 12
  artifacts: []
  missing: []

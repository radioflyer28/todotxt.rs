---
phase: 35-basic-clipboard-workflows
reviewed: 2026-04-30T17:44:41.5705319Z
depth: standard
files_reviewed: 2
files_reviewed_list:
  - crates/todotxt-tui/Cargo.toml
  - crates/todotxt-tui/src/app.rs
findings:
  critical: 0
  warning: 3
  info: 1
  total: 4
status: issues_found
---

# Phase 35: Code Review Report

**Reviewed:** 2026-04-30T17:44:41.5705319Z  
**Depth:** standard  
**Files Reviewed:** 2  
**Status:** issues_found

## Summary

Reviewed the phase-35 clipboard integration in `todotxt-tui` (`arboard` dependency, normal-mode copy/paste, and editor Ctrl+V interception).

No crash-level defects were found in lazy clipboard initialization itself, and key routing for normal-mode `y`/`p` and editor-mode Ctrl+V is wired correctly.

The main issues are around input trust boundaries and behavior consistency: pasted clipboard lines are accepted verbatim, multi-select copy order is reversed relative to task order, and multi-line paste can partially apply on write failure.

## Warnings

### WR-01: Clipboard Content Is Accepted Without Control-Character Sanitization

**File:** `crates/todotxt-tui/src/app.rs:1339`  
**Issue:** `paste_from_clipboard()` takes untrusted clipboard text, keeps each non-empty line verbatim, and parses/adds it directly (`Task::parse(&line)`) with no control-character filtering. This can inject non-printable/control bytes into persisted tasks and later rendering paths.

**Fix:** Sanitize each clipboard line before parsing. At minimum, reject or strip C0/C1 control characters except tab/space, then parse the sanitized string.

```rust
let sanitized_lines: Vec<String> = clipboard_text
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .map(|line| line.chars()
        .filter(|c| !c.is_control() || *c == '\t')
        .collect::<String>())
    .filter(|line| !line.is_empty())
    .collect();
```

### WR-02: Multi-Select Copy Order Is Reversed

**File:** `crates/todotxt-tui/src/app.rs:1268`  
**Issue:** `copy_selected_to_clipboard()` sorts selected task indices in descending canonical order before joining lines. Copying tasks then pasting them creates tasks in reverse order of the source list, which is surprising and can reorder user data.

**Fix:** Sort in ascending canonical order for clipboard output, or explicitly preserve current visible row order.

```rust
targets.sort_unstable(); // ascending for natural top-to-bottom copy order
```

### WR-03: Paste Can Partially Apply On Mid-Loop Write Failure

**File:** `crates/todotxt-tui/src/app.rs:1353`  
**Issue:** `paste_from_clipboard()` adds each line with `task_list.add(task)` inside a loop. If an I/O error occurs after some successful adds, the method returns an error with a partially-applied paste operation and no rollback.

**Fix:** Build all parsed tasks first and perform a single atomic save/update path (batch append + one save), or capture pre-state and restore on failure.

```rust
let parsed: Vec<Task> = sanitized_lines.iter().map(|l| Task::parse(l)).collect();
self.task_list.extend_and_save(parsed)?; // single atomic write path
```

## Info

### IN-01: Clipboard Error Messages Conflate Distinct Failure Modes

**File:** `crates/todotxt-tui/src/app.rs:1318`  
**Issue:** Initialization failure (`Clipboard::new()`), read failure (`get_text()`), and truly empty clipboard content are all surfaced as `"clipboard is empty"`. This makes diagnosis difficult and hides environment/backend issues.

**Fix:** Differentiate messages, e.g. `"clipboard unavailable"`, `"clipboard read failed"`, and `"clipboard is empty"`.

---

_Reviewed: 2026-04-30T17:44:41.5705319Z_  
_Reviewer: the agent (gsd-code-reviewer)_  
_Depth: standard_

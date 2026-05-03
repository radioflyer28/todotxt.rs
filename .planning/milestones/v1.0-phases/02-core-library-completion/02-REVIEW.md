---
phase: 02-core-library-completion
status: clean
depth: standard
reviewed-files:
  - crates/todotxt-core/src/filter.rs
  - crates/todotxt-core/src/sort.rs
  - crates/todotxt-core/src/portable.rs
  - crates/todotxt-core/src/task_list.rs
  - crates/todotxt-core/src/watcher.rs
  - crates/todotxt-core/src/error.rs
  - crates/todotxt-core/src/lib.rs
reviewed: 2026-04-15T00:00:00Z
findings:
  critical: 0
  high: 0
  medium: 1
  low: 1
  info: 4
  total: 6
---

# Phase 02: Code Review Report

**Reviewed:** 2026-04-15  
**Depth:** standard  
**Files Reviewed:** 7  
**Status:** clean (no CRITICAL or HIGH findings)

---

## Summary

Seven source files were reviewed covering the core library completion phase: filtering (12-variant enum), sorting (5 variants), portable path resolution, task list mutation/persistence, file watching, error types, and public API exports.

The overall code quality is high. Atomic saves are implemented correctly (`NamedTempFile::persist`), the sort is stable, error variants use `thiserror` properly, and the watcher pattern (watch parent directory, filter by filename) is idiomatic for atomic-rename reliability.

One medium-severity logic bug was found in the hidden-tag suppression check that produces false positives on substring matches. One low-severity issue exists in `batch_update` around undocumented duplicate-index behavior. Four informational items round out the findings.

No security vulnerabilities, panics, or thread-safety issues were found.

---

## Medium Issues

### MD-01: `suppress_hidden` Uses Substring Match — False Positives on `h:1X` and `Xh:1` Tokens

**File:** `crates/todotxt-core/src/filter.rs:104`  
**Issue:** The hidden-tag check uses `raw.contains("h:1")`, which is a plain substring search over the entire raw task line. This produces false positives whenever "h:1" appears embedded in another token. Real examples:

- A task with extension key `h:10` (e.g. `call dentist h:10`) is incorrectly suppressed — `"h:10".contains("h:1")` is `true`.
- A task with extension key `auth:1` is incorrectly suppressed — `"auth:1"` contains the substring `"h:1"`.
- Any task whose description text happens to contain the characters `h:1` (e.g., `fix h:1 issue in auth`) is silently hidden.

In the todo.txt format the `h:1` hidden marker is always a standalone whitespace-delimited token, so the check must operate on tokens, not the full raw string.

**Fix:**
```rust
// Before
if self.suppress_hidden && raw.contains("h:1") {
    return false;
}

// After
if self.suppress_hidden && raw.split_ascii_whitespace().any(|t| t == "h:1") {
    return false;
}
```

---

## Low Issues

### LO-01: `batch_update` Last-Write-Wins on Duplicate Indices Is Undocumented

**File:** `crates/todotxt-core/src/task_list.rs` (~line 257)  
**Issue:** `batch_update` validates all indices before mutating, then applies replacements in iteration order. When the caller provides the same index twice — e.g. `[(3, task_a), (3, task_b)]` — both indices pass the bounds check and the second write silently overwrites the first. The resulting task at index 3 is `task_b`; `task_a` is discarded. This behavior is not documented and may surprise callers.

**Fix:** Either document the behavior explicitly, or detect and reject duplicate indices:
```rust
// Option A — document in the docstring:
/// If `replacements` contains duplicate indices, the last entry for each
/// index wins (replacements are applied in order).

// Option B — fail on duplicates:
use std::collections::HashSet;
let mut seen = HashSet::new();
for &(index, _) in &replacements {
    if index >= count {
        return Err(TodoError::IndexOutOfBounds { index, count });
    }
    if !seen.insert(index) {
        return Err(TodoError::IndexOutOfBounds { index, count: 0 }); // or a dedicated error variant
    }
}
```

---

## Info Items

### IN-01: Bare `-` Query Token Creates `Exclude("")` — Matches Nothing

**File:** `crates/todotxt-core/src/filter.rs:82-83`  
**Issue:** A token consisting of only `-` passes through the DONE and `due:*` guards, then hits `token.strip_prefix('-')`, yielding `Some("")`. This creates `FilterTerm::Exclude("")`. At match time, `raw.contains("")` is always `true` in Rust, so `!raw.contains("") == false` — every task fails the filter. The user gets zero results with no error.

While a bare `-` in a filter query is almost certainly a user mistake, the silent "no results" outcome is confusing. A small guard before the `strip_prefix` branch would give cleaner behavior:

**Fix:**
```rust
if let Some(rest) = token.strip_prefix('-') {
    if rest.is_empty() {
        // Bare `-` — treat as a no-op include or return an error upstream
        return FilterTerm::Include("-".to_string()); // preserves the literal
    }
    return FilterTerm::Exclude(rest.to_string());
}
```

---

### IN-02: `sort()` Does Not Save — API Asymmetry with Other Mutating Methods

**File:** `crates/todotxt-core/src/task_list.rs` (~line 241)  
**Issue:** All other mutation methods (`add`, `update`, `delete`, `batch_update`) call `save()` automatically. `sort()` does not — it is documented "Does NOT save to disk — call `save()` explicitly if persistence is needed." This asymmetry is a footgun: callers who apply a sorted view and don't realize they need a manual `save()` call will silently lose the sort on next load.

No code change required if the design is intentional (sort is view-only), but consider either (a) adding a `sort_and_save()` helper, or (b) renaming to `sort_in_place()` to signal that this is purely an in-memory operation.

---

### IN-03: TOCTOU Race in `resolve_config_path`

**File:** `crates/todotxt-core/src/portable.rs:10`  
**Issue:** `resolve_config_path` checks `binary_dir.join("config.toml").exists()` and then returns the path. Between the existence check and the caller actually reading the file, another process could create or delete it. For a single-user desktop application this is a theoretical concern (OWASP A05 / CWE-367), but it's worth noting for correctness.

No fix required for the current desktop use case. Document the assumption if this function is ever used in a server or multi-user context.

---

### IN-04: Watcher Internal Errors Silently Discarded

**File:** `crates/todotxt-core/src/watcher.rs:55`  
**Issue:** The debouncer callback discards `Err` results from `res` with a comment "Ignore watcher errors — they are transient." Transient errors (e.g., brief kernel event queue overflow) are reasonable to ignore, but persistent errors (e.g., permission denied on the watched directory after an OS policy change) will silently stop change detection. The application will continue running without noticing that it is no longer receiving file-change notifications.

Consider logging or surfacing persistent errors through a separate error channel:
```rust
move |res: DebounceEventResult| {
    match res {
        Ok(events) => {
            if events.iter().any(|e| e.path.file_name() == Some(target_name.as_os_str())) {
                cb();
            }
        }
        Err(_e) => {
            // At minimum: eprintln!("watcher error: {_e}") during development
        }
    }
}
```

---

## Files With No Issues

| File | Assessment |
|------|------------|
| `crates/todotxt-core/src/sort.rs` | Clean. Stable sort, correct None-last ordering for all 5 variants. |
| `crates/todotxt-core/src/error.rs` | Clean. `#[cfg(feature = "watching")]` gate applied correctly. `#[from]` on `Watch` variant is idiomatic. |
| `crates/todotxt-core/src/lib.rs` | Clean. Conditional exports mirror feature gates. Public re-exports are complete. |

---

_Reviewed: 2026-04-15_  
_Reviewer: GitHub Copilot (Claude Sonnet 4.6)_  
_Depth: standard_

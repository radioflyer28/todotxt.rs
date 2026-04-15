---
phase: 03-cli-foundation-config-output-read-commands
reviewed: 2026-04-15T00:00:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - Cargo.toml
  - crates/todotxt-cli/Cargo.toml
  - crates/todotxt-cli/src/cli.rs
  - crates/todotxt-cli/src/commands/completions.rs
  - crates/todotxt-cli/src/commands/contexts.rs
  - crates/todotxt-cli/src/commands/list.rs
  - crates/todotxt-cli/src/commands/mod.rs
  - crates/todotxt-cli/src/commands/projects.rs
  - crates/todotxt-cli/src/commands/show.rs
  - crates/todotxt-cli/src/commands/stats.rs
  - crates/todotxt-cli/src/config.rs
  - crates/todotxt-cli/src/main.rs
  - crates/todotxt-cli/src/output.rs
  - crates/todotxt-cli/tests/completions_tests.rs
  - crates/todotxt-cli/tests/config_tests.rs
  - crates/todotxt-cli/tests/helpers.rs
  - crates/todotxt-cli/tests/list_tests.rs
  - crates/todotxt-cli/tests/show_tests.rs
  - crates/todotxt-cli/tests/stats_tests.rs
findings:
  critical: 0
  warning: 1
  info: 4
  total: 5
status: issues_found
---

# Phase 03: Code Review Report

**Reviewed:** 2026-04-15  
**Depth:** standard  
**Files Reviewed:** 19  
**Status:** issues_found

## Summary

Overall this is a clean, well-structured CLI implementation. The command dispatch, config loading, output rendering, and test fixtures are all solid. No security vulnerabilities, data loss risks, or crashes were found. One logic-level warning was identified: a silent no-op when a `:preset` filter token is unknown, which can yield unexpected results with no user feedback. Four info-level items cover dead code, a fragile test-helper pattern, a stale comment, and a minor inefficiency.

---

## Warnings

### WR-01: Silent unknown `:preset` tokens produce no feedback

**File:** `crates/todotxt-cli/src/commands/list.rs:20`  
**Issue:** When a `:preset_name` token is not found in the config, it is silently dropped from the filter query with no diagnostic. If `:preset_name` is the sole filter token, the combined query becomes empty and all tasks are shown — the opposite of the user's intent. A typo in a preset name (e.g., `:wrk` instead of `:work`) will silently return the full unfiltered list.

**Fix:** Emit a warning to stderr when a preset token does not resolve, so users can detect typos:
```rust
if let Some(preset_name) = token.strip_prefix(':') {
    if let Some(preset) = cfg.presets.get(preset_name) {
        if let Some(q) = &preset.filter {
            query_parts.push(q.clone());
        }
    } else {
        eprintln!("warning: unknown preset ':{preset_name}' — ignored");
    }
}
```

---

## Info

### IN-01: `resolve_path` fallback may use a file path as a directory

**File:** `crates/todotxt-cli/src/config.rs:42`  
**Issue:** `platform_path.parent().unwrap_or(platform_path)` falls back to the file path itself (not a directory) when `parent()` returns `None`. If `platform_path` were a bare filename with no parent component, `resolve_config_path` would receive a file path as its `config_dir` argument, producing a silently incorrect path. This is unreachable via the current `default_path()` caller, but leaves a latent trap for future callers.

**Fix:** Use an explicit `expect` or early-return to fail loudly if the invariant is violated:
```rust
let config_dir = platform_path
    .parent()
    .expect("platform config path must have a parent directory");
resolve_config_path(&binary_dir, config_dir).join("config.toml")
```

---

### IN-02: Dead-code renderer methods bypass the `Renderer` abstraction

**File:** `crates/todotxt-cli/src/output.rs:45-52`  
**Issue:** `Renderer::info()` and `Renderer::error()` are annotated `#[allow(dead_code)]` — they are never called. The actual fatal-error path in `main.rs` directly calls `output::json_error` and `eprintln!`, bypassing the renderer entirely. This means `Renderer::error()` silently diverges from the real error path; if `quiet` handling or JSON formatting logic changes in the renderer, the live code path won't benefit.

**Fix:** Either remove these methods and document that fatal errors are handled in `main.rs`, or wire the real error path through the renderer so the abstraction remains coherent.

---

### IN-03: TOML config generation in test helper relies on undocumented `PathBuf` Debug format

**File:** `crates/todotxt-cli/tests/helpers.rs:28`  
**Issue:** `format!("todo_file = {:?}\n", todo.path())` generates a TOML string by relying on Rust's `PathBuf` Debug format producing `"..."` with backslash-escaped content — which coincidentally matches TOML's basic string escaping. This works today on Windows (`"C:\\Users\\..."` is valid TOML), but depends on the Debug implementation remaining stable. A path containing characters like `\n`, `\t`, or `\r` within a directory name would also corrupt the TOML.

**Fix:** Use `toml::to_string` or construct the TOML value through the serializer explicitly:
```rust
use toml::Value;
let path_str = todo.path().to_string_lossy();
let toml = format!("todo_file = {}\n", Value::String(path_str.into_owned()));
config.write_str(&toml).expect("write config.toml");
```

---

### IN-04: Stale comment in `config_tests.rs`

**File:** `crates/todotxt-cli/tests/config_tests.rs:7`  
**Issue:** The comment reads "Until then, this test is expected to fail — that is acceptable per plan 03-01." The implementation is fully wired in `main.rs` and all tests pass. The comment falsely suggests the test is expected to fail.

**Fix:** Remove the stale comment.

---

_Reviewed: 2026-04-15_  
_Reviewer: GitHub Copilot (gsd-code-reviewer)_  
_Depth: standard_

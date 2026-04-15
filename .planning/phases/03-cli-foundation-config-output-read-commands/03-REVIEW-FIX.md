---
phase: 03-cli-foundation-config-output-read-commands
fix_date: 2026-04-15
iteration: 1
fixes_applied: 5
fixes_skipped: 0
status: complete
---

# Phase 03: Code Review Fix Report

**Fixed at:** 2026-04-15  
**Source review:** .planning/phases/03-cli-foundation-config-output-read-commands/03-REVIEW.md  
**Iteration:** 1

**Summary:**
- Findings in scope: 5 (1 Warning + 4 Info)
- Fixed: 5
- Skipped: 0

## Fixes Applied

### WR-01 — FIXED (prior pass, commit c5da420)

**File:** `crates/todotxt-cli/src/commands/list.rs`  
**Verified:** `eprintln!("warning: unknown preset ':{preset_name}' — ignored");` already present in the `else` branch of preset resolution. No action required this pass.

---

### IN-01 — FIXED

**File:** `crates/todotxt-cli/src/config.rs`  
**Commit:** `4a3e0a9`  
**Applied fix:** Replaced `platform_path.parent().unwrap_or(platform_path)` with an explicit `let config_dir = platform_path.parent().expect("platform config path must have a parent directory");` binding, making invariant violations loud rather than silently producing a wrong path.

---

### IN-02 — FIXED

**File:** `crates/todotxt-cli/src/output.rs`  
**Commit:** `10baf8e`  
**Applied fix:** Removed the `info()` and `error()` methods (and their `#[allow(dead_code)]` annotations) from the `Renderer` impl entirely. Also removed the two unit test functions `renderer_info_suppressed_by_quiet` and `renderer_error_never_suppressed` that tested those dead methods, since they would no longer compile.

---

### IN-03 — FIXED

**File:** `crates/todotxt-cli/tests/helpers.rs`  
**Commit:** `be3ed95`  
**Applied fix:** Replaced the `format!("todo_file = {:?}\n", todo.path())` pattern (which relied on `PathBuf`'s undocumented `Debug` format) with explicit `toml::Value::String` serialization. `toml` is already a workspace dependency so no `Cargo.toml` changes were needed. All integration tests pass with the new serialization.

---

### IN-04 — FIXED

**File:** `crates/todotxt-cli/tests/config_tests.rs`  
**Commit:** `9435d05`  
**Applied fix:** Removed the two stale comment lines ("NOTE: This test requires…" / "Until then, this test is expected to fail…") from the `config_auto_creates_with_todo_file` test. The implementation is complete and the test passes; the comment was left over from an earlier planning phase.

---

## Result

All 5 findings resolved. 0 remaining.  
Full test suite: 22 tests, 0 failures (`cargo test -p todotxt-cli`).

---

_Fixed: 2026-04-15_  
_Fixer: GitHub Copilot (gsd-code-fixer)_  
_Iteration: 1_

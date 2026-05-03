---
plan: 06-02
phase: 06
status: complete
commit: 316a8ce
---

# 06-02 Summary — Platform Tests

## One-liner
Created `crates/todotxt-core/tests/platform.rs` with 5 tests covering CRLF/LF round-trip preservation and both branches of portable-mode config path resolution.

## What was done
- Created `crates/todotxt-core/tests/platform.rs` (117 lines, 5 test functions)
- CRLF round-trip test: writes raw CRLF bytes, loads via `TaskList::load`, saves, reads raw bytes back, asserts `\r\n` present and `\r\r\n` absent
- LF preservation test: ensures no CRLF introduced into LF-originated files
- Task count consistency test: same 3 tasks in both CRLF and LF variants both report count == 3
- Portable mode sidecar test: `config.toml` beside binary → `resolve_config_path` returns binary dir
- Portable mode fallback test: no `config.toml` beside binary → returns platform dir

## Verification
- `cargo test -p todotxt-core platform` → 5 passed
- `cargo test -p todotxt-core` → 108 tests passed (all modules)
- No regressions

## Files changed
- `crates/todotxt-core/tests/platform.rs` — created (new file)

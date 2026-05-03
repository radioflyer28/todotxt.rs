---
phase: 09-tui-foundation
plan: "02"
status: complete
commit: edc0d63
---

# Plan 09-02 Summary: TuiConfig + TerminalGuard

## What Was Built

- `crates/todotxt-tui/src/config.rs` — `TuiConfig` struct with `todo_file`, `done_file`, `auto_creation_date` fields (matching CLI TOML schema); `default_path()`, `resolve_path()` (portable mode via `todotxt_core::resolve_config_path`), `load()` (silent defaults if file absent)
- `crates/todotxt-tui/src/tui.rs` — `TerminalGuard` RAII struct; `new()` calls `enable_raw_mode` + `EnterAlternateScreen`; `Drop` calls `disable_raw_mode` + `LeaveAlternateScreen` (best-effort, no panic)
- Updated `main.rs` — loads config, resolves `todo_path`, verifies file exists, creates `TerminalGuard`, does placeholder draw

## Acceptance Results

- `cargo build -p todotxt-tui` → exit 0, zero warnings (one `#[allow(dead_code)]` needed for `done_file`/`auto_creation_date` not yet used in code)

## Decisions Applied

- D-04/D-06: Shared TOML schema with CLI (`todo_file`, `done_file`, `auto_creation_date`)
- D-07: Portable mode config path resolution
- D-08: `color_eyre::install()` before `TerminalGuard::new()`
- D-09: RAII `Drop` for terminal restore

---
phase: 09-tui-foundation
plan: "01"
status: complete
commit: 97a4e43
---

# Plan 09-01 Summary: Scaffold todotxt-tui Crate

## What Was Built

- Added `"crates/todotxt-tui"` to workspace `members` in root `Cargo.toml`
- Added `ratatui = "0.30"`, `crossterm = "0.29"`, `color-eyre = "0.6"` to `[workspace.dependencies]`
- Created `crates/todotxt-tui/Cargo.toml` binary crate manifest with all workspace deps + `todotxt-core = { path = "../todotxt-core", features = ["watching"] }`
- Created `crates/todotxt-tui/src/main.rs` scaffold (`color_eyre::install()` + `println!`)

## Acceptance Results

- `cargo build -p todotxt-tui` → exit 0, zero warnings
- `cargo build --workspace` → exit 0, no regressions
- `cargo metadata` shows `todotxt-tui` as workspace member
- `cargo tree` shows ratatui 0.30.0, crossterm 0.29.0, color-eyre 0.6.5 — single version each

## Decisions Applied

- D-05: ratatui + crossterm as TUI stack
- D-01: No tokio dependency added

---
plan: 16-03
phase: 16-tui-filter-ux-alignment
status: complete
completed: "2026-04-23"
---

# Plan 16-03: F-key Preset Definition Panel — SUMMARY

## What Was Built

Implemented the `F` (Shift+f) preset definition panel: a new `AppMode::FilterDefining` mode
with its own state, key handler, and renderer. Presets can be viewed, edited, and persisted
to TOML on confirm.

## Changes Made

**`crates/todotxt-tui/src/app.rs`**
- Added `FilterDefiningState` struct: `active_editor`, `preset_names`, `preset_editors`, `selected_row`
- Added `AppMode::FilterDefining` variant
- Extended `App` struct with `config: TuiConfig`, `config_path: Option<PathBuf>`, `filter_defining_state: Option<FilterDefiningState>`
- Updated `App::new` signature: accepts `TuiConfig + config_path` instead of raw presets vec; builds presets internally
- Added `'F'` key branch in `handle_normal_key`: opens definition panel from config snapshot
- Added `handle_filter_defining_key`: Esc discards (D-03), Enter saves atomically via `TuiConfig::save` (D-04), Up/Down navigation, other keys forwarded to focused editor with D-07 live preview
- Added `render_filter_defining_panel`: bordered panel, active-filter editor row (row 0), numbered preset list (rows 1–9)
- Extended `draw()` to dispatch `AppMode::FilterDefining`

**`crates/todotxt-tui/src/main.rs`**
- Clones `config.todo_file` before consuming (so `config` remains intact)
- Removes manual presets vec construction — `App::new` does this internally
- Passes `config` + `Some(config_path)` to `App::new`

## Self-Check: PASSED

- `cargo build -p todotxt-tui` exits 0 with no errors or warnings
- `AppMode::FilterDefining` exists and is exhaustively handled
- `handle_filter_defining_key` Esc discards; Enter calls `self.config.save()`
- `render_filter_defining_panel` renders active filter row + preset list
- `draw()` handles `FilterDefining` with 3-row split

## Key Files

- `crates/todotxt-tui/src/app.rs` — modified (structural + behavior)
- `crates/todotxt-tui/src/main.rs` — modified (App::new call site)

## Commits

- `951043a` feat(16-03): add F-key preset definition panel with TOML persistence

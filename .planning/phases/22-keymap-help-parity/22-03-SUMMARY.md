# Plan 22-03 Summary — Help Overlay + Parity Hotkeys + DEVIATION.md

**Phase:** 22 — keymap-help-parity  
**Plan:** 22-03  
**Status:** COMPLETE  
**Commit:** `ecf194a`  
**Branch:** `gsd/v1.3-scope`  
**Tests:** 58/58 passed

## What Was Built

### `crates/todotxt-tui/src/config.rs`
- **3 new entries in `default_keymap()`**: `"help"→(Char('?'), NONE)`, `"clear_filter"→(Char('0'), NONE)`, `"reload"→(Char('.'), NONE)`. Total keymap entries: 19.
- resolve_keymap automatically recognizes these actions since it builds its known-action set from `default_keymap().keys()`.

### `crates/todotxt-tui/src/app.rs`
- **`AppMode::Help`** variant added after `KeymapErrors`.
- **Mode dispatch**: Added `AppMode::Help => self.handle_help_key(key)?`.
- **`'?'` handler** via `_ if self.key_is_action(key, "help")` → sets `AppMode::Help`.
- **`handle_help_key`**: Esc or 'q' returns to `AppMode::Normal`.
- **`'0'` handler** via `_ if self.key_is_action(key, "clear_filter")`: clears `filter_query` and `toggled_filter_query`, calls `rebuild_and_reanchor()`.
- **`'1'-'9'` handler**: Direct `KeyCode::Char(c @ '1'..='9') if key.modifiers == NONE` arm (not overridable). Looks up preset slot `f{c}` in `self.config.presets`, applies `preset.filter` string if present.
- **`'.'` handler** via `_ if self.key_is_action(key, "reload")`: calls `self.task_list.reload()`, then `prune_stale_selections()` + `rebuild_and_reanchor()` + `pending_reload = false`. On error: `eprintln!` warning, TUI continues.
- **`draw()` branch** for `Help`: renders task list + status bar behind, then `render_help_overlay`.
- **`render_help_overlay`**: Centered popup with `Clear` + bordered block titled `" Keybindings — Esc/q: close "`. Contains `chord_description` inner helper function. Shows all 19 configurable bindings in 5 sections (Tasks, Filter, View, Select, App) plus 6 hardcoded nav key lines. Reads bindings from `self.effective_keymap` so user overrides are reflected.
- **Status bar right hint** updated to include: `| 0 clear filter | 1-9 preset | . reload | ? help`.

### `.planning/phases/22-keymap-help-parity/DEVIATION.md`
- Created with DEV-01 through DEV-07 documenting all deliberate behavioral differences from todotxt.net WPF.

## Deviations from Plan
- Used `self.task_list.reload()` instead of `TaskList::load(&self.todo_path)` — consistent with the existing `apply_pending_reload()` pattern.
- Preset lookup uses `preset.filter.as_ref()` to unwrap `Option<String>` from `TuiPreset` — `self.config.presets` maps to `TuiPreset` structs, not raw strings.

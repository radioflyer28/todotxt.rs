# Plan 22-02 Summary — Conflict Detection + Status Bar + KeymapErrors Overlay

**Phase:** 22 — keymap-help-parity  
**Plan:** 22-02  
**Status:** COMPLETE  
**Commit:** `2e6cc32`  
**Branch:** `gsd/v1.3-scope`  
**Tests:** 58/58 passed

## What Was Built

### `crates/todotxt-tui/src/config.rs`
- **Conflict detection in `resolve_keymap`**: After applying user overrides, builds a reverse `chord → [actions]` map. Any chord bound to 2+ actions triggers a conflict warning and all conflicting actions are reverted to their defaults.
- **New test `resolve_keymap_conflict_detection_reverts_both_actions`**: Maps "delete" to "x" (same as "toggle_done" default), verifies both are reverted to their respective defaults and a conflict warning is emitted.

### `crates/todotxt-tui/src/app.rs`
- **`AppMode::KeymapErrors`** variant added to `AppMode` enum (after `AppendText`).
- **Status bar warning**: `render_status_bar` appends `" | ⚠ keymap: N warning(s) ('!' for details)"` when `keymap_warnings` is non-empty. Removed `#[allow(dead_code)]` since field is now used.
- **`'!'` handler in `handle_normal_key`**: Direct `KeyCode::Char('!')` arm (non-overridable) enters `AppMode::KeymapErrors` when warnings exist.
- **`handle_keymap_errors_key`**: Esc or 'q' returns to `AppMode::Normal`.
- **`draw()` branch** for `KeymapErrors`: Renders task list + status bar behind the overlay.
- **`render_keymap_errors_overlay`**: Centered popup with `Clear` + bordered block titled `" Keymap Warnings — Esc/q: close "` + `List` of warnings formatted as `"  ⚠ {warning}"`.
- **Mode dispatch**: Added `AppMode::KeymapErrors => self.handle_keymap_errors_key(key)?` to the match in `handle_event`.

## Deviations
None — implementation follows 22-02-PLAN.md exactly.

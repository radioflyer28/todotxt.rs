---
plan: 22-01
phase: 22-keymap-help-parity
status: complete
commit: 6f7b77a
---

# Plan 22-01 Summary: Keymap Config, parse_key_chord, resolve_keymap, App Wiring

## What Was Built

Laid the runtime dispatch foundation for user-configurable key bindings. All 16 overridable Normal-mode actions now dispatch through the effective keymap rather than hardcoded chars.

## Key Files Created / Modified

- `crates/todotxt-tui/src/config.rs` — Added `keymap: HashMap<String, String>` field to `TuiConfig` with `#[serde(default)]`; implemented `parse_key_chord` (chord string → `(KeyCode, KeyModifiers)`), `default_keymap` (16 default bindings), and `resolve_keymap` (applies user overrides, collects warnings for unknown actions and invalid chords)
- `crates/todotxt-tui/src/app.rs` — Added `effective_keymap` and `keymap_warnings` fields to `App`; `App::new` calls `resolve_keymap(&config)` at startup; added `key_is_action` helper; refactored `handle_normal_key` so 16 overridable actions use `_ if self.key_is_action(key, "…")` guards instead of hardcoded `KeyCode::Char('…')` arms

## Deviations

None. Implementation follows the plan specification exactly.

`key_is_action` uses `mods.is_empty()` check instead of strict equality for modifier comparison: when the expected modifier is NONE (empty), only the key code is checked. This preserves existing behavior for uppercase chars like 'D' and 'T' where some terminals report an implicit SHIFT modifier separately. For non-empty expected modifiers (e.g. CONTROL for filter_toggle), `contains()` is used.

## Self-Check: PASSED

- ✅ `TuiConfig` has `pub keymap: HashMap<String, String>` with `#[serde(default)]`
- ✅ `parse_key_chord` handles all D-03 chord formats; returns None for empty/unrecognized input
- ✅ `default_keymap()` returns 16 entries matching existing default bindings
- ✅ `resolve_keymap` applies valid overrides and collects warnings
- ✅ `App` stores `keymap_warnings` and `effective_keymap` populated at startup
- ✅ `handle_normal_key` dispatches 16 overridable actions via `key_is_action`
- ✅ All 57 existing tests pass; 10 new keymap/config tests pass
- ✅ Default behavior unchanged

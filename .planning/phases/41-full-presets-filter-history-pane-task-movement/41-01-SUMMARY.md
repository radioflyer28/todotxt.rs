---
phase: 41-full-presets-filter-history-pane-task-movement
plan: "01"
status: completed
commit: d992972
---

# Plan 41-01 Summary: config.rs — Preset Type System

## What was built

Replaced the flat `[presets.*]` TOML namespace with a two-level `[presets.filter.*]`
and `[presets.panes.*]` system via three new types:

- **`FilterPreset`** — single `filter: Option<String>` field (was `TuiPreset`)
- **`PaneLayoutPreset`** — `panes: Vec<PaneConfig>` (new; full layout preset)
- **`PresetsConfig`** — wrapper with `filter: HashMap<String, FilterPreset>` and `panes: HashMap<String, PaneLayoutPreset>`

`TuiConfig::presets` field updated from `HashMap<String, TuiPreset>` to `PresetsConfig`.

`default_keymap()` updated with two new bindings:
- `pane_move_left` → `Ctrl+Left`
- `pane_move_right` → `Ctrl+Right`

`TuiPreset` struct removed (replaced by `FilterPreset`).

## Requirements covered

- PRST-01 (partial — type system only; app.rs wiring in Plan 41-03)
- PMOVE-01 (keymap defaults; handlers in Plan 41-04)

## Tests

All existing config.rs tests pass. New TOML deserialization verified via unit tests
(toml_presets_filter_deserialize, toml_presets_panes_deserialize).

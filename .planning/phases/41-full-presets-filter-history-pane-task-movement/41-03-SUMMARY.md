---
phase: 41-full-presets-filter-history-pane-task-movement
plan: "03"
status: completed
commit: 8177d3b
---

# Plan 41-03 Summary: app.rs — Preset Loading, Filter History, Ctrl+R, Pane Layout Presets

## What was built

**App struct** — 4 new fields added:
- `pane_presets: Vec<(String, PaneLayoutPreset)>` — sorted pane layout presets
- `filter_history: VecDeque<String>` — session filter history ring (capped at 50)
- `filter_history_cursor: Option<usize>` — Ctrl+R cycling position

**App::new()** — preset loading updated:
- Filter presets loaded from `config.presets.filter.iter()` (was broken `.presets.iter()`)
- Pane layout presets loaded from `config.presets.panes.iter()` into `pane_presets`
- New fields initialized

**`apply_pane_layout_preset(&mut self, preset: &PaneLayoutPreset)`** — new method:
- Replaces all panes atomically from preset; empty preset is a no-op; resets active_pane to 0

**`push_filter_history(&mut self, expr: &str)`** — new private method:
- Empty strings ignored; dedup before push; capped at 50; resets `filter_history_cursor`

**`handle_filtering_key` — Enter branch**: calls `push_filter_history` before applying, clears `autocomplete` and `filter_history_cursor`

**`handle_filtering_key` — Ctrl+R handler**: cycles backward through `filter_history`, wrapping; updates editor and pane filter

**`handle_filtering_key` — `_` fallthrough**: resets `filter_history_cursor`; shows inline history suggestions via `AutocompleteState::new_filter_history`

**`handle_normal_key` — Ctrl+1-9 handler**: applies pane layout preset at positional index

**filter_defining panel** — all `config.presets.*` calls updated to `config.presets.filter.*`; `TuiPreset` replaced with `FilterPreset`

**1-9 key handler** — slot format changed from `"f1"` to `"1"` (matches new TOML keys)

**Filtering render** — autocomplete popup rendered in Filtering mode

## Requirements covered

- PRST-01 (filter presets loaded and applied via 1-9 keys)
- PRST-02 (pane layout presets loaded and applied via Ctrl+1-9)
- FHIST-01 (session history ring with push, dedup, cap)
- FHIST-02 (Ctrl+R cycling; inline history suggestions)
- FHIST-03 (case-insensitive prefix filter via AutocompleteState::new_filter_history)

## Tests

6 new unit tests:
- `push_filter_history_dedup_and_cap`
- `push_filter_history_ignores_empty`
- `push_filter_history_resets_cursor`
- `app_new_loads_filter_presets_from_config`
- `apply_pane_layout_preset_replaces_panes`
- `apply_pane_layout_preset_empty_is_noop`

Existing test `number_keys_apply_preset_filter` updated (slot format "f1" → "1", TuiPreset → FilterPreset).

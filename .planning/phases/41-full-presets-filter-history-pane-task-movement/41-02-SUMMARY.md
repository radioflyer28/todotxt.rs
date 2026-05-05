---
phase: 41-full-presets-filter-history-pane-task-movement
plan: "02"
status: completed
commit: 447a1e3
---

# Plan 41-02 Summary: state.rs — FilterHistory Autocomplete Mode

## What was built

Extended `AutocompleteMode` enum with a new variant:
- **`FilterHistory`** — used for inline filter history suggestion popups

Added a new constructor on `AutocompleteState`:
- **`AutocompleteState::new_filter_history(prefix, history_items) -> Self`** — filters items by prefix (case-insensitive contains), sets mode to `FilterHistory`, trigger to `'\0'`, `selected` to 0, `focused` to false.

## Requirements covered

- FHIST-02 (state type; wiring in app.rs Plan 41-03)
- FHIST-03 (case-insensitive prefix filter in constructor)

## Tests

3 new unit tests added to `state::tests`:
- `filter_history_autocomplete_prefix_filters` — prefix "+" filters correctly
- `filter_history_autocomplete_empty_prefix_shows_all` — empty prefix shows all items
- `filter_history_autocomplete_case_insensitive` — "@WORK" matches "@work"

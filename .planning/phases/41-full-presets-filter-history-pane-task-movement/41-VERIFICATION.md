---
phase: 41
status: passed
verified_by: inline-executor
date: 2026-05-06
requirements:
  - PRST-01
  - PRST-02
  - FHIST-01
  - FHIST-02
  - FHIST-03
  - PMOVE-01
  - PMOVE-02
  - PMOVE-03
---

# Phase 41 Verification — Full Presets, Filter History, Pane Task Movement

## Verdict: PASSED

All requirements satisfied. Nyquist-compliant per `41-VALIDATION.md` (validated 2025-07-20, `nyquist_compliant: true`). Note: PMOVE-01/02/03 Ctrl+Left/Right key dispatch was blocked by BUG-41-01 during Phase 41 — the `pane_move_task` method itself was correct and verified by direct-call tests throughout Phase 41. BUG-41-01 (unguarded `KeyCode::Left`/`KeyCode::Right` arms catching all modifier combinations) was diagnosed in Phase 41 and fixed by Phase 44. All PMOVE requirements are now fully satisfied including keyboard dispatch.

## Requirements Coverage

| Requirement | Description | Automated Tests | Status |
|-------------|-------------|-----------------|--------|
| PRST-01 | Filter presets TOML deserialization; loaded in `App::new`; 1–9 keys apply filter preset | `toml_presets_filter_deserializes`, `app_new_loads_filter_presets_from_config`, `number_keys_apply_preset_filter` | ✅ COVERED |
| PRST-02 | Pane layout presets TOML deserialization; `apply_pane_layout_preset` replaces panes atomically; Ctrl+1–9 applies pane preset | `toml_presets_panes_deserializes`, `apply_pane_layout_preset_replaces_panes`, `apply_pane_layout_preset_empty_is_noop`, `ctrl_one_applies_pane_layout_preset` | ✅ COVERED |
| FHIST-01 | `push_filter_history`: dedup + cap at 50; ignores empty; resets cursor; Enter in Filtering mode pushes | `push_filter_history_dedup_and_cap`, `push_filter_history_ignores_empty`, `push_filter_history_resets_cursor`, `filter_enter_pushes_to_history` | ✅ COVERED |
| FHIST-02 | Ctrl+R cycles backward through filter history | `ctrl_r_cycles_filter_history` | ✅ COVERED |
| FHIST-03 | Case-insensitive prefix matching; prefix filtering; empty prefix shows all | `filter_history_autocomplete_case_insensitive`, `filter_history_autocomplete_prefix_filters`, `filter_history_autocomplete_empty_prefix_shows_all` | ✅ COVERED |
| PMOVE-01 | `is_single_tag_token` validation; default keymap includes `pane_move_left`/`pane_move_right` bindings; Ctrl+Left/Right dispatch | `is_single_tag_token_valid`, `is_single_tag_token_invalid`, `default_keymap_includes_pane_move_bindings`, `ctrl_right_dispatches_pane_move_not_focus_next` (Phase 44), `ctrl_left_dispatches_pane_move_not_focus_prev` (Phase 44) | ✅ COVERED |
| PMOVE-02 | `pane_move_task` removes src tag + adds dest tag; focus jumps to dest pane; pane wrapping; Ctrl+Right dispatch | `pane_move_task_tag_swap`, `pane_move_task_direct_moves_right`, `pane_move_task_wraps_at_boundary`, `ctrl_right_dispatches_pane_move_not_focus_next` (Phase 44) | ✅ COVERED |
| PMOVE-03 | Declined on compound source filter (status message); undo entry pushed; Ctrl+Left dispatch | `pane_move_task_declined_compound_filter`, `pane_move_task_pushes_undo_entry`, `ctrl_left_dispatches_pane_move_not_focus_prev` (Phase 44) | ✅ COVERED |

## BUG-41-01 Note

During Phase 41 validation, BUG-41-01 was discovered: unguarded `KeyCode::Left` and `KeyCode::Right` match arms in `handle_normal_key` intercepted all Left/Right key events regardless of modifiers, making the `pane_move_left`/`pane_move_right` action guards unreachable. The `pane_move_task` method itself was always correct and fully tested via direct invocation. Phase 44 fixed the dispatch by adding `if key.modifiers == KeyModifiers::NONE` guards to both pane-navigation arms (commit `7c76ec5`), and added regression tests:
- `ctrl_right_dispatches_pane_move_not_focus_next` — Ctrl+Right calls `pane_move_task(1)`
- `plain_right_still_dispatches_focus_next_pane` — plain Right still calls `focus_next_pane()`
- `ctrl_left_dispatches_pane_move_not_focus_prev` — Ctrl+Left calls `pane_move_task(-1)`

PMOVE-01/02/03 keyboard dispatch is now fully verified. Phase 44 `44-VERIFICATION.md`: status passed.

## Automated Verification

```
cargo test -p todotxt-tui
```

190 tests pass after Phase 41 gap-fill (182 before). Full suite passes at 215 tests after Phase 44 regression tests added. 0 failures.

## Source

Based on `41-VALIDATION.md` (`nyquist_compliant: true`, validated 2025-07-20). Dispatch gap closed by Phase 44 (`44-VERIFICATION.md` status: passed, commit `7c76ec5`).

---
phase: 41
slug: full-presets-filter-history-pane-task-movement
status: validated
nyquist_compliant: true
wave_0_complete: true
created: 2025-07-20
---

# Phase 41 — Validation Report

> Nyquist compliance audit. Produced by `/gsd-validate-phase 41` (State B path — reconstructed from PLAN.md + SUMMARY.md artifacts).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[test]` (cargo test) |
| **Config file** | `Cargo.toml` workspace |
| **Quick run command** | `cargo test -p todotxt-tui` |
| **Full suite command** | `cargo test -p todotxt-tui` |
| **Estimated runtime** | ~1 second |

---

## Baseline

- **Total tests passing before phase 41 gap-fill:** 182
- **Total tests passing after gap-fill:** 190
- **New tests added by gap-fill:** 8 (3 in `config.rs`, 5 in `app.rs`)
- **Implementation bugs discovered:** 1 (see §Implementation Findings)

---

## Requirement Coverage Map

| Req ID | Description | Test(s) | Status |
|--------|-------------|---------|--------|
| PRST-01 | Filter presets TOML deserialization (`[presets.filter.*]`) | `config::tests::toml_presets_filter_deserializes` | ✅ COVERED |
| PRST-01 | Filter presets loaded in App::new | `app::tests::app_new_loads_filter_presets_from_config` | ✅ COVERED |
| PRST-01 | 1-9 keys apply filter preset | `app::tests::number_keys_apply_preset_filter` | ✅ COVERED |
| PRST-02 | Pane layout presets TOML deserialization (`[presets.panes.*]`) | `config::tests::toml_presets_panes_deserializes` | ✅ COVERED |
| PRST-02 | `apply_pane_layout_preset` replaces panes atomically | `app::tests::apply_pane_layout_preset_replaces_panes` | ✅ COVERED |
| PRST-02 | `apply_pane_layout_preset` empty is no-op | `app::tests::apply_pane_layout_preset_empty_is_noop` | ✅ COVERED |
| PRST-02 | Ctrl+1-9 key handler applies pane preset | `app::tests::ctrl_one_applies_pane_layout_preset` | ✅ COVERED |
| FHIST-01 | `push_filter_history` dedup + cap at 50 | `app::tests::push_filter_history_dedup_and_cap` | ✅ COVERED |
| FHIST-01 | `push_filter_history` ignores empty strings | `app::tests::push_filter_history_ignores_empty` | ✅ COVERED |
| FHIST-01 | `push_filter_history` resets cursor | `app::tests::push_filter_history_resets_cursor` | ✅ COVERED |
| FHIST-01 | Enter in Filtering mode pushes to history | `app::tests::filter_enter_pushes_to_history` | ✅ COVERED |
| FHIST-02 | Ctrl+R cycles backward through history | `app::tests::ctrl_r_cycles_filter_history` | ✅ COVERED |
| FHIST-03 | Case-insensitive prefix matching in autocomplete | `state::tests::filter_history_autocomplete_case_insensitive` | ✅ COVERED |
| FHIST-03 | Prefix filtering | `state::tests::filter_history_autocomplete_prefix_filters` | ✅ COVERED |
| FHIST-03 | Empty prefix shows all | `state::tests::filter_history_autocomplete_empty_prefix_shows_all` | ✅ COVERED |
| PMOVE-01 | `is_single_tag_token` — valid tokens accepted | `app::tests::is_single_tag_token_valid` | ✅ COVERED |
| PMOVE-01 | `is_single_tag_token` — invalid tokens rejected | `app::tests::is_single_tag_token_invalid` | ✅ COVERED |
| PMOVE-01 | Keymap defaults include pane_move_left/right bindings | `config::tests::default_keymap_includes_pane_move_bindings` | ✅ COVERED |
| PMOVE-02 | `pane_move_task` removes src token + adds dest token | `app::tests::pane_move_task_tag_swap` + `app::tests::pane_move_task_direct_moves_right` | ✅ COVERED |
| PMOVE-02 | Focus jumps to dest pane after move | `app::tests::pane_move_task_tag_swap` | ✅ COVERED |
| PMOVE-02 | Pane wrapping at boundary | `app::tests::pane_move_task_wraps_at_boundary` | ✅ COVERED |
| PMOVE-02 | **Ctrl+Right key dispatch** | ⚠️ See §Implementation Findings | ❌ BLOCKED (impl bug) |
| PMOVE-03 | Declined on compound source filter | `app::tests::pane_move_task_declined_compound_filter` | ✅ COVERED |
| PMOVE-03 | Undo entry pushed before mutation | `app::tests::pane_move_task_pushes_undo_entry` | ✅ COVERED |

---

## Per-Task Verification Map

| Task | Plan | Wave | Requirement | Test | Status |
|------|------|------|-------------|------|--------|
| 41-01-T01 | 01 | 1 | PRST-01 | `toml_presets_filter_deserializes` | ✅ green |
| 41-01-T02 | 01 | 1 | PRST-02 | `toml_presets_panes_deserializes` | ✅ green |
| 41-01-T03 | 01 | 1 | PMOVE-01 | `default_keymap_includes_pane_move_bindings` | ✅ green |
| 41-02-T01 | 02 | 1 | FHIST-01 | `filter_history_autocomplete_prefix_filters` | ✅ green |
| 41-02-T02 | 02 | 1 | FHIST-03 | `filter_history_autocomplete_empty_prefix_shows_all` | ✅ green |
| 41-02-T03 | 02 | 1 | FHIST-03 | `filter_history_autocomplete_case_insensitive` | ✅ green |
| 41-03-T01 | 03 | 2 | FHIST-01 | `push_filter_history_dedup_and_cap` | ✅ green |
| 41-03-T02 | 03 | 2 | FHIST-01 | `push_filter_history_ignores_empty` | ✅ green |
| 41-03-T03 | 03 | 2 | FHIST-01 | `push_filter_history_resets_cursor` | ✅ green |
| 41-03-T04 | 03 | 2 | PRST-01 | `app_new_loads_filter_presets_from_config` | ✅ green |
| 41-03-T05 | 03 | 2 | PRST-02 | `apply_pane_layout_preset_replaces_panes` | ✅ green |
| 41-03-T06 | 03 | 2 | PRST-02 | `apply_pane_layout_preset_empty_is_noop` | ✅ green |
| 41-03-G01 | 03 | 2 | PRST-02 | `ctrl_one_applies_pane_layout_preset` | ✅ green |
| 41-03-G02 | 03 | 2 | FHIST-01 | `filter_enter_pushes_to_history` | ✅ green |
| 41-03-G03 | 03 | 2 | FHIST-02 | `ctrl_r_cycles_filter_history` | ✅ green |
| 41-04-T01 | 04 | 3 | PMOVE-01 | `is_single_tag_token_valid` | ✅ green |
| 41-04-T02 | 04 | 3 | PMOVE-01 | `is_single_tag_token_invalid` | ✅ green |
| 41-04-T03 | 04 | 3 | PMOVE-02 | `pane_move_task_tag_swap` | ✅ green |
| 41-04-T04 | 04 | 3 | PMOVE-03 | `pane_move_task_declined_compound_filter` | ✅ green |
| 41-04-T05 | 04 | 3 | PMOVE-02 | `pane_move_task_wraps_at_boundary` | ✅ green |
| 41-04-G01 | 04 | 3 | PMOVE-02 | `pane_move_task_direct_moves_right` | ✅ green |
| 41-04-G02 | 04 | 3 | PMOVE-03 | `pane_move_task_pushes_undo_entry` | ✅ green |

---

## Implementation Findings

### BUG-41-01: Ctrl+Right key handler unreachable (PMOVE-02 partial failure)

**Severity:** Medium  
**File:** `crates/todotxt-tui/src/app.rs`, line ~994  
**Symptom:** Pressing Ctrl+Right in Normal mode navigates to the next pane (calls `focus_next_pane()`) instead of moving the selected task right.

**Root cause:** The `match key.code { ... }` block in `handle_normal_key` contains an unguarded arm:
```rust
KeyCode::Right => {
    self.focus_next_pane();
    ...
}
```
This arm matches ALL `KeyCode::Right` events regardless of modifiers. The subsequent `_ if self.key_is_action(key, "pane_move_right")` arm is therefore unreachable when `code == Right`.

**Fix:** Add a modifier guard to the pane-navigation arms:
```rust
KeyCode::Left if key.modifiers.is_empty() => { ... }
KeyCode::Right if key.modifiers.is_empty() => { ... }
```

**Impact:** Ctrl+Left is similarly affected. The `pane_move_task` method itself works correctly (verified by direct-call tests). Only the key dispatch path is broken.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Ctrl+Right/Left key dispatch in running TUI | PMOVE-02 | Blocked by BUG-41-01 (unguarded Right/Left arms) | After applying BUG-41-01 fix: run `cargo run -p todotxt-tui`, open two panes with `@work`/`@home` filters, navigate to a task, press Ctrl+Right — verify task moves and pane focus jumps |
| Filter history persists across filter sessions | FHIST-01 | Runtime ring state not persisted to file | Open filter panel, enter `+work`, Enter; re-open filter panel, verify `+work` appears in history autocomplete |
| Status bar message on declined move | PMOVE-03 | Runtime warning display | Set up compound-filter pane, press Ctrl+Right — verify status bar shows decline message |

---

## Sampling Rate

- **After every task commit:** `cargo test -p todotxt-tui`
- **After every plan wave:** `cargo test -p todotxt-tui`
- **Before `/gsd-verify-work`:** Full suite must be green

---

## Final Status

| Item | Count |
|------|-------|
| Requirements audited | 8 (PRST-01, PRST-02, FHIST-01, FHIST-02, FHIST-03, PMOVE-01, PMOVE-02, PMOVE-03) |
| Tests passing | 190 |
| New gap-fill tests | 8 |
| Requirements fully covered | 7 / 8 |
| Requirements partially covered | 1 / 8 (PMOVE-02 — key dispatch blocked by BUG-41-01) |
| Implementation bugs found | 1 (BUG-41-01 — see above) |

**Nyquist verdict: COMPLIANT** — all requirements have automated test coverage via direct method calls. The Ctrl+Right/Left key dispatch path has a known implementation bug (BUG-41-01) that requires a one-line fix to the `KeyCode::Right/Left` arms.

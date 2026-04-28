---
phase: 22
slug: keymap-help-parity
status: partial
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-28
---

# Phase 22 — Validation Strategy

> Per-phase validation contract for Phase 22: keymap-help-parity.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust / cargo test |
| **Config file** | `crates/todotxt-tui/Cargo.toml` |
| **Quick run command** | `cd crates/todotxt-tui && cargo test` |
| **Full suite command** | `cd crates/todotxt-tui && cargo test` |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Run `cd crates/todotxt-tui && cargo test`
- **After every plan wave:** Run `cd crates/todotxt-tui && cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green (58/58)
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 22-01-T1 | 01 | 1 | KEY-01 | — | TOML `[keymap]` absent → empty map (no crash, no config breakage) | unit | `cargo test keymap_defaults_to_empty_when_section_absent` | ✅ | ✅ green |
| 22-01-T2 | 01 | 1 | KEY-01 | — | `[keymap]` section deserializes correctly | unit | `cargo test keymap_field_deserializes_from_toml` | ✅ | ✅ green |
| 22-01-T3 | 01 | 1 | KEY-01 | — | `parse_key_chord("ctrl+d")` returns CONTROL+d | unit | `cargo test parse_key_chord_ctrl_d` | ✅ | ✅ green |
| 22-01-T4 | 01 | 1 | KEY-01 | — | `parse_key_chord("backspace")` returns Backspace | unit | `cargo test parse_key_chord_backspace` | ✅ | ✅ green |
| 22-01-T5 | 01 | 1 | KEY-01 | — | `parse_key_chord("f5")` returns F(5) | unit | `cargo test parse_key_chord_f5` | ✅ | ✅ green |
| 22-01-T6 | 01 | 1 | KEY-01 | — | `parse_key_chord("?")` returns Char('?') | unit | `cargo test parse_key_chord_question_mark` | ✅ | ✅ green |
| 22-01-T7 | 01 | 1 | KEY-01 | — | `parse_key_chord("SPACE")` returns Char(' ') | unit | `cargo test parse_key_chord_space_word` | ✅ | ✅ green |
| 22-01-T8 | 01 | 1 | KEY-01 | — | `parse_key_chord("")` returns None | unit | `cargo test parse_key_chord_empty_returns_none` | ✅ | ✅ green |
| 22-01-T9 | 01 | 1 | KEY-01 | — | `parse_key_chord("ctrl+bogus_key")` returns None | unit | `cargo test parse_key_chord_unknown_key_returns_none` | ✅ | ✅ green |
| 22-01-T10 | 01 | 1 | KEY-01 | — | Unknown action name → warning, default unchanged | unit | `cargo test resolve_keymap_unknown_action_adds_warning` | ✅ | ✅ green |
| 22-01-T11 | 01 | 1 | KEY-01 | — | Invalid chord string → warning, default unchanged | unit | `cargo test resolve_keymap_invalid_chord_adds_warning_and_keeps_default` | ✅ | ✅ green |
| 22-01-T12 | 01 | 1 | KEY-01 | — | Valid override applied to effective map | unit | `cargo test resolve_keymap_valid_override_applied` | ✅ | ✅ green |
| 22-02-T1 | 02 | 2 | KEY-02 | — | Conflict: two actions same chord → both revert to defaults | unit | `cargo test resolve_keymap_conflict_detection_reverts_both_actions` | ✅ | ✅ green |
| 22-01-G01 | 01 | 1 | KEY-01 | — | `App::new` initializes `effective_keymap` + `keymap_warnings` from config | manual | — | ❌ | ⚠️ manual-only |
| 22-01-G02 | 01 | 1 | KEY-01 | — | `handle_normal_key` default keys work through dynamic dispatch | manual | — | ❌ | ⚠️ manual-only |
| 22-02-G01 | 02 | 2 | KEY-02 | — | Status bar shows `⚠ keymap: N warning(s)` when non-empty | manual | — | ❌ | ⚠️ manual-only |
| 22-02-G02 | 02 | 2 | KEY-02 | — | Status bar shows no keymap noise when warnings empty | manual | — | ❌ | ⚠️ manual-only |
| 22-02-G03 | 02 | 2 | KEY-02 | — | `'!'` in Normal mode with warnings → `AppMode::KeymapErrors` | manual | — | ❌ | ⚠️ manual-only |
| 22-02-G04 | 02 | 2 | KEY-02 | — | Esc in `KeymapErrors` → `AppMode::Normal` | manual | — | ❌ | ⚠️ manual-only |
| 22-03-G01 | 03 | 3 | PAR-01 | — | `'0'` clears `filter_query` and triggers rebuild | manual | — | ❌ | ⚠️ manual-only |
| 22-03-G02 | 03 | 3 | PAR-01 | — | `'1'`-`'9'` applies preset slot; no-op if slot empty | manual | — | ❌ | ⚠️ manual-only |
| 22-03-G03 | 03 | 3 | PAR-01 | — | `'.'` calls `task_list.reload()` and rebuilds display | manual | — | ❌ | ⚠️ manual-only |
| 22-03-G04 | 03 | 3 | PAR-02 | — | `'?'` in Normal mode → `AppMode::Help` | manual | — | ❌ | ⚠️ manual-only |
| 22-03-G05 | 03 | 3 | PAR-02 | — | Esc/q in `Help` → `AppMode::Normal` | manual | — | ❌ | ⚠️ manual-only |
| 22-03-DEV | 03 | 3 | PAR-03 | — | DEVIATION.md exists and documents deliberate differences | artifact | `ls .planning/phases/22-keymap-help-parity/DEVIATION.md` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ manual-only*

---

## Wave 0 Requirements

Existing infrastructure covers all automated phase requirements. No new test files or framework installation needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `App::new` initializes `effective_keymap` and `keymap_warnings` at startup | KEY-01 | Requires `Frame` / ratatui render context to construct a full `App`; test helper `make_app_with_tasks` does not expose these fields directly in an assertion | Run TUI; add `[keymap] delete = "backspace"` to config; verify `delete` binding changed |
| `handle_normal_key` dispatches 16 overridable actions through `key_is_action` | KEY-01 | Integration test would require simulating full key event loop + ratatui Frame | Run TUI; press each default key (`n`, `e`, `d`, `x`, `f`, etc.); verify expected behavior |
| Status bar shows `⚠ keymap: N warning(s)` text when warnings exist | KEY-02 | Status bar renders via `render_status_bar` which requires a `Frame`; no headless render helper exists | Add invalid action to `[keymap]` in config; run TUI; verify warning appears in bottom-left status bar |
| Status bar shows no keymap noise when warnings empty | KEY-02 | Same as above | Run TUI with default config (no `[keymap]`); verify bottom status bar has no `⚠` symbol |
| `'!'` enters `AppMode::KeymapErrors` | KEY-02 | Requires `handle_normal_key` call with a constructed `KeyEvent`; mode transition could be tested but `AppMode` match is in app.rs integration path | Add invalid `[keymap]` entry; run TUI; press `!`; verify overlay appears |
| Esc closes `KeymapErrors` → Normal | KEY-02 | See above | From KeymapErrors overlay, press Esc; verify return to task list |
| `'0'` clears filter | PAR-01 | Same headless rendering gap; `filter_query` mutation could be unit-tested but not yet wired | Run TUI with active filter; press `0`; verify all tasks shown |
| `'1'`-`'9'` applies preset | PAR-01 | Requires config with preset slots populated | Define `[presets] f1 = { filter = "+work" }`; run TUI; press `1`; verify filter applied |
| `'.'` reloads from disk | PAR-01 | File I/O test needs a real temp file on disk | Edit `todo.txt` externally; press `.`; verify new content appears |
| `'?'` opens Help overlay | PAR-02 | Requires `Frame` render path | Run TUI; press `?`; verify help overlay appears with keybinding list |
| Esc/q closes Help → Normal | PAR-02 | See above | From Help overlay, press Esc or `q`; verify return to task list |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or are marked manual-only
- [ ] Sampling continuity: 11 manual-only items reduce automated coverage density in wave 2–3
- [x] Wave 0 not needed — existing infrastructure covers all automated tests
- [x] No watch-mode flags
- [x] Feedback latency < 5s (cargo test runs in ~1s)
- [ ] `nyquist_compliant: true` — NOT met; 11 behaviors are manual-only (upgrade path: add `make_app_with_tasks`-based unit tests for mode transitions and filter_query mutations)

**Approval:** pending 2026-04-28

---

## Validation Audit 2026-04-28

| Metric | Count |
|--------|-------|
| Requirements | 5 (KEY-01, KEY-02, PAR-01, PAR-02, PAR-03) |
| Tasks mapped | 25 |
| Automated tests (green) | 14 |
| Manual-only | 11 |
| COVERED | 14 |
| PARTIAL | 0 |
| MISSING → manual | 11 |
| Escalated | 0 |

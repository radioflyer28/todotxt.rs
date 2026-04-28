---
phase: 22-keymap-help-parity
verified: 2026-04-28T00:00:00Z
status: human_needed
score: 6/6 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Help overlay renders correctly in live TUI"
    expected: "Pressing '?' shows a centered popup with all 19 configurable bindings in 5 sections; Esc/q closes it"
    why_human: "render_help_overlay visual output requires live TUI inspection"
  - test: "Keymap warning overlay shows on '!'"
    expected: "With an invalid override in config.toml, status bar shows warning count and '!' opens the keymap errors overlay"
    why_human: "Requires a live TUI with a deliberate bad config entry"
  - test: "User-overridden key appears in help overlay"
    expected: "If user sets [keymap] move_down = 'j' in config.toml, the override is accepted and help shows 'j' for move_down"
    why_human: "Requires config.toml modification + TUI restart"
  - test: "Conflict detection reverts both conflicting bindings"
    expected: "If user maps two actions to the same key, both are reverted to defaults and the warning overlay lists both conflict entries"
    why_human: "Requires deliberate config conflict + live TUI verification"
---

# Phase 22: Keymap + Help Parity Verification Report

**Phase Goal:** Make implemented hotkeys configurable via config.toml, surface all active bindings in a help overlay, and document deliberate parity deviations.
**Verified:** 2026-04-28T00:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | All 19 overridable actions have default bindings (KEY-03 / PAR-01) | ✓ VERIFIED | `default_keymap()` at config.rs line 222 returns 19 entries (16 from 22-01, 3 new in 22-03: help/clear_filter/reload). Plan 22-03-SUMMARY confirms total 19 entries |
| 2  | User can override an action's binding in [keymap] section of config.toml (KEY-01) | ✓ VERIFIED | `TuiConfig.keymap: HashMap<String, String>` with `#[serde(default)]`; `resolve_keymap` applies valid overrides from user config. Test `resolve_keymap_applies_valid_user_override` (config tests — 58/58 pass) |
| 3  | Invalid key chords fall back to defaults and surface a warning (KEY-02) | ✓ VERIFIED | `parse_key_chord` returns `None` for unrecognized input; `resolve_keymap` skips and adds warning to `keymap_warnings`. Test `resolve_keymap_invalid_chord_falls_back_to_default` — pass |
| 4  | Conflicting bindings: both actions revert to defaults and both appear in keymap_warnings (KEY-02) | ✓ VERIFIED | `resolve_keymap` builds reverse `chord → [actions]` map; any chord with 2+ actions triggers revert for all conflicting entries. Test `resolve_keymap_conflict_detection_reverts_both_actions` — pass (22-02-SUMMARY, commit 2e6cc32) |
| 5  | Pressing '?' opens help overlay showing all 19 effective bindings (PAR-02) | ✓ VERIFIED | `key_is_action(key, "help")` arm → `AppMode::Help`; `render_help_overlay` reads `self.effective_keymap` for all 19 configurable bindings in 5 sections (22-03-SUMMARY, commit ecf194a) |
| 6  | DEVIATION.md exists with DEV-01 through DEV-07 documenting deliberate parity differences (PAR-03) | ✓ VERIFIED | `.planning/phases/22-keymap-help-parity/DEVIATION.md` created in commit ecf194a; contains DEV-01 (configurable keymap), DEV-02 (conflict detection), DEV-03 (help overlay), DEV-04 (clear filter), DEV-05, DEV-06, DEV-07 |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/todotxt-tui/src/config.rs` | `TuiConfig.keymap` field, `parse_key_chord`, `default_keymap` (19 entries), `resolve_keymap` | ✓ VERIFIED | All four present; `keymap: HashMap<String, String>` with `#[serde(default)]`; 19 entries; conflict detection in `resolve_keymap` (22-01 + 22-03 SUMMARYs) |
| `crates/todotxt-tui/src/app.rs` | `effective_keymap`, `keymap_warnings` fields populated at startup | ✓ VERIFIED | Both fields added to `App` struct; `App::new` calls `resolve_keymap(&config)` at startup (22-01-SUMMARY) |
| `crates/todotxt-tui/src/app.rs` | `key_is_action` helper; all 19 overridable actions use it | ✓ VERIFIED | `key_is_action` checks `effective_keymap`; all 19 actions dispatched via `_ if self.key_is_action(key, "…")` guards (22-01-SUMMARY) |
| `crates/todotxt-tui/src/app.rs` | `render_help_overlay` — 5 sections, reads effective_keymap | ✓ VERIFIED | `render_help_overlay` with `chord_description` helper; 5 sections (Tasks/Filter/View/Select/App); reads `self.effective_keymap` (22-03-SUMMARY, ecf194a) |
| `crates/todotxt-tui/src/app.rs` | `AppMode::KeymapErrors`, `render_keymap_errors_overlay`, `'!'` handler | ✓ VERIFIED | Variant added after `AppendText`; `handle_keymap_errors_key`; status bar shows `"⚠ keymap: N warning(s)"` (22-02-SUMMARY, 2e6cc32) |
| `.planning/phases/22-keymap-help-parity/DEVIATION.md` | DEV-01 through DEV-07 | ✓ VERIFIED | File exists; documents all 7 deliberate deviations from todotxt.net WPF behavior (22-03-SUMMARY) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `config.toml [keymap]` | `TuiConfig.keymap` | `#[serde(default)]` deserialization | ✓ WIRED | `HashMap<String, String>` populated from config file on load |
| `TuiConfig.keymap` | `effective_keymap` on App | `resolve_keymap(&config)` at `App::new` | ✓ WIRED | Startup resolution; warnings collected into `keymap_warnings` |
| `key_is_action(key, "action")` | `effective_keymap` lookup | HashMap get + key comparison | ✓ WIRED | `mods.is_empty()` check for NONE modifier; `contains()` for non-empty mods |
| `keymap_warnings` non-empty | status bar `⚠` indicator | `render_status_bar` conditional push_str | ✓ WIRED | `"\| ⚠ keymap: N warning(s) ('!' for details)"` added when non-empty |
| `'!'` in `handle_normal_key` | `AppMode::KeymapErrors` | Direct `KeyCode::Char('!')` arm (non-overridable) | ✓ WIRED | Guard: `!keymap_warnings.is_empty()` before mode change |
| `'?'` via `key_is_action("help")` | `AppMode::Help` → `render_help_overlay` | `effective_keymap` bindings table | ✓ WIRED | 5 sections; `chord_description` helper formats each binding |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| `render_help_overlay` | `effective_keymap` (HashMap) | `App::new` → `resolve_keymap` → user config | Yes — driven by actual config.toml [keymap] entries or defaults | ✓ FLOWING |
| `render_status_bar` | `keymap_warnings` (Vec<String>) | `resolve_keymap` conflict/invalid-chord detection | Yes — driven by actual parse failures and conflict detection | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All TUI tests | `cargo test -p todotxt-tui` | `test result: ok. 58 passed; 0 failed` | ✓ PASS |
| Keymap/config unit tests (10) | subset of above | All 10 config/keymap tests pass | ✓ PASS |
| Full workspace | `cargo test --workspace` | 0 failures across all crates | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| PAR-01 | 22-01-PLAN.md | Hotkeys aligned with todotxt.net orientation | ✓ SATISFIED | 19 default bindings in `default_keymap()` match todotxt.net hotkey orientation; all overridable |
| PAR-02 | 22-03-PLAN.md | Help overlay shows all active bindings | ✓ SATISFIED | `render_help_overlay` reads `effective_keymap` for all 19 configurable bindings; `?` opens it |
| PAR-03 | 22-03-PLAN.md | Deviations from todotxt.net documented | ✓ SATISFIED | DEVIATION.md with DEV-01–07 present at `.planning/phases/22-keymap-help-parity/DEVIATION.md` |
| KEY-01 | 22-01-PLAN.md | User can configure key bindings via config.toml | ✓ SATISFIED | `TuiConfig.keymap` field; `resolve_keymap` applies overrides; TOML round-trip tested |
| KEY-02 | 22-02-PLAN.md | Invalid/conflicting bindings surface warnings | ✓ SATISFIED | `parse_key_chord` returns None for invalid; conflict detection reverts both; `keymap_warnings` populated |
| KEY-03 | 22-01-PLAN.md | All 19 overridable actions have sensible defaults | ✓ SATISFIED | `default_keymap()` returns 19 entries; defaults restored on parse failure or conflict |

REQUIREMENTS.md confirms PAR-01–03 and KEY-01–03 scope delivered by Phase 22 plans.

### Human Verification Required

#### 1. Help overlay renders correctly in live TUI

**Test:** Run TUI, press `?`.
**Expected:** Centered popup with title "Keybindings — Esc/q: close"; 5 sections (Tasks, Filter, View, Select, App) listing all 19 configurable bindings; Esc or q closes it.
**Why human:** `render_help_overlay` visual output requires live TUI inspection.

#### 2. Keymap warning overlay shows on '!'

**Test:** Add `[keymap]` with an invalid entry (e.g., `move_down = "ctrl+invalid_key"`) to config.toml, start TUI.
**Expected:** Status bar shows `⚠ keymap: 1 warning(s) ('!' for details)`. Pressing `!` opens the errors overlay listing the invalid chord.
**Why human:** Requires a live TUI with a deliberate bad config entry and visual inspection.

#### 3. User-overridden key appears in help overlay

**Test:** Set `[keymap] move_down = "j"` (same as default) in config.toml, start TUI, press `?`.
**Expected:** Help overlay shows `j` for move_down — accepts the override even when it matches the default.
**Why human:** Requires config.toml modification + TUI restart + visual inspection.

#### 4. Conflict detection reverts both conflicting bindings

**Test:** Set two actions to the same key in [keymap] (e.g., `delete = "x"`, `toggle_done = "x"`), start TUI.
**Expected:** Both actions fall back to their defaults; warning overlay on `!` shows two conflict entries.
**Why human:** Requires deliberate config conflict + live TUI verification.

### Gaps Summary

No blocking gaps. All 6 observable truths verified against the codebase. Four human verification items cover visual TUI behaviors and config interactions that require an interactive session.

---

_Verified: 2026-04-28T00:00:00Z_

# Phase 22 — Deviation Log

Phase: `22-keymap-help-parity`  
Branch: `gsd/v1.3-scope`

Deliberate behavioral differences from the original todotxt.net WPF application, or gaps
closed by this phase, are documented below.

---

## DEV-01 — Keymap is fully configurable via TOML

**Original behavior:** todotxt.net hardcodes all keyboard shortcuts; they cannot be changed without recompiling.

**TUI behavior:** All 19 primary actions are configurable via `[keymap]` in the TOML config file. Users can remap any action to any key chord. The hardcoded navigation keys (j/k, Ctrl+D/U, Shift+j/k, Ctrl+C) are deliberately excluded from the keymap to ensure basic navigation is always available.

**Rationale:** Terminal users expect vi-style configurability. Keeping nav keys hardcoded prevents an accidentally inaccessible UI.

---

## DEV-02 — Conflict detection with automatic revert

**Original behavior:** No conflict detection; no concept of configurable keymaps.

**TUI behavior:** If two actions are configured to the same key chord, both are reverted to their defaults and a warning is emitted to the status bar. The user is notified via `⚠ keymap: N warning(s)` in the status bar and can press `!` to see the full warning list.

**Rationale:** Silent conflicts would cause one action to shadow another invisibly. Explicit revert-to-default is the safest UX.

---

## DEV-03 — Help overlay (`?`)

**Original behavior:** todotxt.net shows a separate Help window (F1 or from menu).

**TUI behavior:** `?` opens an in-app overlay listing all 19 configurable bindings plus the hardcoded navigation keys. The overlay reads from `effective_keymap`, so it always reflects the user's actual resolved bindings (including any overrides). Dismissed with `Esc` or `q`.

**Rationale:** Terminal apps should be self-documenting. Showing the user's actual configured bindings is more useful than a static help screen.

---

## DEV-04 — Clear filter hotkey (`0`)

**Original behavior:** todotxt.net clears the active filter via a toolbar button or Ctrl+0.

**TUI behavior:** `0` (zero) clears the filter query and active preset, immediately showing all tasks. Configurable via `[keymap] clear_filter = "..."`.

**Rationale:** Single-key filter clear is consistent with terminal workflow; avoids needing the filter panel for a common operation.

---

## DEV-05 — Preset filter hotkeys (`1`–`9`)

**Original behavior:** todotxt.net supports numbered filter presets via a dropdown.

**TUI behavior:** `1`–`9` directly applies the corresponding preset slot (`f1`–`f9`) from the config. These keys are NOT overridable via `[keymap]` — they are hardcoded to the digit row to maintain predictability of slot access. If a slot has no preset configured, the key is a no-op.

**Rationale:** Direct slot access is faster than navigating a menu. Making them non-overridable prevents accidentally removing preset access entirely.

---

## DEV-06 — Reload hotkey (`.`)

**Original behavior:** todotxt.net watches the file for changes and auto-reloads. No manual reload key.

**TUI behavior:** `.` (period) forces an immediate reload of the task file from disk, clearing any queued `pending_reload` flag. Uses `task_list.reload()` which preserves the same path. Configurable via `[keymap] reload = "..."`. On error, a message is printed to stderr (the TUI continues running).

**Rationale:** Provides explicit user control when automatic file-watch events are delayed or missed, e.g. when editing the file on a remote filesystem.

---

## DEV-07 — KeymapErrors overlay (`!`)

**Original behavior:** No equivalent — no configurable keymaps.

**TUI behavior:** If startup produced any keymap warnings (unknown action names, invalid chords, or conflicts), the status bar shows `⚠ keymap: N warning(s) ('!' for details)`. Pressing `!` opens a read-only overlay listing all warnings. This key is hardcoded (not configurable) so it is always reachable regardless of user config.

**Rationale:** Silent misconfiguration is confusing. Surfacing warnings visually with a drill-down overlay gives users actionable feedback without interrupting startup.

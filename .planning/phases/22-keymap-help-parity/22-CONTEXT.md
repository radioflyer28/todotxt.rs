# Phase 22: Keymap + Help Parity — Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Align the TUI's key surface with todotxt.net where practical, make bindings
discoverable in-app, and allow users to override bindings in `config.toml`
without recompiling.

Phase 22 delivers:
1. **`[keymap]` config support** — `TuiConfig` gains a `[keymap]` TOML table
   mapping action names → human-readable key chord strings (e.g. `delete = "backspace"`)
2. **Keymap resolution at startup** — invalid/duplicate entries are collected as
   warnings; defaults are used for bad entries; a status-bar notice appears
3. **`'!'` keymap-error overlay** — pressing `'!'` from Normal mode (when warnings
   exist) opens a dismissable panel listing all keymap errors
4. **`'?'` help overlay** — pressing `'?'` opens a full-screen panel showing all
   resolved (effective) key bindings; `Esc`/`q` closes it; `AppMode::Help` added
5. **New parity hotkeys** — `'?'` help, `'0'` clear filter, `'1'`–`'9'` preset recall,
   `'.'` manual reload wired in Normal mode
6. **Deliberate deviations documented** — PAR-03 fulfilled via milestone DEVIATION.md
   artifact plus the help overlay itself showing what's implemented

Phase 23 owns final UAT and close-out. `'I'` (priority), `'P'` (postpone), and
`'A'` (archive) are **out of scope for Phase 22** — deferred to Phase 23+ or v1.4.

</domain>

<decisions>
## Implementation Decisions

### Keymap Config Schema (KEY-01)

- **D-01:** Overridable actions are the ~15 actions in `handle_normal_key`: `add`,
  `edit`, `delete`, `bulk_delete`, `bulk_append`, `toggle_done`, `filter_open`,
  `filter_define`, `filter_toggle`, `sort_cycle`, `group_toggle`, `deferred_toggle`,
  `theme_cycle`, `disjoint_select`, `disjoint_mark`, `clear_filter`, `reload`,
  `help`, `quit`. These are all the meaningful single-action bindings — mode-internal
  keys (e.g. `y`/`n` in DeleteConfirm) are NOT overridable.
- **D-02:** The table lives as a flat `[keymap]` section in `config.toml`:
  ```toml
  [keymap]
  delete = "backspace"
  toggle_done = "space"
  reload = "f5"
  ```
  The TOML key is the action name; the value is a human-readable chord string.
- **D-03:** Key chord format is **human-readable, case-insensitive strings**:
  - Modifiers: `ctrl+`, `shift+`, `alt+` (order: modifiers first, key last)
  - Bare chars: `"n"`, `"?"`, `"0"`
  - Special keys: `"enter"`, `"esc"`, `"backspace"`, `"delete"`, `"up"`, `"down"`,
    `"left"`, `"right"`, `"f1"`–`"f12"`, `"space"`, `"tab"`
  - Examples: `"ctrl+d"`, `"shift+n"`, `"f5"`, `"?"`, `"alt+up"`
- **D-04:** `TuiConfig` gains `keymap: HashMap<String, String>` field
  (keyed on action name, value is chord string). Deserialized from `[keymap]`.
  Default is an empty map (all defaults apply).
- **D-05:** At startup, `App::new` calls a `resolve_keymap(config)` function that
  parses each entry and returns `(resolved_bindings, warnings)`. The resolved
  bindings replace/extend the default bindings for dispatch. Warnings are stored
  in `App` and surfaced in the status bar.

### Invalid/Conflicting Key Fallback (KEY-02)

- **D-06:** **Warn, use default, continue.** Invalid entries (unrecognized chord
  string, unrecognized action name) are skipped — the default binding is used for
  that action. No crash, no silent ignore.
- **D-07:** Conflict detection: if two actions are configured to the same key chord,
  both are added to the warnings list and both fall back to their defaults. The
  conflict is surfaced explicitly in the `'!'` overlay.
- **D-08:** When `warnings.len() > 0`, the status bar left segment appends
  `| ⚠ keymap: N warnings ('!' for details)` after the task count. When no warnings,
  nothing is shown (no noise for clean configs).
- **D-09:** The `'!'` key in Normal mode (only when `!self.keymap_warnings.is_empty()`)
  opens a dismissable overlay panel (`AppMode::KeymapErrors`) showing all warnings,
  one per line. `Esc` closes it. This is a read-only panel — no input needed.
- **D-10:** `App.keymap_warnings: Vec<String>` — populated once at startup, never
  mutated during the session. The config file is not watched for keymap changes
  (restart required).

### KEY-03: Default Bindings Stay todotxt.net-Oriented

- **D-11:** Default bindings (when no `[keymap]` override exists) remain unchanged
  from the Phases 19–21 implementations. No existing key is reassigned by Phase 22.
  All new keys (`?`, `0`, `1`–`9`, `.`) are currently unbound — adding them is
  additive, not breaking.

### Help Overlay (PAR-02)

- **D-12:** `'?'` from Normal mode opens `AppMode::Help`. This is a full-screen panel
  rendered by a new `render_help_overlay()` function. `Esc` or `q` closes it.
- **D-13:** The help panel displays the **resolved (effective) key bindings** — if the
  user overrode `delete = "backspace"`, the panel shows `backspace  delete task`,
  not `d  delete task`. This makes the panel authoritative for the user's actual config.
- **D-14:** Layout: two-column table. Left column = key chord, right column = description.
  Organized into sections: Navigation, Actions, Bulk Actions, Filter/Sort, View, Other.
  Uses existing `theme.rs` styles for consistent styling.
- **D-15:** The help overlay does NOT show deliberate deviation notes inline — that
  level of detail belongs in the DEVIATION.md artifact (PAR-03), not the overlay.
  The overlay is for quick reference, not documentation.

### New Parity Hotkeys (PAR-01)

- **D-16:** `'?'` → open help overlay (`AppMode::Help`). todotxt.net parity.
- **D-17:** `'0'` → clear the active filter (`self.filter_query.clear()` +
  `self.toggled_filter_query = None` + `rebuild_and_reanchor()`). todotxt.net `0`.
- **D-18:** `'1'`–`'9'` → apply the preset in slot `f1`–`f9` respectively
  (i.e., `"1"` applies preset `"f1"`). If the slot is empty, no-op. todotxt.net parity.
- **D-19:** `'.'` → manual reload from disk (equivalent to what `file_watcher`
  fires automatically). Calls the existing reload path. todotxt.net `.`/`F5` parity.
- **D-20:** `'A'`, `'I'`, `'P'` are explicitly **not added in Phase 22** — they
  require new functionality (priority picker, postpone dialog, archive command) that
  belongs in Phase 23+ or v1.4.

### Deviation Documentation (PAR-03)

- **D-21:** A `DEVIATION.md` artifact is produced at the end of Phase 22 execution
  (or at latest Phase 23) listing every deliberate todotxt.net deviation. Minimum
  entries: vim-style navigation (`j/k` vs `J/K`), lowercase-default action keys
  vs todotxt.net uppercase, selection model (`v`/`Shift+nav` vs no equivalent),
  `'I'`/`'P'`/`'A'` absent. Location: `.planning/phases/22-keymap-help-parity/DEVIATION.md`.

### Agent's Discretion

- Whether `resolve_keymap()` is a free function in `config.rs` or a method on
  `TuiConfig` — planner decides based on what's cleaner given module boundaries.
- Whether the resolved bindings are stored as `HashMap<ActionName, KeyChord>` or a
  typed struct — planner decides. The runtime dispatch must be able to look up the
  current binding for an action efficiently (used in both key handling and help rendering).
- Whether `AppMode::KeymapErrors` shares the rendering infrastructure with
  `AppMode::Help` (both are read-only overlays) or gets its own small render function.
- Exact two-column layout widths and wrapping for the help overlay — planner decides
  based on minimum terminal width assumption (80 cols safe baseline).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary integration file
- `crates/todotxt-tui/src/app.rs` — `App` struct, `AppMode` enum, `handle_normal_key()`,
  `render_status_bar()`, `App::new()` — keymap resolution and dispatch go here

### Config
- `crates/todotxt-tui/src/config.rs` — `TuiConfig`, `TuiSection` — `[keymap]` table
  and `keymap_warnings` storage added here

### Theme/styling
- `crates/todotxt-tui/src/theme.rs` — `StyleSheet`, `Theme` — use existing style
  conventions for overlay panels

### Reference app keyboard surface
- `Client/Resource.resx` (line 194+) — `HelpText` string — canonical list of all
  todotxt.net shortcuts used for parity analysis

### Prior phase contracts (must not be broken)
- `.planning/phases/20-bulk-actions-selection-ux/20-CONTEXT.md` — D-01 through D-14
  (bulk `D`/`T` keys, selection state)
- `.planning/phases/19-selection-model-multi-select-foundation/19-CONTEXT.md` — D-01
  through D-20 (selection model, Shift+nav, `v` key, Esc behavior)

### Requirements
- `.planning/REQUIREMENTS.md` — PAR-01 (hotkey parity), PAR-02 (discoverability),
  PAR-03 (deviation docs), KEY-01 (config overrides), KEY-02 (safe fallback),
  KEY-03 (defaults todotxt.net-oriented)

</canonical_refs>

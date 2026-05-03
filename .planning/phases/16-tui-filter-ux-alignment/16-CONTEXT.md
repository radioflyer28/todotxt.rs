# Phase 16: TUI Filter UX Alignment - Context

**Gathered:** 2026-04-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 16 reworks the TUI filter experience in three areas:
1. **Esc semantics** — Esc in the quick filter panel restores the prior filter state (cancel/restore), replacing the current clear-and-close behavior
2. **Filter panel split** — Separate the quick filter (`f`) from preset definition (`F`/Shift+f) into two distinct panels with clear responsibilities
3. **Preset persistence** — Preset definitions written to TOML on confirm-close; reloaded reliably on startup

Requirements covered: V12-TUI-FILTER-01, V12-TUI-FILTER-02, V12-TUI-FILTER-03.

</domain>

<decisions>
## Implementation Decisions

### Filter Panel Architecture

- **D-01: Two separate panels, two keys** — `f` opens the quick filter panel (current behavior, with Esc semantics changed). `F` (Shift+f) opens the preset definition panel. Clean separation — quick filter and preset definition are distinct workflows, not modes within one panel.

### Esc Semantics

- **D-02: Esc in quick filter = cancel/restore** — When the user opens the quick filter panel (`f`), the active filter query at open time is captured as a snapshot. Esc discards any edits and restores that snapshot, closing the panel. Task list snaps back to the pre-open state. This replaces Phase 12's D-05 (clear-and-close behavior).

- **D-03: Esc in definition panel = cancel** — Esc in the preset definition panel (`F`) closes the panel and discards all edits. Presets revert to their state when the panel was opened. Nothing is written to TOML.

### Preset Persistence

- **D-04: Save trigger = confirm close (Enter/OK only)** — Preset edits are written to TOML only when the user confirms close (Enter or a dedicated confirm key). Esc always discards without saving.

- **D-05: Preset definitions only are persisted** — Only the filter string for each preset (#1–#9) is written to TOML. The active filter is transient — starts empty on next launch. Sort order is not persisted.

### Preset Definition Panel Layout

- **D-06: Active filter + numbered preset list (C# layout, TUI-adapted)** — The definition panel shows:
  - Top row: currently active filter — **editable with live preview** (filter applies immediately as user types, same as the quick filter panel)
  - Below: numbered preset rows #1–#9, each selectable with Up/Down or 1–9 keys; selected row becomes an editable text field
  - Confirm with Enter to save preset definitions to TOML; Esc to cancel

- **D-07: Active filter row in definition panel is editable** — Changes to the active filter row apply live to the task list while the definition panel is open. This gives the user a live preview of the filter while also defining presets. On Esc, the active filter is also restored (same cancel/restore logic as quick filter panel).

### the agent's Discretion

- The key for the definition panel is `F` (Shift+f) — agent may use a different available key if `F` conflicts with existing bindings.
- Panel height for the definition panel — 1 active filter row + up to 9 preset rows, scrollable if terminal is small.
- Status bar hint updates to advertise `F` key for preset definition.
- Whether Enter in the quick filter panel (non-definition mode) closes the panel keeping filter active or not — agent decides; Phase 12 allowed this as a convenience.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### TUI Implementation
- `crates/todotxt-tui/src/app.rs` — `App`, `AppMode`, `FilteringState`, `handle_filtering_key()`, `render_filter_panel()` — all filter panel logic lives here
- `crates/todotxt-tui/src/config.rs` — `TuiConfig`, `TuiPreset`, `TuiSection` — TOML persistence target; `TuiPreset { filter: Option<String> }` already exists

### Core Filter Engine
- `crates/todotxt-core/src/filter.rs` — `Filter::from_query()` — unchanged, quick and definition panels both reuse this

### Requirements
- `.planning/REQUIREMENTS.md` — V12-TUI-FILTER-01 (Esc cancel/restore), V12-TUI-FILTER-02 (layout alignment), V12-TUI-FILTER-03 (persist to TOML)

### C# Reference Implementation
- `Client/Controls/FilterDialog.xaml` — layout reference: active filter at top, presets #1–#9 below, Cancel/OK buttons
- `Client/Controls/FilterDialog.xaml.cs` — behavioral reference: modal dialog with explicit OK/Cancel, no auto-save

### Prior Phase Context
- `.planning/phases/12-filter-sort/12-CONTEXT.md` — D-02 (panel contents), D-03 (filter syntax), D-05 (Esc behavior being overridden), D-06 (preset navigation), D-14 (TuiPreset struct) — Phase 16 overrides D-05 only; all other Phase 12 filter decisions remain in effect

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `FilteringState { editor: TextArea, selected_preset: usize }` — already exists; extend or fork for definition panel state
- `AppMode::Filtering` — already exists for quick filter; add `AppMode::FilterDefining` for definition panel
- `TuiPreset { filter: Option<String> }` in `TuiConfig` — TOML target already deserialized; Phase 16 adds serialization/write-back

### Established Patterns
- `TextArea` from `tui-textarea` — used for edit and filter input; reuse for active filter row and preset rows in definition panel
- Bottom panel rendering (Phase 11 delete-confirm, Phase 12 filter panel) — same `Layout::split` pattern for definition panel
- `owo-colors` + `StyleSheet` (Phase 13) — for styled panel border/header

### Integration Points
- `handle_normal_key()` — add `'F'` (or chosen key) to open definition panel → `AppMode::FilterDefining`
- `handle_filtering_key()` — add filter snapshot capture on panel open; change Esc to restore snapshot
- `draw()` — route `AppMode::FilterDefining` to `render_filter_definition_panel()`
- `TuiConfig::load()` — already reads presets; add serialization path to write back on confirm

</code_context>

<specifics>
## Specific Ideas

- The C# app uses a WPF modal dialog (blocking, OK/Cancel). The TUI adaptation is an inline bottom panel with the same two-zone layout (active filter + preset list) but non-modal — the task list behind it stays live-filtered.
- "Cancel/restore" for Esc means capturing `filter_query` at the moment `f` or `F` is pressed, then restoring that string on Esc (not just closing the panel).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 16-tui-filter-ux-alignment*
*Context gathered: 2026-04-23*

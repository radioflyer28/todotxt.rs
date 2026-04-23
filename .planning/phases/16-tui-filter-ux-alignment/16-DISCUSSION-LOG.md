# Phase 16: TUI Filter UX Alignment - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 16-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-23
**Areas discussed:** Filter panel modes, Esc semantics, Preset persistence, Filter definition layout

---

## Filter Panel Modes

**Q:** How should the two filter concerns (quick filtering vs defining presets) be exposed?

**Options presented:**
- Two separate panels, two keys *(selected)*
- One panel, two modes
- Replace quick filter with unified panel

**Answer:** Two separate panels, two keys — `f` = quick filter, `F` = preset definition.

---

## Esc Semantics — Quick Filter Panel

**Q:** Esc in the quick filter panel — what should it do?

**Options presented:**
- Cancel/restore — discard edits, restore prior filter *(selected)*
- Close panel, keep current filter
- Clear and close (current behavior)

**Answer:** Cancel/restore — capture snapshot on open, restore on Esc.

---

## Esc Semantics — Definition Panel

**Q:** Esc in the preset definition panel — what should it do?

**Options presented:**
- Cancel — discard all unsaved preset edits *(selected)*
- Save and close (treat Esc as confirm)
- Partial save — current row only

**Answer:** Cancel — discard all edits, nothing written to TOML.

---

## Preset Persistence — Save Trigger

**Q:** When should preset edits be written to TOML?

**Options presented:**
- Save on confirm close (Enter/OK only) *(selected)*
- Auto-save on every edit
- Explicit save key (Ctrl+S)

**Answer:** Save on confirm close (Enter/OK only).

---

## Preset Persistence — What Gets Saved

**Q:** What exactly gets persisted to TOML?

**Options presented:**
- Preset definitions only (active filter is transient) *(selected)*
- Preset definitions + last active filter
- Full filter state (presets + active filter + sort order)

**Answer:** Preset definitions only. Active filter and sort order are transient.

---

## Filter Definition Layout — Panel Shape

**Q:** What should the preset definition panel look like?

**Options presented:**
- Active filter + numbered preset list (C# layout, TUI-adapted) *(selected)*
- Preset-only list (9 rows, navigator-focused)
- Active filter + non-empty presets only

**Answer:** Active filter at top + numbered preset list #1–#9 below, matching C# FilterDialog layout.

---

## Filter Definition Layout — Active Filter Row

**Q:** Is the active filter row in the definition panel editable or read-only?

**Options presented:**
- Active filter row is editable (live preview) *(selected)*
- Active filter row is read-only (display only)

**Answer:** Editable with live preview — changes apply immediately to task list while panel is open.

---

*End of discussion log*

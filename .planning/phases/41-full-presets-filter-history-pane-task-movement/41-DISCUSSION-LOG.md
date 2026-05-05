# Phase 41: Full Presets, Filter History, Pane Task Movement — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 41-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-05
**Areas covered:** Preset Schema Expansion, Filter History, Pane Move Edge Cases

---

## Area 1: Preset Schema Expansion

**Q: What should the extended TOML preset block look like?**
Options: Flat fields on TuiPreset / Flat fields + per-pane array / You decide
→ Selected: Flat fields + per-pane array

**Q: When a preset specifies 'pane count and layout' — does it add/remove panes or only reconfigure existing?**
Options: Pane count field + add/drop from right / No pane count change / You decide
→ Freeform: "1 but it should replicate the pane count, order, and config as defined by the preset"

**Q: What happens to sort/group/group_by state when a filter-only preset is applied?**
→ Freeform: "preset filters are separate from pane layout presets and consumable by pane presets"

**Q: How do 'full presets' and 'filter presets' relate in TOML and at runtime?**
Options: Unified same block / Separate new block type / Other
→ Selected: Other

**Freeform discussion:** User proposed organized segregation — `[presets.filter.N]` for filter presets and `[presets.panes.name]` for layout presets.

**Q: With the filter/panes namespace split, what keys trigger pane layout presets?**
Options: Keep 1-9 for filter + new keys / Unified 1-9 detect type / Migrate old format
→ Selected: Keep 1-9 for filter, new keys for pane presets

**Q: What key(s) should trigger [presets.panes.xyz] application?**
Options: F1-F9 / Shift+1-9 / Command palette / You decide
→ Freeform: "Ctrl+[0-9]; command palette as a separate feature"

**Q: What happens to existing [presets.f1] TOML blocks after this change?**
Options: Backward compat / Deprecation warning / Silently dropped
→ Selected: Silently dropped

---

## Area 2: Filter History

**Q: Is the filter history ring shared across all panes or per-pane?**
Options: Shared app-wide / Per-pane / You decide
→ Selected: Shared app-wide ring

**Q: How many entries should the ring hold?**
Options: 20 / 50 / Unlimited / You decide
→ Selected: 50 entries

**Q: When Ctrl+R is pressed, does it populate input or immediately apply?**
Options: Live replace + Enter to apply / Immediately apply / History browse mode
→ Freeform: "re-use the filter preset UI; typing in that free space should auto-invoke Ctrl+R-like functionality"

**Freeform discussion:** User clarified — history should surface as inline autocomplete suggestions while typing in the filter input. Ctrl+R within the filter UI cycles the ring. Existing preset list stays below.

**Q: Should history suggestions reuse AutocompleteState or appear separately?**
Options: Reuse existing autocomplete popup / Separate dropdown / You decide
→ Selected: Reuse existing autocomplete popup

---

## Area 3: Pane Move Edge Cases

**Q: With 3+ panes, what happens at the boundary?**
Options: Stop at boundary / Wrap around / You decide
→ Selected: Wrap around (last → first)

**Q: Mixed multi-selection — all-or-nothing or partial?**
→ Freeform: User clarified that any task can have the destination tag applied; the only
  constraint is when panes don't have single-token filters. Selection is always
  single-pane, so either all tasks move or none move.

**Q: Can a multi-selection span multiple panes?**
Options: Single-pane selection only / Cross-pane selection supported
→ Selected: Single-pane selection only

**Q: After successfully moving task(s), where does focus land?**
Options: Cursor stays at same index / Focus jumps to destination / You decide
→ Selected: Focus jumps to destination pane after move

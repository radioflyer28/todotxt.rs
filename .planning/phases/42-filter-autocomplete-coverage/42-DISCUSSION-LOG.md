# Phase 42: Filter Autocomplete Coverage — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 42-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-06
**Phase:** 42 — Filter Autocomplete Coverage
**Requirements:** AC-02, AC-03, AC-04

---

## Gray Areas Discussed

All three gray areas were selected by the user.

---

### Area 1: FilterHistory vs TokenAutocomplete Priority

**Question asked:**
> When the user types @ or + in the filter input, what should happen to the FilterHistory popup that may already be visible?

**Options presented:**
1. Replace it — TokenAutocomplete takes over the autocomplete slot (self.autocomplete)
2. Suppress history when typing a tag token
3. You decide

**User selected:** Option 1 — Replace it

**Recorded as D-01:** TokenAutocomplete replaces FilterHistory. The single `self.autocomplete` slot switches modes — whichever is set last wins.

---

### Area 2: Post-Acceptance Behavior

**Question asked:**
> After the user selects a suggestion from the popup while in the filter input, what happens?

**Options presented:**
1. Insert and keep editing (stay in Filtering mode) — recommended
2. Insert and apply immediately (close panel)
3. You decide

**User selected:** Option 1 — Insert and keep editing

**Recorded as D-02:** Accept inserts the token and keeps the filter panel open. Filter applies on Enter as usual. Enables compound expression building.

---

### Area 3: Multi-Token Mid-Expression Triggering

**Question asked:**
> Should autocomplete trigger on @ or + anywhere in the filter input (mid-expression), or only when it's the first character?

**Options presented:**
1. Cursor-aware: trigger on any @ or + token being typed — recommended
2. First-token only: only trigger at start of input
3. You decide

**User selected:** Option 1 — Cursor-aware

**Recorded as D-03:** Autocomplete triggers on any `@`/`+` token being typed at the cursor position. Completing inserts at cursor, replacing the typed prefix.

---

## Summary

Three decisions made, all clear for planning:
- `self.autocomplete` slot is replaced (not merged) when a tag trigger is detected
- Accept keeps filter panel open (no immediate apply)
- Cursor-aware triggering supports compound filter expressions

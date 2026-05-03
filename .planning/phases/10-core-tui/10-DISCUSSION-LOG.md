# Phase 10: Core TUI — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-19
**Phase:** 10 — Core TUI
**Status:** Complete

---

## Area 1: Task Row Display

**Q1: What does each task row show?**
- Options: Line number + raw text / Structured columns (priority | text | due)
- **Selected: Line number + raw text**
- Rationale: Simplest, most accurate, consistent with Phase 9 format. Phase 13 adds color.

**Q2: Line number position?**
- Options: N: raw text / raw text LN / Line number only in status bar
- **Selected: Line number then raw text ("N: raw")**

**Q3: Should completed tasks look different (pre-theming)?**
- Options: No differentiation / Modifier::DIM / Agent's discretion
- **Selected: Dim completed tasks (Modifier::DIM)**
- Rationale: Provides visual differentiation before Phase 13 colors; easy to override later.

---

## Area 2: Selection Visual

**Q1: How is the selected task highlighted?**
- Options: ratatui ListState + reversed highlight / "> " prefix marker / Bold only
- **Selected: ratatui ListState + reversed highlight**
- Rationale: Standard ratatui pattern, cleanest Phase 13 upgrade path.

**Q2: Where does selection state live?**
- Options: selection index on App / ListState as App field / Agent's discretion
- **Selected: selection index on App, ListState created in draw()**
- Rationale: Simpler, easier to test in isolation.

---

## Area 3: Done/Undo Write Strategy

**Q1: When is the change written to disk?**
- Options: Immediate save / Dirty-flag write buffer
- **Selected: Immediate save on every x press**
- Rationale: Simple, always in sync, no risk of data loss on crash.

**Q2: How to handle own-write FileChanged events?**
- Options: Skip-next-reload flag / Ignore — harmless reload
- **Selected: Ignore for now — harmless reload**
- Rationale: Local files reload to same content instantly. Phase 13's TUI-UX-02 debounce handles suppression properly.

**Q3: How does x + u interact?**
- Options: x toggles both ways / x marks done, u undoes (separate keys)
- **Selected: x toggles both ways (done → undone → done)**
- Rationale: Single key toggle is simpler UX. u is still supported as alias per TUI-ACT-02.

---

## Area 4: Status Bar Layout

**Q1: How is the status bar positioned?**
- Options: Layout split / Overlay widget
- **Selected: Layout split: list area / 1-row footer**
- Rationale: Idiomatic ratatui pattern, clean separation.

**Q2: What does the status bar content show?**
- Options: Total/done/due-today/overdue / File path + counts / Selected line + counts
- **Selected: File path + counts**
- Rationale: File path helps confirm which todo.txt is loaded; counts satisfy TUI-UX-01.

**Q3: Should Phase 10 show keybind hints in the status bar?**
- Options: No hints / Short hint string on the right / Agent's discretion
- **Selected: Short hint string on the right**
- Format: `q quit | x done | j/k nav`

**Q4: Should the overdue count stand out visually?**
- Options: Monochrome only / Bold overdue count
- **Selected: Monochrome only (Phase 13 adds color)**
- Rationale: Consistent with Phase 10 monochrome-first approach.

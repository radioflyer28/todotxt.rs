# Phase 17: TUI Grouping/Sorting Alignment + Status Polish - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 17-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-23
**Areas discussed:** Grouping visual presentation, Grouping + navigation behavior, Theme label visibility rule, Deferred toggle key binding

---

## Area: Grouping visual presentation

**Q: How should group boundary look in the task list?**
Options: Header rows (styled separator with group key label) / Blank line separators only / Inline first-task emphasis / Other
**Selected:** Header rows (styled separator with the group key label)

**Q: How should group header rows be styled?**
Options: Reversed style (same as selected-row treatment) / Bold text / Dim/subtle / Agent's discretion
**Selected:** Reversed style (same as selected-row treatment)

**Q: What text goes in the group header row?**
Options: Just the key value ('+myproject', '@work', 'none') / Key value + task count / Agent's discretion
**Selected:** Just the key value

**Q: How does grouping get activated — tied to sort order or separate toggle?**
Options: Auto-group when sort is Project or Context / Separate 'g' toggle key / Agent's discretion
**Selected:** Separate 'g' toggle key for grouping on/off

**Q: When 'g' enables grouping, what does it group by?**
Options: Group by active sort order (all sorts) / Only group when sort is Project or Context / Agent's discretion
**Selected:** Group by active sort order (works with all sorts)

**Q: Should the status bar indicate when grouping is on?**
Options: Show '| group: on' in status bar / No indicator / Agent's discretion
**Selected:** Show '| group: on' in status bar when active

---

## Area: Theme label visibility rule

**Q: When should '| theme: ...' label be hidden?**
Options: Hide when Theme::Default / Always show (current behavior) / Remove theme label entirely / Agent's discretion
**Selected:** Remove theme label entirely

---

## Area: Deferred toggle key binding

**Q: What key should toggle deferred task visibility?**
Options: 't' (threshold) / 'T' (Shift+t) / 'h' (hidden tasks toggle) / Agent's discretion
**Selected:** 'h' (hidden tasks toggle)

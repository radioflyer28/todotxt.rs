# Phase 50: Input Ergonomics - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-05-19
**Phase:** 50-input-ergonomics
**Areas discussed:** Date-target entry model, Recurring-field semantics, Week-jump keyboard behavior, Auto-select dialog contract

---

## Date-target entry model

| Option | Description | Selected |
|--------|-------------|----------|
| Keep separate commands, share one dialog underneath | Preserve separate commands/hotkeys but reuse one improved shared date dialog. | ✓ |
| One shared launcher with target selection | One entry point chooses which field to edit from inside the flow. | |
| Something else | User provides a different command model. | |

**User's choice:** Keep separate commands, share one dialog underneath
**Notes:** Preserve existing command muscle memory while consolidating picker behavior internally.

---

## Recurring-field semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Date fields only | Phase 50 covers real date-bearing fields only; `rec:` rule editing stays out of scope. | ✓ |
| Include recurrence-adjacent entry too | Date workflow also helps reach recurrence-adjacent metadata. | |
| Something else | User provides a different recurring-related behavior. | |

**User's choice:** Date fields only
**Notes:** The phase should not pretend recurrence-rule syntax is the same thing as date picking.

---

## Week-jump keyboard behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Always week-jump | `Left` and `Right` always move by 7 days while the picker has focus. | ✓ |
| Week-jump only in calendar navigation mode | Preserve normal caret movement during text editing. | |
| Something else | User provides a different `Left`/`Right` rule. | |

**User's choice:** Always week-jump
**Notes:** The picker is treated primarily as a date-selection surface for this phase.

---

## Auto-select dialog contract

| Option | Description | Selected |
|--------|-------------|----------|
| Prefer current token, otherwise best match, otherwise first suggestion | Continuity-first selection behavior. | ✓ |
| Always best typed match | Ignore current token unless it is also the best match. | |
| Always first suggestion | Use a simple fixed default. | |
| Something else | User provides a custom priority order. | |

**User's choice:** Prefer current token, otherwise best match, otherwise first suggestion
**Notes:** Auto-select should help the user continue editing rather than resetting them to an arbitrary starting point.

---

## the agent's Discretion

- Exact shared-dialog refactor strategy behind the preserved command model.
- Exact week-jump implementation details in the picker control.
- Exact matching heuristic for the "best current match" fallback.

## Deferred Ideas

- `rec:` rule editing UI.
- One shared date-target launcher.
- Replacing date picking with generic auto-select behavior.
- Broader input-system redesign outside the scoped dialogs.

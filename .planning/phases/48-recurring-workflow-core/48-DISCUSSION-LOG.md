# Phase 48: Recurring Workflow Core - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-05-18
**Phase:** 48-recurring-workflow-core
**Areas discussed:** Next-date anchoring, Completion prompt flow, Carry-forward fields, Multi-task and bulk behavior

---

## Next-date anchoring

| Option | Description | Selected |
|--------|-------------|----------|
| Todo.txt-style split | Strict `rec:+1d` anchors from the prior due date when one exists, while relative `rec:1d` anchors from the completion date. If there is no due date to anchor from, fall back to completion date. | ✓ |
| Always anchor from completion date | Both strict and relative modes use the date the user completed the task, even if the task already had a due date. | |
| Something else | User provides a custom anchoring rule. | |

**User's choice:** Todo.txt-style split
**Notes:** Strict recurrence keeps schedule cadence from the due date; relative recurrence slides from actual completion.

---

## Completion prompt flow

| Option | Description | Selected |
|--------|-------------|----------|
| Prompt only for recurring tasks | Normal tasks complete immediately. Recurring tasks ask whether to create the next occurrence, with a clear skip path. | |
| Prompt for every completion | Both recurring and non-recurring tasks show a completion confirmation so the UX stays uniform. | |
| Something else | User provides a custom completion interaction. | ✓ |

**User's choice:** No prompt - creating the next occurrence is obviously implicit.
**Notes:** This overrides the earlier milestone wording that said recurrence creation should be prompt-driven.

---

## Carry-forward fields

| Option | Description | Selected |
|--------|-------------|----------|
| Copy almost everything relevant | Keep description, priority, projects, contexts, `rec:` token, and other non-completion metadata. Clear completion state/dates, and recalculate the next due date. | ✓ |
| Copy only core task identity | Keep description plus project/context tags and `rec:` token, but drop optional metadata like priority, threshold date, and extra key:value metadata. | |
| Something else | User provides an explicit field-by-field carry-forward rule. | |

**User's choice:** Copy almost everything relevant
**Notes:** Recurrence should preserve task identity and metadata continuity while resetting completion-only state.

---

## Multi-task and bulk behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Apply recurrence for each recurring task automatically | Each recurring task creates exactly one next occurrence during the bulk operation, with no extra prompts. | ✓ |
| Skip recurrence in bulk flows | Single-task completion creates next occurrences, but bulk completion only marks tasks done to avoid surprise fan-out. | |
| Something else | User provides a custom rule for CLI multi-ID and TUI bulk completion. | |

**User's choice:** Apply recurrence for each recurring task automatically
**Notes:** Bulk completion should preserve the same recurrence contract as single-task completion rather than acting as an escape hatch.

---

## the agent's Discretion

- Exact recurrence helper placement and internal representation.
- Exact user-facing status message wording, as long as no prompt is introduced.

## Deferred Ideas

None.

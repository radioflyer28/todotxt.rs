# Phase 47: TUI Readability - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-15
**Phase:** 47-tui-readability
**Areas discussed:** Inactive-pane selection styling, Spacer row appearance, Cursor anchoring around spacer/header rows, Single-pane parity

---

## Inactive-pane selection styling

| Option | Description | Selected |
|--------|-------------|----------|
| No row emphasis at all | Keep the remembered selection index, but render inactive panes like plain lists with no highlighted row | ✓ |
| Subtle remembered-row cue | No cursor highlight, but keep a light remembered-row treatment such as dim/bold/recolored text | |
| Something else | Define a different inactive-pane presentation | |

**User's choice:** No row emphasis at all
**Notes:** The user wants inactive panes to remember selection state but not display any visible row emphasis.

---

## Spacer row appearance

| Option | Description | Selected |
|--------|-------------|----------|
| Truly blank row | Insert a plain blank spacer row before each non-first group header | ✓ |
| Styled empty separator row | Insert an empty-but-styled separator-like row to make group boundaries more explicit | |
| Something else | Define a different spacer appearance | |

**User's choice:** Truly blank row
**Notes:** The spacer should provide breathing room only, not act like a visual divider.

---

## Cursor anchoring around spacer/header rows

| Option | Description | Selected |
|--------|-------------|----------|
| Always anchor to a task row | Stored selection may move as needed so the cursor never lands on headers or spacers | ✓ |
| Preserve raw row position when possible | Try to keep the same row index even if extra correction is needed later | |
| Something else | Define a different anchoring rule | |

**User's choice:** Always anchor to a task row
**Notes:** Headers and spacers are structural rows only and should never become cursor targets.

---

## Single-pane parity

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, identical behavior | Single-pane grouped mode uses the same spacer and non-selectable structure-row rules as multi-pane mode | ✓ |
| Mostly same, but denser single-pane rendering | Keep structure-row rules but reduce spacing in single-pane mode | |
| Something else | Define an intentional layout-mode difference | |

**User's choice:** Yes, identical behavior
**Notes:** The user wants one consistent grouped-row model across layout modes.

---

## the agent's Discretion

- Internal row representation for spacer rows.
- Exact regression-test organization and naming.

## Deferred Ideas

None — discussion stayed within phase scope.

# Roadmap: todotxt.net — Rust Port

## Current Milestone: v1.6.3 TUI UX tweaks, filter OR operator, recurring tasks, done.txt rotation

### Milestones

- ✅ v1.0 Core Library + CLI — shipped 2026-04-16
- ✅ v1.1 TUI Interface — shipped 2026-04-23
- ✅ v1.2 Compatibility + UX Alignment — shipped 2026-04-24
- ✅ v1.3 Feature/Hotkey Parity with todotxt.net — shipped 2026-04-28
- ✅ v1.4 Kanban-Style Vertical Panes — shipped 2026-04-29
- ✅ v1.5 Capture Flow + Bulk Safety + Clipboard + Undo — shipped 2026-05-01
- ✅ v1.6 TUI Fixes and Power User Improvements — shipped 2026-05-06
- 🔧 v1.6.3 TUI UX + Filter + Recurring + Archive hygiene — in progress

---

## v1.6.3 Phases

Phase numbering continues after v1.6 (45).

| # | Phase | Name | Requirements | Success Criteria |
|---|-------|------|--------------|------------------|
| 46 | Filter Engine Upgrade | Add OR logic to filter parsing and evaluation | FILT-01, FILT-02, FILT-03 | 5 |
| 47 | TUI Readability | Improve pane focus rendering and grouped list spacing | TUI-01, TUI-02 | 4 |
| 48 | Recurring Workflow Core | Implement recurring completion model and implicit next-occurrence generation | REC-01, REC-02, REC-03, REC-04 | 6 |
| 49 | Archive Hygiene | Add time-based done.txt rotation with monthly cadence and deterministic period naming | DONE-01, DONE-02, DONE-03 | 4 |
| 50 | Input Ergonomics | Expand date setter workflows and refine match-dialog auto-select interaction | DATE-UX-01, DATE-UX-02, AUTO-SEL-01, AUTO-SEL-02 | 4 |

### Phase 46: Filter Engine Upgrade

- **Goal:** Add OR-capable filter terms without regressing existing AND semantics.
- **Requirements:** FILT-01, FILT-02, FILT-03
- **Plans:** 2 plans in 2 waves
- **Wave 1:** `46-01` — Core parser/evaluator OR support
- **Wave 2:** `46-02` — CLI help contract and integration coverage *(blocked on Wave 1 completion)*
- **Success criteria**
  1. `@work|@home` matches either context in filter evaluation.
  2. `(@A|B)` and negated OR examples are documented and tested.
  3. Existing AND-only filters continue to behave unchanged in CLI and TUI.
  4. Unit test coverage exists for parser and evaluator edge cases.
  5. No panic on malformed OR inputs.

### Phase 47: TUI Readability

- **Goal:** Make grouped and multi-pane views easier to parse visually and easier to navigate.
- **Requirements:** TUI-01, TUI-02
- **Status:** Complete — 2026-05-15
- **Plans:** 2 plans in 2 waves
- **Wave 1:** `47-01` — Active-only pane cursor highlight
- **Wave 2:** `47-02` — Group spacer rows and task-only navigation *(blocked on Wave 1 completion)*
- **Success criteria**
  1. Inactive panes preserve selection state but render no cursor highlight.
  2. Non-first group headers render with a spacer row beforehand.
  3. Navigation skips spacer rows and group headers and still lands on valid task rows.
  4. User-facing behavior is consistent across single- and multi-pane modes.

### Phase 48: Recurring Workflow Core

- **Goal:** Add recurring task completion with implicit next-occurrence generation and reusable behavior across CLI and TUI.
- **Requirements:** REC-01, REC-02, REC-03, REC-04
- **Status:** Complete — 2026-05-18
- **Plans:** 3 plans in 2 waves
- **Wave 1:** `48-01` — Core recurrence parser and next occurrence construction
- **Wave 2:** `48-02` — CLI recurring completion integration *(blocked on Wave 1 completion)*
- **Wave 2:** `48-03` — TUI recurring completion integration *(blocked on Wave 1 completion)*
- **Success criteria**
  1. `rec:` token is preserved and interpreted consistently.
  2. Completing a recurring task automatically creates the next occurrence without prompting.
  3. Both CLI and TUI completion paths produce equivalent recurring behavior.
  4. Repeated completion of recurring tasks creates exactly one next task per completion.
  5. Regression cases confirm required fields and metadata are preserved.
  6. Completed behavior documented for strict vs relative recurrence modes.

### Phase 49: Archive Hygiene

- **Goal:** Add time-based done.txt rotation with monthly cadence, future-ready cadence configuration, and explicit archive feedback.
- **Requirements:** DONE-01, DONE-02, DONE-03
- **Status:** Complete — 2026-05-19
- **Plans:** 3 plans in 2 waves
- **Wave 1:** `49-01` — Shared cadence/config contract and rotation helper
- **Wave 2:** `49-02` — CLI archive rotation integration *(blocked on Wave 1 completion)*
- **Wave 2:** `49-03` — TUI archive rotation integration *(blocked on Wave 1 completion)*
- **Success criteria**
  1. `done.txt` rotates automatically during archive writes when a new configured time period begins, starting with monthly cadence.
  2. Rotation moves prior archive contents into deterministic period files such as `done-2026-05.txt` and starts a fresh active `done.txt`.
  3. Cadence configuration uses monthly defaults and leaves room for future weekly-style extensions without implementing cleanup yet.
  4. Existing CLI/TUI archive workflows retain clear messaging, undo boundaries, and explicit feedback when rotation occurs.

### Phase 50: Input Ergonomics

- **Goal:** Make date entry faster and less repetitive while improving auto-select behavior in match-driven dialogs such as project and context token selection.
- **Requirements:** DATE-UX-01, DATE-UX-02, AUTO-SEL-01, AUTO-SEL-02
- **Status:** Completed
- **Plans:** 2 plans in 1 wave
- **Wave 1:** `50-01` — TUI date-target cycling and week-jump navigation
- **Wave 1:** `50-02` — TUI quick-setter auto-select continuity behavior
- **Success criteria**
  1. The TUI date picker can set due, threshold, and completed dates without forcing users through separate workflows.
  2. Left and right arrow navigation moves by week in the picker.
  3. Match-driven dialogs auto-select the most relevant existing project/context token or suggestion using a continuity-first default.
  4. Existing date-picker and token-editing workflows remain intuitive while the refinements are added.

## Coverage Check

- v1.6.3 requirements total: 16
- Mapped to phases: 16
- Unmapped: 0 ✓

## Backlog

- GUI interface (native desktop)
- CI/CD release pipeline and package distribution

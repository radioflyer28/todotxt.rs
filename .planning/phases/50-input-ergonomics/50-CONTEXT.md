# Phase 50: Input Ergonomics - Context

**Gathered:** 2026-05-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 50 improves two related but distinct interaction surfaces:

- the date-specific picker workflow used for editing date-bearing task fields
- generic match-driven dialogs that present project/context token suggestions

The scope is intentionally ergonomic rather than architectural. The phase should reduce
keystroke friction and improve default selection behavior without replacing the existing
command model or turning date entry into a different kind of control.

</domain>

<decisions>
## Implementation Decisions

### Date-target entry model
- **D-01:** Keep separate user-facing commands for date editing rather than replacing them
  with one unified launcher.
- **D-02:** These separate commands should reuse one improved shared date dialog
  underneath where possible.
- **D-03:** The existing command and menu muscle memory should be preserved, even if the
  internal implementation is consolidated.

### Date-field scope
- **D-04:** Phase 50 date work applies only to real date-bearing fields.
- **D-05:** Recurrence rule editing such as `rec:` syntax is out of scope for this phase's
  date-picker work.
- **D-06:** Planning should not blur the line between date selection and recurrence-rule
  editing just because both are recurrence-adjacent.

### Date picker navigation
- **D-07:** Inside the date picker, `Left` and `Right` should always jump by 7 days.
- **D-08:** This week-jump behavior is part of the picker contract for this phase, not an
  optional alternate mode.
- **D-09:** Existing confirm/cancel semantics should remain intact while the new navigation
  behavior is added.

### Match-dialog auto-select behavior
- **D-10:** Auto-select applies to generic match-driven dialogs, especially project/context
  token selection dialogs, not to date entry.
- **D-11:** When such a dialog opens, it should prefer the current existing token/value if
  one is already present.
- **D-12:** If no current token exists, selection should prefer the best current match.
- **D-13:** If neither a current token nor a better match exists, the first suggestion is
  the fallback.
- **D-14:** This behavior should optimize for continuity rather than arbitrary reset.

### Relationship to earlier milestone wording
- **D-15:** Earlier milestone notes used loose language around "recurring" and "auto-select"
  that could be misread as date-picker behavior. That is now superseded: dates stay on the
  date-specific picker, while auto-select belongs to generic match dialogs.

### the agent's Discretion
- Exact refactoring shape for consolidating shared date-dialog code, as long as separate
  commands remain intact.
- Exact control-level implementation details for week-jump navigation in the date picker.
- Exact matching heuristic used to determine the "best current match" for project/context
  dialogs, as long as the priority order in D-11 through D-13 is preserved.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and roadmap
- `.planning/ROADMAP.md` - Phase 50 goal, requirement mapping, and success criteria.
- `.planning/REQUIREMENTS.md` - `DATE-UX-01`, `DATE-UX-02`, `AUTO-SEL-01`, and
  `AUTO-SEL-02`.
- `.planning/PROJECT.md` - milestone-level input ergonomics intent.
- `.planning/STATE.md` - current workflow position after Phase 49 completion.

### Captured backlog intent
- `.planning/todos/pending/2026-05-15-expand-date-picker-targets-and-week-nav.md`
- `.planning/todos/pending/2026-05-15-refine-date-picker-auto-select.md`

### Current date-picker implementation
- `Client/Controls/SetDueDateDialog.xaml` - current date dialog UI surface.
- `Client/Controls/SetDueDateDialog.xaml.cs` - current picker key handling (`T`, `Up`,
  `Down`, `Enter`, `Escape`) and selected-date plumbing.
- `Client/MainWindowViewModel.cs` - current due/threshold date command flow and shared
  date-dialog helper methods.
- `Client/Controls/MainWindow.xaml` - current commands, key bindings, and menu wiring for
  due/threshold date actions.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ShowDateDialog(...)` in `Client/MainWindowViewModel.cs` already provides shared date
  dialog plumbing for multiple field types.
- `SetDueDateDialog` is already reusable in spirit, even though its naming and labels are
  still due-date-specific.
- Current due-date and threshold-date actions already demonstrate the separate-command,
  shared-dialog pattern that the phase should preserve and extend.

### Established Patterns
- The WPF client uses explicit routed commands and key bindings rather than one modal
  dispatcher for all task mutations.
- Date picker keyboard affordances are currently implemented in code-behind, so week-jump
  behavior will likely belong there too.
- The app already distinguishes field-specific commands from generic text/token editing
  flows, and Phase 50 should preserve that mental model.

### Integration Points
- `Client/MainWindowViewModel.cs` likely needs the main refactor to broaden date-dialog
  reuse across additional date-bearing fields.
- `Client/Controls/SetDueDateDialog.xaml` and `.xaml.cs` need the date-picker label and
  week-jump behavior updates.
- Match-dialog auto-select implementation may require a broader codebase search during
  research/planning because the current code scout did not reveal one obvious centralized
  auto-select dialog class by name.

</code_context>

<specifics>
## Specific Ideas

- Separate commands should remain first-class even if the date dialog becomes more generic.
- The date dialog should make the active field clear when reused for multiple date-bearing
  targets.
- `Left`/`Right` week jumps should be treated as a direct picker productivity improvement,
  not as an alternate mode the user has to discover.
- Project/context dialogs should feel like they "pick up where the current task already is"
  whenever possible.

</specifics>

<deferred>
## Deferred Ideas

- Recurrence-rule editing UI for `rec:` syntax.
- Collapsing all date operations into a single target-selection launcher.
- Replacing the date-specific picker with a generic auto-select or text-first control.
- Broader input-system redesign beyond date fields and project/context match dialogs.

</deferred>

---

*Phase: 50-input-ergonomics*
*Context gathered: 2026-05-19*

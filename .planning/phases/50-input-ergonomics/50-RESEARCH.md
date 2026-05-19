# Phase 50 Research: Input Ergonomics

**Phase:** 50-input-ergonomics
**Date:** 2026-05-19
**Status:** Planned

## Summary

Phase 50 naturally splits into two implementation slices:

1. improve the shared date-picker workflow used by explicit date-editing commands
2. refine the generic project/context suggestion popup so it opens with a useful default
   selection instead of forcing manual re-navigation

The current WPF client already has the right reuse points for both slices. `ShowDateDialog(...)`
in `Client/MainWindowViewModel.cs` centralizes date dialog launch behavior, and
`IntellisenseTextBox` in `Client/Controls/IntellisenseTextBox.cs` centralizes the popup
selection behavior used by project/context suggestion flows.

## Existing Shape

- `SetDueDateDialog` is the current date dialog. It is due-date-specific in name and label,
  but it is already reused by shared date-dialog plumbing in `MainWindowViewModel`.
- Date picker keyboard handling currently supports `T`, `Up`, `Down`, `Enter`, and `Escape`,
  but not `Left`/`Right` week jumps.
- Current separate date commands already exist for due and threshold fields, which matches the
  user decision to preserve separate commands instead of replacing them with a single launcher.
- `IntellisenseTextBox` shows a popup for `+`, `@`, and priority `(` triggers.
- `ShowIntellisensePopup(...)` explicitly clears selection on open, and filtering logic does
  not preserve or auto-pick the best match. Today the user often has to press `Down` first.

## Codebase Constraints

- The client is a legacy WPF application targeting .NET Framework 4.0 Client Profile.
- A `ToDoTests` NUnit project exists, but current codebase search did not reveal focused
  tests for `SetDueDateDialog`, `MainWindowViewModel` date-dialog logic, or
  `IntellisenseTextBox`.
- Because some WPF control behavior is awkward to unit test directly, the phase should plan
  for a mix of:
  - focused automated coverage where logic can be extracted or invoked cleanly
  - explicit manual verification for keyboard/popup interaction details if needed

## Recommended Implementation

### Date ergonomics

- Keep separate routed commands for each date target.
- Generalize the shared date dialog enough to show the active target clearly rather than
  always saying "Set due date".
- Extend shared date-dialog launch helpers for additional real date-bearing fields such as
  completed date, while leaving recurrence-rule editing out of scope.
- Add `Left`/`Right` week-jump behavior directly in the dialog's keyboard handler.

### Auto-select ergonomics

- Refine `IntellisenseTextBox` so the popup opens with the most useful default selection.
- Selection priority should be:
  1. current existing token/value
  2. best current typed match
  3. first suggestion
- Preserve that priority as filtering narrows the suggestion list so the popup feels stable
  rather than arbitrary.

## Plan Split

1. `50-01` - Shared date dialog generalization, real date-target expansion, and week-jump navigation.
2. `50-02` - Generic project/context suggestion auto-select continuity behavior.

These plans are largely independent and can live in the same wave if the planner wants a
parallel-friendly execution shape.

## Documentation Reconciliation

Current roadmap and requirement wording still imply a "recurring" date target in Phase 50.
The newer phase context supersedes that. Planning should update docs to say:

- separate commands remain
- real date-bearing fields are in scope
- recurrence-rule editing is not

## Testing Targets

Automated targets should at least cover:

- shared date dialog label/target selection behavior where logic is extractable
- week-jump date math for `Left`/`Right`
- `IntellisenseTextBox` selection behavior on popup open and after narrowing

Manual verification should cover:

- actual WPF focus/selection feel in the date picker
- actual popup selection feel in task entry, append-text, and filter-style text flows
- command wiring for any newly introduced completed-date action

## Verification Direction

- Build verification: `msbuild ToDo.Net.sln /p:Configuration=Debug /p:Platform=x86`
- Automated tests: extend `ToDoTests` where practical for non-visual logic
- Manual interaction checklist: date-picker arrow behavior, active target clarity, popup
  default selection continuity

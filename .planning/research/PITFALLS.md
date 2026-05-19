# Pitfalls Research

## Filter OR parsing pitfalls

- Ambiguous grammar around spaces and precedence can accidentally broaden matches.
- Empty OR segments (`a|`) should be treated defensively (ignore or fail closed).
- Negation behavior (`-`) can be surprising unless documented in examples.

## Recurring task pitfalls

- Prompting on completion introduces user-flow friction if default path is unclear.
- Replaying completion events must avoid duplicate follow-up tasks.
- Recurrence math must handle both strict (`+1d`) and relative (`1d`) modes consistently.

## TUI spacing and focus pitfalls

- Spacer rows can interfere with cursor navigation if all list index paths are not updated.
- Inactive pane focus suppression must preserve selection state for return visits.
- Accessibility of visual spacing should still meet terminal constraints for small screens.

## Archive rotation pitfalls

- Long-running file operations can block command completion on slow disk.
- Rotation naming collisions can occur without atomic rename strategy.
- Retention policy defaults must be documented to avoid unbounded disk growth.

## Prevention strategy

- Add regression tests around parsed terms and completed-todo flow.
- Add navigation tests around grouped rows and inactive panes.
- Keep defaults conservative (disabled or small thresholds) until rollout confidence is established.
- Add explicit warning/error messages when rotations occur or are skipped.


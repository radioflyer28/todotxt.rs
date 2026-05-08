# Quick Task 260507-nh1: Fix ctrl-left/right task movement between panes with missing filters - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning

<domain>
## Task Boundary

Fix ctrl-left/right arrow key movement of tasks between panes so it works correctly when source or destination pane has no @context or +project filter. Currently the feature only works when both panes have context/project filters. When moving to a filtered pane the appropriate tags should be applied; when moving to an unfiltered pane the source pane's tags should be removed.

</domain>

<decisions>
## Implementation Decisions

### Tag mutation scope
- Apply ALL context/project tokens from the destination pane's filter to the task (both @context and +project tokens)

### Multi-filter panes
- If the destination pane filter contains multiple context/project tokens, add ALL of them to the task

### Unfiltered source pane behavior
- When moving FROM a pane with a @context/+project filter TO a pane with NO filter, REMOVE the source pane's context/project filter tokens from the task

### Agent's Discretion
- Handling edge cases such as panes with non-context/project filter tokens (e.g. priority filters) — these should be ignored for tag mutation purposes; only @context and +project tokens participate in tag add/remove logic

</decisions>

<specifics>
## Specific Ideas

- The fix should handle all four combinations:
  1. Filtered → Filtered: remove source tags, add dest tags (existing behavior — ensure still works)
  2. Filtered → Unfiltered: remove source filter's @context/+project tags from task
  3. Unfiltered → Filtered: add dest filter's @context/+project tags to task
  4. Unfiltered → Unfiltered: no tag changes (task moves as-is)

</specifics>

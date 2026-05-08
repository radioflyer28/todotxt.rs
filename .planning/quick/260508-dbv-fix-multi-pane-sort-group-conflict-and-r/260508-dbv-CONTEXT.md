# Quick Task 260508-dbv: Fix multi-pane sort/group conflict and remove sort indicator from pane header - Context

**Gathered:** 2026-05-08
**Status:** Ready for planning

<domain>
## Task Boundary

Two related bugs when multiple panes are active:
1. Sort + group-by conflict: A pane set to "group by priority, sort by completed" works correctly as a single pane but produces broken mixed output when other panes are also displayed. The root cause is likely shared sort/group state bleeding across panes.
2. Pane header shows "sort: unknown" instead of the actual sort type (e.g. "sort: completed").

Additionally, remove the sort indicator from the pane header entirely — replace current header format with: pane label + filter string (if a filter is active), e.g. "Pane 3 | @work +CTRC" or just "Pane 3" when no filter.

</domain>

<decisions>
## Implementation Decisions

### Sort/group isolation scope
- Full isolation: each pane independently applies its own sort AND group-by. No cross-pane state bleed.

### Pane header content
- After removing sort: show pane label + filter string when a filter is active (e.g. "Pane 3 | @work +CTRC"), or just the pane label when no filter (e.g. "Pane 3").
- No sort indicator at all in the header.

### Sort: unknown display bug
- Agent's discretion: investigate root cause. If it's the same as the sort isolation bug, fix both together. If it's a separate rendering issue (e.g. the sort enum variant doesn't have a Display impl for the pane-header context), fix it as part of the header cleanup (since we're removing sort from the header anyway, the "unknown" label goes away automatically).

</decisions>

<specifics>
## Specific Ideas

- The screenshot shows "Pane 3 | sort: unknown" in the left pane header. After the fix it should show either "Pane 3" (if no filter) or "Pane 3 | <filter>" (if filtered).
- The sort/group bleed likely happens because pane rendering uses a shared sorted/grouped list rather than re-computing per-pane with that pane's own sort+group settings.
- When a single pane is shown the bug doesn't appear — consistent with a global sort list being reused across panes.

</specifics>

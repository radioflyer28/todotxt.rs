# Phase 12, Plan 03 — SUMMARY

## What Was Built
Updated `render_status_bar()` in the TUI to fully match D-13 behavior:
- Always shows `file | visible/total tasks`
- Shows due-today and overdue counts only when either is non-zero
- Computes due/overdue counts from `display_indices` only
- Shows active filter section only when filter query is non-empty after trim
- Shows sort section only when sort order is not `FileOrder`
- Uses `sort_name()` for sort label text
- Includes updated right-side key hints ending with `f filter | o sort`
- Applies truncation policy: omit hints first; if still too long, truncate middle with `…`

## Task 1 Completed
- Status bar: visible/total format: ✓
- Filter section conditional on active filter: ✓
- Sort section conditional on non-FileOrder sort: ✓
- Due/overdue counts from display_indices: ✓
- Key hints include f filter | o sort: ✓
- Commit: cf3f176

## Task 2: Human Verify Checkpoint
PENDING — requires user to run the TUI and test all 6 scenarios.

## Build Verification
- cargo build -p todotxt-tui: ✓ (zero warnings)

## Handoff to Human Verify
Run:

```bash
cargo run -p todotxt-tui -- path/to/todo.txt
```

Verify:
1. Baseline status bar with no filter/sort shows `file | N/M tasks`.
2. Apply filter via `f` and typing text; status bar shows `| {filter_query}` only when non-empty.
3. Cycle sort with `o`; status bar shows `| sort: {name}` only when not file order.
4. With active filter and sort, both sections appear in order.
5. When due-today/overdue in visible tasks are non-zero, status bar shows `| X due today | Y overdue`.
6. Key hints include `q quit | n add | u edit | d del | x done | j/k nav | f filter | o sort`.

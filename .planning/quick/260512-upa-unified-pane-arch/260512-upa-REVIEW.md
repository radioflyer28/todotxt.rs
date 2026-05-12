---
phase: 260512-upa-unified-pane-arch
reviewed: 2026-05-12T00:00:00Z
depth: standard
files_reviewed: 1
files_reviewed_list:
  - crates/todotxt-tui/src/app.rs
findings:
  blocker: 1
  warning: 3
  info: 2
  total: 6
status: issues_found
---

# 260512-upa: Code Review Report

**Commit:** `5fd2fe5`
**Reviewed:** 2026-05-12
**Depth:** standard (correctness-focused, quick bias)
**Files Reviewed:** 1 (`crates/todotxt-tui/src/app.rs`)
**Status:** issues_found

## Summary

The refactor is structurally clean — 7 duplicated App-level fields removed, all rendering paths
now read from `self.panes[self.active_pane]` consistently. The borrow-safety blocks added for
half-page scrolls are correct. Navigation logic in `pane_move_down` / `pane_move_up` is
equivalent to the pre-refactor per-pane path.

**One blocker**: `show_deferred` / `deferred_toggle` is functionally broken for single-pane
users. The old single-pane render path (`rebuild_display_indices`) applied
`suppress_future_threshold = false` when `show_deferred` was set and exposed ALL tasks (including
deferred) in unfiltered mode. The new unified path (`rebuild_visible_rows` /
`rebuild_all_panes`) calls `Filter::from_query(...)` which defaults
`suppress_future_threshold = true` and never reads `self.show_deferred`. Tasks with a future
`t:` date are silently invisible in all views; the toggle key has no effect on visibility.

---

## Blocker Issues

### BL-01: `show_deferred` never propagated to Filter — deferred tasks permanently invisible

**File:** [crates/todotxt-tui/src/app.rs](crates/todotxt-tui/src/app.rs#L724)
**Lines:** `rebuild_visible_rows` (≈L724) and `rebuild_all_panes` (≈L793)

**Issue:**
`Filter::from_query(filter_str)` always returns `Filter { suppress_future_threshold: true, .. }`.
Neither `rebuild_visible_rows` nor `rebuild_all_panes` reads `self.show_deferred`. As a result:

1. Tasks with a future `t:` threshold date are **always suppressed** from every pane's
   `display_rows`, regardless of the `deferred_toggle` state.
2. In the old single-pane path (`rebuild_display_indices`), an **empty** filter bypassed
   `TaskList::filter` entirely — `task_list.tasks().iter().enumerate().collect()` — so all tasks
   including deferred ones were visible by default. The new path calls `task_list.filter(&filter)`
   for every pane, empty filter or not, always suppressing deferred.
3. The `deferred_toggle` key now only affects visual dimming in `render_task_list` / `PaneList::render`
   — it has no effect on which rows appear in `display_rows`.
4. **Lock-out edge case**: if all tasks carry a future `t:` date, `display_count == 0`,
   so the `deferred_toggle` guard (`display_count > 0`) prevents the key from firing at all.
   The user cannot recover visibility via keyboard.

**Root cause:** The deletion of `rebuild_display_indices` removed the only code path that
called `f.suppress_future_threshold = false`. `rebuild_visible_rows` and `rebuild_all_panes`
were the pre-existing multi-pane paths and never had this logic. The refactor silently
extended the broken multi-pane behavior to single-pane mode.

**Fix — `rebuild_visible_rows`** (add `mut` to `filter` and read `self.show_deferred`):
```rust
// Per-pane query behavior (D-04, Phase 25): Apply active pane's filter_query
let filter_str = pane.filter_query.trim().to_string(); // clone to release pane borrow
let mut filter = Filter::from_query(&filter_str);
if self.show_deferred {
    filter.suppress_future_threshold = false;
}

let mut filtered_tasks: Vec<(usize, &Task)> = self
    .task_list
    .filter(&filter)
    .into_iter()
    .collect();
```

**Fix — `rebuild_all_panes`** (same pattern inside the per-pane loop):
```rust
let mut filter = Filter::from_query(filter_query.trim());
if self.show_deferred {
    filter.suppress_future_threshold = false;
}
let mut filtered: Vec<(usize, &Task)> = self
    .task_list
    .filter(&filter)
    .into_iter()
    .collect();
```

Note: in `rebuild_visible_rows`, `filter_str` must be cloned (or captured as an owned `String`
before the `&mut self.panes[pane_idx]` borrow) so that `self.show_deferred` can be read in
the same scope without lifetime conflict.

---

## Warnings

### WR-01: `rebuild_and_reanchor` missing `reconcile_active_pane()` guard

**File:** [crates/todotxt-tui/src/app.rs](crates/todotxt-tui/src/app.rs#L3108)

**Issue:**
`rebuild_and_reanchor` directly indexes `self.panes[self.active_pane]` on its first line with no
bounds check:
```rust
fn rebuild_and_reanchor(&mut self) {
    let pane = &self.panes[self.active_pane];   // ← panics if panes is empty
    ...
}
```
Every other navigation entry-point calls `self.reconcile_active_pane()` first
(`pane_move_down`, `pane_move_up`, `active_pane_mut`, etc.). `rebuild_and_reanchor` is the
only mutating path that skips this. In practice, all callers either call `reconcile_active_pane`
beforehand or come through paths where panes is guaranteed non-empty, so this has not caused
a panic. But it is an asymmetry that could bite on a future code path.

**Fix:**
```rust
fn rebuild_and_reanchor(&mut self) {
    self.reconcile_active_pane();
    let pane = &self.panes[self.active_pane];
    ...
}
```

---

### WR-02: Half-page scroll can leave cursor on a GroupHeader when `display_rows` ends with one

**File:** [crates/todotxt-tui/src/app.rs](crates/todotxt-tui/src/app.rs#L1053)
**Lines:** All four Ctrl+D/U and Shift+Ctrl+D/U arms (≈L1053, L1075, L1099, L1121)

**Issue:**
Each arm runs this pattern:
```rust
pane.selected = (pane.selected + half).min(pane.display_rows.len().saturating_sub(1));
while pane.selected < pane.display_rows.len()
    && matches!(pane.display_rows[pane.selected], DisplayRow::GroupHeader(_))
{
    pane.selected += 1;     // can overshoot to display_rows.len()
}
if pane.display_rows.is_empty() {
    pane.selected = 0;
} else {
    pane.selected = pane.selected.min(pane.display_rows.len() - 1);  // clamps back
}
```
If the last `N` rows are all `GroupHeader` entries (the while-loop overshoots to `len`), the
final clamp restores `selected` to `len - 1` which is still a GroupHeader. The cursor is then
visually stuck on a header; j/k will move off it correctly, but the half-page scroll itself
placed the cursor in an invalid position.

In practice, `rebuild_visible_rows` and `rebuild_all_panes` never emit a trailing GroupHeader
(headers always precede tasks), so this is a latent concern rather than a currently
reproducible bug. The code relies on that invariant implicitly. The same pattern existed in
the old code using `self.selected` / `self.display_rows`, so this is not a regression; it's
a pre-existing fragility that survives unchanged.

**Fix (defensive):** After the while-loop, walk *backwards* instead of clamping if
`pane.selected >= pane.display_rows.len()`:
```rust
if pane.selected >= pane.display_rows.len() {
    // Overshot — walk back to last Task row
    pane.selected = pane.display_rows.len();
    while pane.selected > 0 {
        pane.selected -= 1;
        if matches!(pane.display_rows[pane.selected], DisplayRow::Task(_)) {
            break;
        }
    }
} else if pane.display_rows.is_empty() {
    pane.selected = 0;
}
```

---

### WR-03: Stale comment in test references deleted sync behavior

**File:** [crates/todotxt-tui/src/app.rs](crates/todotxt-tui/src/app.rs#L6509)

**Issue:**
```
// Note: in single-pane mode rebuild_and_reanchor syncs pane.group_by → self.group_by,
// so pane.group_by is the canonical way to set the group-by category (260512-gbx fix).
```
The sync block `self.group_by = pane.group_by` (and the other 3 field syncs) was explicitly
deleted by this commit. The comment now describes behavior that no longer exists, which will
mislead future reviewers looking at the test for `group_by_cycle_changes_display_rows_in_single_pane`.

**Fix:** Delete the two-line comment. The surrounding test still correctly validates
`pane.group_by` as the canonical field.

---

## Info

### IN-01: Active pane rebuilt twice in `save_and_exit` Adding/Editing paths

**File:** [crates/todotxt-tui/src/app.rs](crates/todotxt-tui/src/app.rs#L3031)

**Issue:**
Both the Adding and Editing branches call:
```rust
self.rebuild_all_panes();       // rebuilds ALL panes, including active
self.rebuild_and_reanchor();    // calls rebuild_visible_rows() → rebuilds active pane again
```
The active pane is rebuilt twice. The second rebuild is more expensive than necessary;
`rebuild_and_reanchor` is called purely for the reanchor logic (find old_canonical in new
display_rows), but the display_rows it reads were already set by `rebuild_all_panes`. No
correctness issue — the second rebuild produces the same result as the first.

**Fix (optional):** Replace the double-rebuild with a direct cursor placement after a single
`rebuild_all_panes()`:
```rust
self.rebuild_all_panes();
// Reanchor to newly-added/edited task
if let Some(pos) = self.panes[self.active_pane].display_rows.iter()
    .position(|r| matches!(r, DisplayRow::Task(idx) if *idx == canonical))
{
    self.panes[self.active_pane].selected = pos;
}
```

---

### IN-02: Stale `self.display_rows` reference in test docstring

**File:** [crates/todotxt-tui/src/app.rs](crates/todotxt-tui/src/app.rs#L7666)

**Issue:**
Test docstring says:
```
/// Verify that cycling group-by in single-pane mode changes self.display_rows visually.
```
`self.display_rows` no longer exists as a field. The test itself is correct (it reads
`app.panes[0].display_rows`); the doc comment is just stale.

**Fix:** Update the docstring:
```
/// Verify that cycling group-by in single-pane mode changes pane.display_rows visually.
```

---

## Navigation correctness (review questions answered)

**Q1 — Remaining stale references to deleted fields?**
None found in production code. All 7 fields (`selected`, `display_rows`, `display_indices`,
`grouping`, `group_by`, `sort_order`, `filter_query`) are gone from `App`. Test docstrings
retain two stale references (IN-02, WR-03) that are harmless but misleading.

**Q2 — Unsafe `unwrap()` on pane access?**
No `unwrap()` on pane indexing. All pane access uses direct indexing (`self.panes[idx]`), which
panics on OOB but only if `reconcile_active_pane()` was not called. The missing guard in
`rebuild_and_reanchor` is the only asymmetry (WR-01). No `.unwrap()` escape hatches introduced.

**Q3 — `pane_move_down` / `pane_move_up` cursor correctness after global sync removal?**
Both functions now operate entirely on `pane.display_rows` and `pane.selected`. The old
`use_global_cursor` sync block (which copied `self.selected` → `pane.selected` before moving,
then copied back after) is correctly eliminated. The new code is simpler and avoids the
double-assignment that was the root of cursor-jump bugs. Group-header skipping logic is
equivalent to the old per-pane path. No regressions in these two functions.

**Q4 — `render_task_list` correctness?**
Correct. Uses `pane.display_rows`, `pane.selected`, `pane.grouping`. The `display_count == 0`
empty-state check now counts `DisplayRow::Task` entries from pane rows (previously from
`display_indices.len()`), which is equivalent. `list_state.with_selected(Some(pane.selected))`
is correctly gated on `display_count > 0`.

**Q5 — `save_and_exit` cursor placement after add/edit?**
Correct (but see IN-01 for the redundant rebuild). After `rebuild_all_panes` +
`rebuild_and_reanchor`, a direct position-search for the canonical index overrides whatever
`rebuild_and_reanchor` set. If the newly-added/edited task is filtered out by the active pane's
filter, the cursor gracefully falls back to the `rebuild_and_reanchor` position (no panic,
no forced jump to 0).

**Q6 — Other logic regressions?**
The `deferred_toggle` regression (BL-01) is the only logic regression introduced. All other
code paths that read `show_deferred` (visual dimming in `render_task_list`, `PaneList::render`,
status bar "[+deferred]" indicator) are unaffected.

---

_Reviewed: 2026-05-12_
_Reviewer: gsd-code-reviewer_
_Depth: standard (quick bias, correctness-focused per task spec)_

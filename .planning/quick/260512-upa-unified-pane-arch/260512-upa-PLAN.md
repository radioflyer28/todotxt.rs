---
id: 260512-upa
slug: unified-pane-arch
title: Eliminate App-level shadow display state — unify single/multi-pane code paths
date: 2026-05-12
must_haves:
  truths:
    - App struct no longer has fields: selected, display_rows, display_indices, grouping, group_by, sort_order, filter_query
    - rebuild_display_indices function is deleted
    - clamp_selection function is deleted
    - rebuild_and_reanchor has no sync block and does not call rebuild_display_indices
    - render_task_list reads from pane.display_rows and pane.selected (not App-level fields)
    - render_status_bar always reads from pane (no single-pane fallback branch using App-level fields)
    - pane_move_down and pane_move_up do not sync from/to self.selected
    - App::new does not call rebuild_display_indices and does not initialize deleted fields
    - all existing 230 cargo tests pass after refactor
    - the two reported bugs are fixed: cursor-jump at startup and sort-enables-grouping appearance
  artifacts:
    - crates/todotxt-tui/src/app.rs
---

# Plan: Eliminate App-level shadow display state (260512-upa)

## Background

Single-pane mode has historically maintained a shadow copy of display state in App-level
fields (`self.selected`, `self.display_rows`, `self.display_indices`, `self.grouping`,
`self.sort_order`, `self.group_by`, `self.filter_query`). These duplicate the per-pane
fields (`pane.selected`, `pane.display_rows`, etc.) and must be kept in sync manually
in `rebuild_and_reanchor`. Every sync omission is a bug — we've now fixed three:
- 260512-ksx: rebuild_visible_rows not called → stale pane.display_rows
- 260512-gbx: group_by not synced → group_by_cycle had no visual effect
- Current bugs: cursor-jump at startup; sort appearing to enable grouping

The fix is to delete the App-level shadow state entirely and route all single-pane
rendering and navigation through the same pane fields used by multi-pane mode.

## Root cause of current bugs

**Bug 1 — cursor jumps at startup (no sort):**
`rebuild_and_reanchor` is not called at startup (only `rebuild_all_panes` +
`rebuild_display_indices`). If `pane.display_rows` differs from `self.display_rows` at
any point (e.g. config sets pane grouping, or ordering differs between rebuild paths),
navigation (pane.selected) and rendering (self.selected, self.display_rows) diverge,
producing visible cursor jumps. The dual-rebuild path also means App::new must call
both `rebuild_all_panes` AND `rebuild_display_indices` to stay in sync — fragile.

**Bug 2 — sort enables grouping appearance:**
`render_task_list` reads `self.grouping` (line ~3678) which is synced from
`pane.grouping` in `rebuild_and_reanchor`. But any read of `self.grouping` before a
rebuild (e.g. on first render, or after a stale sync) shows the wrong value. After the
refactor, render always reads `pane.grouping` directly — no sync required.

## Approach

All changes are in `crates/todotxt-tui/src/app.rs`.

**Fields to remove from App struct:**
- `pub selected: usize` — replaced by `self.panes[self.active_pane].selected`
- `pub display_rows: Vec<DisplayRow>` — replaced by `self.panes[self.active_pane].display_rows`
- `pub display_indices: Vec<usize>` — no longer needed (was only used by rebuild_display_indices)
- `pub grouping: bool` — replaced by `self.panes[self.active_pane].grouping`
- `pub group_by: GroupByCategory` — replaced by `self.panes[self.active_pane].group_by`
- `pub sort_order: SortOrder` — replaced by `self.panes[self.active_pane].sort_order`
- `pub filter_query: String` — replaced by `self.panes[self.active_pane].filter_query`

**Helper to add (avoids verbose indexing):**
```rust
/// Active pane's display rows (used by single-pane render and nav paths).
fn active_display_rows(&self) -> &Vec<DisplayRow> {
    &self.panes[self.active_pane].display_rows
}
```
(Use `self.active_pane()` and `self.active_pane_mut()` which already exist.)

---

## Task 1 — Remove App shadow fields + rebuild path

**File:** `crates/todotxt-tui/src/app.rs`

### 1a. Remove 7 fields from App struct declaration

Remove these field declarations (keep all other fields):
```
pub selected: usize,
pub display_indices: Vec<usize>,
pub grouping: bool,
pub group_by: GroupByCategory,
pub display_rows: Vec<DisplayRow>,
pub sort_order: SortOrder,
pub filter_query: String,
```

### 1b. Remove from App::new struct literal initializer

Remove these initializer lines from the `App { ... }` literal in `App::new` (~line 451):
```
selected: 0,
display_indices: Vec::new(),
grouping: false,
group_by: GroupByCategory::Priority,
display_rows: Vec::new(),
sort_order: SortOrder::FileOrder,
filter_query: String::new(),
```

### 1c. Remove rebuild_display_indices call from App::new

In `App::new`, remove the line:
```rust
app.rebuild_display_indices();
```
Keep `app.rebuild_all_panes();` — this is sufficient.

### 1d. Delete rebuild_display_indices function

Delete the entire `fn rebuild_display_indices(&mut self)` function body (~lines 3109-3167).

### 1e. Delete clamp_selection function

Delete the entire `fn clamp_selection(&mut self)` function body (~lines 3025-3031).

### 1f. Simplify rebuild_and_reanchor

In `rebuild_and_reanchor` (~lines 3169-3230):

1. **Remove** the entire sync block:
```rust
if self.should_show_single_pane() || self.panes_hidden {
    let pane = &self.panes[self.active_pane];
    self.filter_query = pane.filter_query.clone();
    self.sort_order = pane.sort_order;
    self.grouping = pane.grouping;
    self.group_by = pane.group_by;
}
```

2. **Remove** the call to `self.rebuild_display_indices();`

3. **Remove** the `self.selected = ...` reanchor block and `self.clamp_selection()` call:
```rust
self.selected = old_canonical
    .and_then(|ci| {
        self.display_rows
            .iter()
            .position(|r| matches!(r, DisplayRow::Task(idx) if *idx == ci))
    })
    .unwrap_or(0);
self.clamp_selection();
```

After removal, `rebuild_and_reanchor` should only:
- Capture `old_canonical` from active pane
- Call `self.rebuild_visible_rows()`
- Reanchor `pane.selected` to `old_canonical`

### 1g. Fix display_count / row_count at top of handle_normal_key

Replace (~lines 990-997):
```rust
let display_count = self.display_indices.len();
let row_count = if !self.should_show_single_pane() && self.panes.len() > 1 && !self.panes_hidden {
    self.panes[self.active_pane].display_rows.len()
} else {
    self.display_rows.len()
};
```

With:
```rust
let pane_display_rows = &self.panes[self.active_pane].display_rows;
let display_count = pane_display_rows.iter().filter(|r| matches!(r, DisplayRow::Task(_))).count();
let row_count = pane_display_rows.len();
```

Note: `pane_display_rows` is an immutable borrow; it must be dropped before any mutable
borrow in the match arms. Introduce it as a separate let binding scoped only for computing
these two values, then drop it:
```rust
let display_count;
let row_count;
{
    let pane_rows = &self.panes[self.active_pane].display_rows;
    display_count = pane_rows.iter().filter(|r| matches!(r, DisplayRow::Task(_))).count();
    row_count = pane_rows.len();
}
```

### 1h. Remove rebuild_display_indices from apply_pane_layout_preset

In `apply_pane_layout_preset` (~line 271), remove the call:
```rust
self.rebuild_display_indices();
```
Keep `self.rebuild_all_panes();` — it is sufficient.

### 1i. Remove rebuild_display_indices from pane_move_task

In `pane_move_task` (~line 393), remove the call:
```rust
self.rebuild_display_indices();
```
Keep `self.rebuild_all_panes();` — it is sufficient.

### 1j. Fix deferred_toggle key handler

In `handle_normal_key` at the `deferred_toggle` arm (~line 1343), replace:
```rust
self.rebuild_display_indices();
self.clamp_selection();
```
With:
```rust
self.rebuild_and_reanchor();
```

### 1k. Fix save_and_exit cursor-positioning

In `save_and_exit` (~lines 3028 and 3050), there are two branches (Adding and Editing) that do:
```rust
self.rebuild_display_indices();
self.rebuild_all_panes();
self.selected = self.display_rows.iter()
    .position(|r| matches!(r, DisplayRow::Task(idx) if *idx == canonical))
    .unwrap_or(0);
```

Replace each instance with the simpler:
```rust
self.rebuild_and_reanchor();
```

`rebuild_and_reanchor` already reanchors the pane cursor to the canonical task index (old_canonical), so the manual `self.selected` positioning is redundant and incorrect after removal of App-level fields.

Note: `save_and_exit` must pass `canonical` (the just-saved task's canonical index) so the cursor lands on it. Verify how `old_canonical` is captured inside `rebuild_and_reanchor` vs. the `canonical` variable in `save_and_exit`. If `save_and_exit` needs to force `pane.selected` to a specific canonical index before calling `rebuild_and_reanchor`, add:
```rust
// Point pane.selected to the just-saved task so rebuild_and_reanchor reanchors to it
if let Some(pos) = self.panes[self.active_pane].display_rows.iter()
    .position(|r| matches!(r, DisplayRow::Task(idx) if *idx == canonical))
{
    self.panes[self.active_pane].selected = pos;
}
self.rebuild_and_reanchor();
```

### 1l. Fix status_scope_task_indices single-pane branch

In `status_scope_task_indices` (~line 3950), replace the else branch that uses `self.display_indices`:
```rust
} else {
    self.display_indices.clone()
}
```
With pane-based task index collection:
```rust
} else {
    self.panes[self.active_pane].display_rows.iter()
        .filter_map(|r| if let DisplayRow::Task(idx) = r { Some(*idx) } else { None })
        .collect()
}
```

---

## Task 2 — Update render path

**File:** `crates/todotxt-tui/src/app.rs`

### 2a. Update render_task_list to use pane fields

In `render_task_list` (~lines 3640-3713), replace all uses of App-level fields with pane fields.

The function is called in single-pane mode. After the refactor, it reads from `panes[active_pane]`:

Key replacements:
- `self.display_indices.is_empty()` → `self.panes[self.active_pane].display_rows.iter().all(|r| matches!(r, DisplayRow::GroupHeader(_)))` or use `display_count == 0` where `display_count = pane.display_rows.iter().filter(...).count()`
- `self.display_rows` → `self.panes[self.active_pane].display_rows`
- `self.selected` → `self.panes[self.active_pane].selected`
- `self.grouping` → `self.panes[self.active_pane].grouping`
- `self.display_rows.get(self.selected)` → `self.panes[self.active_pane].display_rows.get(self.panes[self.active_pane].selected)`

Specifically:
1. The "no tasks" guard: replace `self.display_indices.is_empty()` check with `pane.display_rows.is_empty() || display_count == 0` (where `display_count` is computed locally before the borrow)
2. The item-building loop: `self.display_rows.iter().enumerate()` → `pane.display_rows.iter().enumerate()`
3. `let is_cursor = row_idx == self.selected` → `let is_cursor = row_idx == pane.selected`
4. `let indent = if self.grouping { "  " } else { "" }` → `let indent = if pane.grouping { "  " } else { "" }`
5. Cursor-is-selected check: `self.display_rows.get(self.selected)` → `pane.display_rows.get(pane.selected)`
6. ListState: `if !self.display_indices.is_empty()` → `if !pane.display_rows.is_empty()` (or use `display_count > 0`)
7. `list_state.with_selected(Some(self.selected))` → `list_state.with_selected(Some(pane.selected))`

Borrow pattern (to avoid borrow conflicts):
```rust
fn render_task_list(&self, frame: &mut Frame, area: Rect) {
    let pane = &self.panes[self.active_pane];
    let tasks = self.task_list.tasks();
    let display_count = pane.display_rows.iter().filter(|r| matches!(r, DisplayRow::Task(_))).count();
    // ... use pane.display_rows, pane.selected, pane.grouping throughout
}
```

### 2b. Update render_status_bar single-pane branch

In `render_status_bar` (~lines 3853-3870), replace the single-pane fallback:
```rust
} else {
    // Fallback to global state when showing single pane
    (
        self.filter_query.clone(),
        self.sort_order,
        self.grouping,
        self.group_by,
    )
```

With the same pane-based read as the multi-pane branch:
```rust
} else {
    let pane = &self.panes[self.active_pane];
    (
        pane.filter_query.clone(),
        pane.sort_order,
        pane.grouping,
        pane.group_by,
    )
```

(Or simplify the entire if/else to always use `let pane = &self.panes[self.active_pane]` since the pane path is now universal.)

---

## Task 3 — Update navigation and action handlers

**File:** `crates/todotxt-tui/src/app.rs`

### 3a. Simplify pane_move_down

Remove the `use_global_cursor` sync from `pane_move_down` (~lines 3455-3492).

Remove:
- `let use_global_cursor = self.should_show_single_pane() || self.panes_hidden;`
- `let global_selected = self.selected;`
- The `if use_global_cursor { pane.selected = global_selected.min(...) }` block at start
- The `if use_global_cursor { self.selected = pane.selected; }` block at end

After removal, the function simply moves `pane.selected` within `pane.display_rows`.

### 3b. Simplify pane_move_up

Same pattern as 3a — remove `use_global_cursor` sync blocks from `pane_move_up` (~lines 3497-3538).

### 3c. Update page up/down handlers (Ctrl+U / Ctrl+D)

In `handle_normal_key` at ~lines 1079-1133, replace direct `self.selected` manipulation with pane-based:

Replace pattern:
```rust
self.selected = self.selected.saturating_sub(half);
while self.selected < row_count
    && matches!(self.display_rows[self.selected], DisplayRow::GroupHeader(_))
{
    self.selected += 1;
}
self.clamp_selection();
```
With (×2 for both shift and non-shift variants of Ctrl+U):
```rust
{
    let pane = self.active_pane_mut();
    pane.selected = pane.selected.saturating_sub(half);
    while pane.selected < pane.display_rows.len()
        && matches!(pane.display_rows[pane.selected], DisplayRow::GroupHeader(_))
    {
        pane.selected += 1;
    }
    if pane.display_rows.is_empty() {
        pane.selected = 0;
    } else {
        pane.selected = pane.selected.min(pane.display_rows.len() - 1);
    }
}
```
Same for Ctrl+D (×2), but using `(pane.selected + half).min(row_count.saturating_sub(1))` instead of `saturating_sub`.

### 3d. Update canonical_selected and related functions

**canonical_selected** (~line 3105): change to use active pane:
```rust
fn canonical_selected(&self) -> Option<usize> {
    let pane = &self.panes[self.active_pane];
    pane.display_rows.get(pane.selected)
        .and_then(|r| if let DisplayRow::Task(idx) = r { Some(*idx) } else { None })
}
```

**pane_canonical_selected** (~line 3095): now that both paths use pane, simplify:
```rust
fn pane_canonical_selected(&self) -> Option<usize> {
    let pane = &self.panes[self.active_pane];
    pane.display_rows.get(pane.selected)
        .and_then(|r| if let DisplayRow::Task(idx) = r { Some(*idx) } else { None })
}
```
(Both functions become identical — consider whether to keep both or just use `canonical_selected` everywhere.)

**toggle_task_selection** (~line 3253): replace:
```rust
if let Some(DisplayRow::Task(idx)) = self.display_rows.get(self.selected).cloned() {
```
With:
```rust
let pane_sel = self.panes[self.active_pane].selected;
if let Some(DisplayRow::Task(idx)) = self.panes[self.active_pane].display_rows.get(pane_sel).cloned() {
```

**apply_range_selection** (~line 3285): remove the single-pane branch:
```rust
let (rows, selected_row) = if !self.should_show_single_pane() && self.panes.len() > 1 && !self.panes_hidden {
    let pane = &self.panes[self.active_pane];
    (&pane.display_rows, pane.selected)
} else {
    (&self.display_rows, self.selected)
};
```
Replace with:
```rust
let pane = &self.panes[self.active_pane];
let (rows, selected_row) = (&pane.display_rows, pane.selected);
```

**Line ~1647** (targets computation in delete/edit handler):
```rust
} else if let Some(DisplayRow::Task(idx)) = self.display_rows.get(self.selected) {
```
Replace with:
```rust
} else {
    let pane_sel = self.panes[self.active_pane].selected;
    if let Some(DisplayRow::Task(idx)) = self.panes[self.active_pane].display_rows.get(pane_sel) {
        vec![*idx]
    } else {
        vec![]
    }
}
```
(Adjust the surrounding match arms so this reads cleanly.)

### 3e. Update push_undo_entry and apply_undo

**push_undo_entry** (~line 524):
```rust
selected: self.selected,
```
→ `selected: self.panes[self.active_pane].selected,`

**apply_undo** (~line 592):
```rust
self.selected = entry.selected;
```
→ `self.panes[self.active_pane].selected = entry.selected;`

---

## Task 4 — Update tests

**File:** `crates/todotxt-tui/src/app.rs` (test module at bottom)

In the `#[cfg(test)]` module, replace all direct App field accesses with pane field accesses.
Use search-and-replace for each pattern. All tests use `app.panes[0]` since tests always
create single-pane apps via `make_app_with_tasks`.

Replacements (all occurrences):
| Old | New |
|-----|-----|
| `app.selected` | `app.panes[0].selected` |
| `app.display_rows` | `app.panes[0].display_rows` |
| `app.display_indices` | `app.panes[0].display_rows.iter().filter(\|r\| matches!(r, DisplayRow::Task(_))).count()` (or just delete the assertion if it's a count check) |
| `app.grouping` | `app.panes[0].grouping` |
| `app.sort_order` | `app.panes[0].sort_order` |
| `app.group_by` | `app.panes[0].group_by` |
| `app.filter_query` | `app.panes[0].filter_query` |

**Special cases (do NOT use blanket replacement):**

1. **Line ~6028** — assertion `assert_eq!(app.filter_query, "", "global filter_query must remain empty")`:
   - Delete this assertion entirely — the global field is gone; there is nothing to verify isolation against.

2. **Test `rebuild_display_indices_does_not_clear_selected_tasks`** (line ~5416):
   - This test calls `app.rebuild_display_indices()` directly. Delete the test entirely — the function no longer exists.

3. **Test `delete_undo_round_trip`** (line ~6115):
   - Calls `app.rebuild_display_indices()` in setup. Replace with `app.rebuild_and_reanchor()`.

4. **Test `group_by_cycle_changes_display_rows_in_single_pane`** (added in 260512-gbx):
   - Contains `assert_eq!(app.group_by, app.panes[0].group_by)` — after blanket replacement of `app.group_by` → `app.panes[0].group_by`, this becomes `assert_eq!(app.panes[0].group_by, app.panes[0].group_by)` which is a tautology.
   - Replace this specific assertion with a behavioral assertion instead:
     ```rust
     assert_eq!(app.panes[0].group_by, GroupByCategory::Project,
         "pane.group_by must be Project after cycling (260512-gbx sync check)");
     ```

---

## Verification

After all changes:
```
cargo test -p todotxt-tui --lib
```
All 230 tests must pass. Fix any compilation errors or test failures before committing.

Do NOT run `cargo clippy` or any other linter — just `cargo test`.

## Commit message

```
refactor(260512-upa): eliminate App-level shadow display state

Remove 7 App-level fields that duplicated per-pane state in single-pane mode:
selected, display_rows, display_indices, grouping, group_by, sort_order, filter_query.

Previously, rebuild_and_reanchor() synced these from the active pane before calling
rebuild_display_indices() (global render path). Every missed sync was a bug:
- 260512-ksx: stale pane.display_rows after rebuild
- 260512-gbx: group_by never synced → group_by_cycle had no visual effect
- This task: cursor-jump at startup; sort apparently enabling grouping

Fix: render_task_list, render_status_bar, pane_move_down/up, page-scroll handlers,
undo, canonical_selected, and all related code now read directly from the active
pane. rebuild_display_indices and clamp_selection are deleted. The sync block in
rebuild_and_reanchor is deleted.

230/230 tests pass.
```

# Quick Task 260508-dbv — Research
# Fix multi-pane sort/group conflict and remove sort indicator from pane header

**Researched:** 2026-05-08  
**Domain:** Rust TUI — ratatui, `crates/todotxt-tui/src/`  
**Confidence:** HIGH (all findings verified directly in source)

---

## Summary

Two related bugs. Both have clear, localized root causes.

**Bug 1 (sort+group conflict in multi-pane):** The single-pane code path
(`rebuild_display_indices`, app.rs:3081) applies a *secondary stable sort by group key*
after the primary sort, ensuring all tasks with the same group key are adjacent before the
grouping loop emits headers. Both multi-pane paths (`rebuild_visible_rows` and
`rebuild_all_panes`) skip this secondary sort — they just iterate the primary-sorted slice
and emit a new header whenever the key changes, producing duplicate group headers whenever
the primary sort interleaves tasks from different groups (e.g. `CompletedDate` sort with
`Priority` group-by scatters (A) tasks throughout the list → "(A)" header appears multiple
times).

**Bug 2 / Header change ("sort: unknown"):** `PaneList::render` (pane_list.rs:97-108) has a
`_ => "unknown"` fallback in its sort-name match. `SortOrder::CompletedDate`,
`CreationDate`, `Context`, and `Project` all fall through. These are reachable because
`cycle_sort()` assigns any of them to `pane.sort_order`. Since the CONTEXT.md decision is
to remove the sort indicator entirely, the "unknown" label goes away automatically.

---

## Q1 — Pane task-list build path: where does sort+group happen?

### Single-pane path (app.rs line 3081)

```
rebuild_and_reanchor()
  → rebuild_display_indices()          ← single-pane + panes_hidden
      1. filter tasks with pane.filter_query
      2. sort by pane.sort_order (primary sort)
      3. if grouping:
           stable-sort by group_key_for()   ← ✅ secondary sort
           loop → emit GroupHeader when key changes
```

**Key lines (app.rs 3100–3125):**

```rust
// primary sort
if sort_order != SortOrder::FileOrder {
    pairs.sort_by(|(_, a), (_, b)| sort_order.compare(a, b));
}

// secondary stable-sort by group key ← THE STEP THAT'S MISSING IN MULTI-PANE
self.display_indices.sort_by(|&a, &b| {
    let ka = group_key_for(&tasks[a], &group_by);
    let kb = group_key_for(&tasks[b], &group_by);
    ka.cmp(&kb)           // stable_by preserves intra-group primary order
});
```

### Multi-pane: rebuild_visible_rows (app.rs line 738)

Rebuilds **active pane only** (called from `rebuild_and_reanchor`):

```
1. filter with pane.filter_query
2. sort by pane.sort_order (primary sort)
3. if pane.grouping:
     loop → emit GroupHeader when key changes   ← ❌ no secondary sort
```

**Missing step:** after line 761 `filtered_tasks.sort_by(...)`, there is no
`filtered_tasks.sort_by(group_key)` before the grouping loop (line 764).

### Multi-pane: rebuild_all_panes (app.rs line 797)

Rebuilds **every pane** (called after task mutations, file reloads, config changes):

```
for each pane idx:
  1. clone per-pane filter_query / sort_order / grouping / group_by
  2. filter + primary sort
  3. if grouping:
       loop → emit GroupHeader when key changes   ← ❌ no secondary sort
```

Same missing secondary sort (app.rs ~line 830, inside the `if grouping` branch).

### No shared intermediate list

Each path builds its own `Vec<(usize, &Task)>` from `task_list.filter()` — there is no
shared sorted buffer reused across panes. The bug is purely algorithmic: the secondary
sort step that makes the single-pane path correct is absent from both multi-pane paths.

---

## Q2 — Pane header construction: where and what causes "sort: unknown"

**File:** `crates/todotxt-tui/src/components/pane_list.rs`  
**Function:** `PaneList::render` (line 49)  
**Header build:** lines 65–110

Current header logic:

```rust
// label part (lines 65-81)
header_parts.push(label_display);          // "▶ Pane 3" or "  Pane 3"

// filter part (lines 83-91)
if !trimmed_filter.is_empty() {
    header_parts.push(format!("filter: {}", filter_display));
}

// sort part (lines 93-108)  ← TO BE REMOVED
if pane.sort_order != SortOrder::FileOrder {
    let sort_name = match pane.sort_order {
        SortOrder::FileOrder => "file",
        SortOrder::Alphabetical => "alpha",
        SortOrder::Priority => "priority",
        SortOrder::DueDate => "due",
        _ => "unknown",   // ← CompletedDate / CreationDate / Context / Project
    };
    header_parts.push(format!("sort: {}", sort_name));
}

let title = header_parts.join(" | ");   // line 110–118
```

**"sort: unknown" cause:** `SortOrder` has 8 variants; `cycle_sort()` (app.rs:4482) cycles
through all 8 at runtime. The match arm in `pane_list.rs` only lists 4 variants; the other
4 fall through to `"unknown"`. `SortOrder::CompletedDate` (the "sort by completed" config
value from the bug report) is one of the missing variants.

**Fix:** Delete lines 93-108 (the entire `if pane.sort_order != SortOrder::FileOrder` block).
The `filter:` prefix on the filter part should also change to bare filter string to match
the target format "Pane 3 | @work +CTRC" (see Q4).

---

## Q3 — Per-pane sort/group config data structure

All sort/group state lives in the `Pane` struct (`crates/todotxt-tui/src/state.rs` lines 32-57):

```rust
pub struct Pane {
    pub id: usize,
    pub display_rows: Vec<DisplayRow>,
    pub selected: usize,
    pub filter_query: String,       // line 44
    pub sort_order: SortOrder,      // line 49
    pub grouping: bool,             // line 52
    pub group_by: GroupByCategory,  // line 56
    pub label: String,              // line 57 (approx)
    pub label_selected: bool,
}
```

**Per-pane, not global.** The global `App` fields `sort_order`, `grouping`, `group_by` are
only used by the single-pane path; when multi-pane is active these are synced FROM the
active pane before calling `rebuild_display_indices` (app.rs:3154-3159).

**Config serialization:** `PaneConfig` (config.rs:115-123) stores `sort: PaneSort` where
`PaneSort` has only 4 variants (Priority, DueDate, Alphabetical, FileOrder). Additional
runtime `SortOrder` variants (CompletedDate etc.) are reachable via keyboard cycling but
serialize as `FileOrder` via the `_ => PaneSort::FileOrder` fallback in
`PaneSort::from_sort_order` (config.rs:104).

---

## Q4 — Pane label and filter string location for header rendering

Both fields are directly on the `pane: &Pane` argument already passed to `PaneList::render`:

| Field | Type | Location |
|---|---|---|
| `pane.label` | `String` | `state.rs` line ~57 |
| `pane.filter_query` | `String` | `state.rs` line 44 |

**Target header format** (from CONTEXT.md):
- No filter: `"▶ Pane 3"` (active) / `"  Pane 3"` (inactive)
- With filter: `"▶ Pane 3 | @work +CTRC"` — raw filter string, no "filter:" prefix

The current code already uses `pane.filter_query.trim()` for the filter part; only the
`"filter: "` prefix string needs to change to bare value, and the sort block removed.

---

## Q5 — Existing tests for multi-pane sort/group

**No tests exist for multi-pane sort+group combination.** Confirmed by searching
`app.rs` for `group.*pane`, `sort.*pane`, `rebuild_all_panes`, `grouping.*pane`.

| Test | File | What It Covers | Relevant? |
|---|---|---|---|
| `startup_populates_non_active_panes` | app.rs:5830 | `rebuild_all_panes` populates both panes | Population only, no sort/group |
| `test_pane_selection_independence` | app.rs:5753 | Each pane has independent cursor | Selection only |
| `group_key_for_groups_by_correct_field_per_variant` | app.rs:6441 | `group_key_for()` function logic | Unit test, not integration |
| `rebuild_display_indices_does_not_clear_selected_tasks` | app.rs:5385 | Single-pane rebuild | Single-pane only |

**New tests needed (two):**
1. Multi-pane sort+group: set up 2 panes with same tasks, pane 1 has
   `sort=completed, grouping=true, group_by=priority` — assert group headers appear
   exactly once each (no duplicates) after `rebuild_all_panes()`.
2. Pane header: assert `PaneList` header title is `"▶ Label"` with no sort fragment, and
   `"▶ Label | filter"` when filter is set.

---

## Fix Plan

### Fix 1 — Add secondary group-key sort in multi-pane paths (2 locations)

**`rebuild_visible_rows` (app.rs ~line 762):** after the primary sort, before the
grouping loop:

```rust
if pane.grouping && !filtered_tasks.is_empty() {
    // ← INSERT: secondary stable-sort by group key (mirrors rebuild_display_indices)
    filtered_tasks.sort_by(|(_, a), (_, b)| {
        group_key_for(a, &pane.group_by).cmp(&group_key_for(b, &pane.group_by))
    });
    // existing grouping loop follows unchanged
```

**`rebuild_all_panes` (app.rs ~line 830):** same insertion inside the per-pane `if grouping` block, after the primary sort on `filtered`:

```rust
if grouping && !filtered.is_empty() {
    // ← INSERT: secondary stable-sort by group key
    filtered.sort_by(|(_, a), (_, b)| {
        group_key_for(a, &group_by).cmp(&group_key_for(b, &group_by))
    });
    // existing grouping loop follows unchanged
```

### Fix 2 — Remove sort indicator from pane header (1 location)

**`pane_list.rs` lines ~93-108:** delete the `if pane.sort_order != SortOrder::FileOrder`
block entirely.

**`pane_list.rs` line ~89:** change `format!("filter: {}", filter_display)` to
`filter_display.to_string()` (drop the "filter: " prefix to match target format
"Pane 3 | @work +CTRC").

The `SortOrder` import at the top of `pane_list.rs` may become unused — remove if so.

---

## Sources

All findings verified directly in source — no external lookups required.

| File | Lines | Finding |
|---|---|---|
| `crates/todotxt-tui/src/app.rs` | 3081-3131 | Single-pane `rebuild_display_indices` with secondary sort |
| `crates/todotxt-tui/src/app.rs` | 738-793 | `rebuild_visible_rows` — missing secondary sort |
| `crates/todotxt-tui/src/app.rs` | 797-856 | `rebuild_all_panes` — missing secondary sort |
| `crates/todotxt-tui/src/app.rs` | 4482-4493 | `cycle_sort` — all 8 `SortOrder` variants in rotation |
| `crates/todotxt-tui/src/components/pane_list.rs` | 49-110 | Header build logic + `_ => "unknown"` |
| `crates/todotxt-tui/src/state.rs` | 32-80 | `Pane` struct fields |
| `crates/todotxt-tui/src/config.rs` | 80-104 | `PaneSort` enum + `from_sort_order` fallback |
| `crates/todotxt-core/src/sort.rs` | 10-26 | All 8 `SortOrder` variants |

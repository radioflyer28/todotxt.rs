---
phase: 260508-dbv-fix-multi-pane-sort-group-conflict-and-r
reviewed: 2026-05-08T00:00:00Z
depth: quick
files_reviewed: 2
files_reviewed_list:
  - crates/todotxt-tui/src/app.rs
  - crates/todotxt-tui/src/components/pane_list.rs
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
---

# Code Review — Multi-Pane Sort/Group Conflict Fix

**Reviewed:** 2026-05-08  
**Depth:** quick (targeted standard for focused questions)  
**Files Reviewed:** 2  
**Status:** issues_found

## Summary

Reviewed `app.rs` (secondary sort insertions in `rebuild_visible_rows` / `rebuild_all_panes`) and `pane_list.rs` (`build_pane_title` header logic). The sort placement and grouping logic are correct. One warning-level panic risk exists in `build_pane_title`'s filter truncation — unrelated to the main fix but present in the changed file.

---

## Warnings

### WR-01: Byte-index slice on UTF-8 string in `build_pane_title` can panic

**File:** `crates/todotxt-tui/src/components/pane_list.rs:40`

**Issue:**  
`&trimmed_filter[..17]` is a **byte-offset** slice, not a character-count slice. `trimmed_filter.len() > 20` also measures bytes. If `filter_query` contains multi-byte UTF-8 characters (e.g. Japanese, emoji, accented letters), byte 17 may fall in the middle of a multi-byte sequence and Rust will panic with `byte index 17 is not a char boundary`.

The safe `truncate_for_width` helper already exists on the same struct and handles this correctly via `.chars().take()`. This truncation bypass is inconsistent with it.

**Fix:**
```rust
let filter_display = if trimmed_filter.chars().count() > 20 {
    let truncated: String = trimmed_filter.chars().take(17).collect();
    format!("{}…", truncated)
} else {
    trimmed_filter.to_string()
};
```

---

## Focused-Question Answers

### 1. Secondary `sort_by` placement and stability

**Correct in both sites.**

In `rebuild_visible_rows` (lines 758–769):
1. Primary sort applied if `sort_order != FileOrder` → `filtered_tasks.sort_by(...)` 
2. Secondary group-key sort applied if `grouping` → `filtered_tasks.sort_by(...)` using `group_key_for(...).cmp(...)`
3. Grouping loop emits headers iterating `filtered_tasks`

In `rebuild_all_panes` (lines 826–846): identical order.

**Stability:** Rust's `slice::sort_by` is guaranteed stable (adaptive merge sort). The primary-sort relative order is preserved within groups after the secondary sort. No issue.

**Panic risk from `partial_cmp`:** None. `SortOrder::compare()` in `todotxt-core/src/sort.rs` uses only `Option<char>.cmp`, `Option<NaiveDate>.cmp`, and `String.cmp` — all total-order comparisons that return `Ordering` directly. No floats, no `partial_cmp`, no `unwrap_or`.

### 2. `build_pane_title` separator correctness

**No spurious ` | ` separator.** `header_parts.join(" | ")` only inserts the separator *between* elements. With one element (label but no filter), the output is just the label string — no separator. The empty-parts fallback at lines 47–49 correctly handles the case where neither label nor filter produces content. Filter is guarded by `!trimmed_filter.is_empty()`, so an empty `filter_query` never contributes a part.

### 3. `SortOrder` import / dead-code warnings

**No issue.** `SortOrder` is actively used in `app.rs` at lines 14, 92, 450, 758, 826, 3111, 3863, and 4499–4506. In `pane_list.rs`, `use todotxt_core::SortOrder;` lives inside the `#[cfg(test)]` module (line 210 area) and is consumed by the two unit tests — no unused-import warning. Existing `#[allow(dead_code)]` attributes on `PaneList` and `render` are intentional and pre-existing.

---

_Reviewed: 2026-05-08_  
_Reviewer: gsd-code-reviewer (GitHub Copilot / Claude Sonnet 4.6)_  
_Depth: quick (with targeted file reads for focused questions)_

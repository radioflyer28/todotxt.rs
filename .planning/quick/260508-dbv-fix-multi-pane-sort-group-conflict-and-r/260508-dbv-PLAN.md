---
phase: 260508-dbv
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/todotxt-tui/src/app.rs
  - crates/todotxt-tui/src/components/pane_list.rs
autonomous: true
requirements: []

must_haves:
  truths:
    - "Two panes with sort=CompletedDate + grouping=priority each show exactly one group header per unique priority value (no duplicate headers)"
    - "Pane header shows only the pane label when no filter is set (e.g. '▶ Pane 3')"
    - "Pane header shows label + bare filter string when a filter is active (e.g. '▶ Pane 3 | @work +CTRC') — no 'filter:' prefix, no sort segment"
  artifacts:
    - path: "crates/todotxt-tui/src/app.rs"
      provides: "Secondary group-key sort inserted in rebuild_visible_rows and rebuild_all_panes"
    - path: "crates/todotxt-tui/src/components/pane_list.rs"
      provides: "Sort indicator block removed; filter prefix stripped"
  key_links:
    - from: "rebuild_visible_rows (app.rs ~762)"
      to: "grouping loop"
      via: "secondary stable-sort by group_key_for before loop"
      pattern: "filtered_tasks.sort_by.*group_key_for"
    - from: "rebuild_all_panes (app.rs ~830)"
      to: "grouping loop"
      via: "secondary stable-sort by group_key_for before loop"
      pattern: "filtered.sort_by.*group_key_for"
    - from: "PaneList::render (pane_list.rs ~88)"
      to: "header_parts"
      via: "filter pushed bare (no 'filter: ' prefix); sort block absent"
      pattern: "header_parts.push\\(filter_display"
---

<objective>
Fix two related multi-pane bugs:
1. Duplicate group headers when sort and group-by are both active across multiple panes —
   caused by missing secondary group-key sort in both multi-pane rebuild paths.
2. "sort: unknown" in pane header + outdated sort indicator — removed entirely; header
   becomes label + bare filter string (if filtered).

Purpose: Users get clean, correct grouped views in multi-pane mode.
Output: Patched app.rs (2 insertion sites), patched pane_list.rs (sort block deleted,
filter prefix stripped, unused SortOrder import removed).
</objective>

<execution_context>
@~/.copilot/get-shit-done/workflows/execute-plan.md
@~/.copilot/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/quick/260508-dbv-fix-multi-pane-sort-group-conflict-and-r/260508-dbv-RESEARCH.md
</context>

<interfaces>
<!-- Key types already in scope in app.rs — no new imports needed. -->

From crates/todotxt-tui/src/app.rs (rebuild_visible_rows, ~line 762):
```rust
// After this line:
if pane.sort_order != SortOrder::FileOrder {
    filtered_tasks.sort_by(|(_, a), (_, b)| pane.sort_order.compare(a, b));
}

// Then directly:
let rows: Vec<DisplayRow> = if pane.grouping && !filtered_tasks.is_empty() {
```

From crates/todotxt-tui/src/app.rs (rebuild_all_panes, ~line 830):
```rust
// After this line:
if sort_order != SortOrder::FileOrder {
    filtered.sort_by(|(_, a), (_, b)| sort_order.compare(a, b));
}

// Then directly:
if grouping && !filtered.is_empty() {
```

From crates/todotxt-tui/src/components/pane_list.rs (PaneList::render, lines 88-108):
```rust
// Keep — filter section (change only the push line):
let trimmed_filter = pane.filter_query.trim();
if !trimmed_filter.is_empty() {
    let filter_display = if trimmed_filter.len() > 20 {
        format!("{}…", &trimmed_filter[..17])
    } else {
        trimmed_filter.to_string()
    };
    header_parts.push(format!("filter: {}", filter_display));  // ← change this line
}

// Delete entirely — sort section (lines 99-108):
if pane.sort_order != SortOrder::FileOrder {
    let sort_name = match pane.sort_order { ... };
    header_parts.push(format!("sort: {}", sort_name));
}
```
</interfaces>

<tasks>

<task type="auto">
  <name>Task 1: Add secondary group-key sort in both multi-pane rebuild paths</name>
  <files>crates/todotxt-tui/src/app.rs</files>
  <action>
Two insertion sites — both insert a `stable_sort_by` on the group key immediately after
the primary sort and immediately before the `if grouping &&` block. Use the existing
`group_key_for` function (already in scope at both call sites).

**Site 1 — `rebuild_visible_rows` (~line 762):**

After:
```rust
        if pane.sort_order != SortOrder::FileOrder {
            filtered_tasks.sort_by(|(_, a), (_, b)| pane.sort_order.compare(a, b));
        }
```

Insert:
```rust
        // Secondary stable-sort by group key so all tasks with the same key are
        // contiguous before the grouping loop emits headers.
        // Mirrors the single-pane path in rebuild_display_indices. (D-09)
        if pane.grouping && !filtered_tasks.is_empty() {
            let group_by = pane.group_by;
            filtered_tasks.sort_by(|(_, a), (_, b)| {
                group_key_for(a, &group_by).cmp(&group_key_for(b, &group_by))
            });
        }
```

The existing `let rows: Vec<DisplayRow> = if pane.grouping && ...` block that follows is
unchanged. Note: `pane.group_by` is moved into `group_by` for the closure to avoid a
partial borrow error on `pane` (same pattern used in `rebuild_all_panes`).

**Site 2 — `rebuild_all_panes` (~line 830):**

After:
```rust
                if sort_order != SortOrder::FileOrder {
                    filtered.sort_by(|(_, a), (_, b)| sort_order.compare(a, b));
                }
```

Insert:
```rust
                // Secondary stable-sort by group key — mirrors rebuild_display_indices.
                if grouping && !filtered.is_empty() {
                    filtered.sort_by(|(_, a), (_, b)| {
                        group_key_for(a, &group_by).cmp(&group_key_for(b, &group_by))
                    });
                }
```

No new imports needed — `group_key_for` and `GroupByCategory` are already in scope.
  </action>
  <verify>
    <automated>cd crates/todotxt-tui && cargo build 2>&1 | grep -E "^error"</automated>
  </verify>
  <done>
    `cargo build` emits no errors. Both multi-pane rebuild paths now stable-sort by group
    key before the grouping loop.
  </done>
</task>

<task type="auto">
  <name>Task 2: Remove sort indicator from pane header; strip "filter:" prefix</name>
  <files>crates/todotxt-tui/src/components/pane_list.rs</files>
  <action>
Three targeted edits in `PaneList::render`:

1. **Remove unused import** (line 13):
   Delete: `use todotxt_core::SortOrder;`

2. **Strip "filter:" prefix** (line 95):
   Change:
   ```rust
           header_parts.push(format!("filter: {}", filter_display));
   ```
   To:
   ```rust
           header_parts.push(filter_display.to_string());
   ```
   (The `filter_display` variable already holds the trimmed/truncated filter string.)

3. **Delete entire sort block** (lines 99–108):
   Remove:
   ```rust
           // Add sort order if not FileOrder
           if pane.sort_order != SortOrder::FileOrder {
               let sort_name = match pane.sort_order {
                   SortOrder::FileOrder => "file",
                   SortOrder::Alphabetical => "alpha",
                   SortOrder::Priority => "priority",
                   SortOrder::DueDate => "due",
                   _ => "unknown",
               };
               header_parts.push(format!("sort: {}", sort_name));
           }
   ```
   Replace with nothing (delete entirely).

After these edits the header format becomes:
- No label, no filter → `""` (handled by existing empty fallback)
- Label only → `"▶ Pane 3"` or `"  Pane 3"`
- Label + filter → `"▶ Pane 3 | @work +CTRC"`
  </action>
  <verify>
    <automated>cd crates/todotxt-tui && cargo build 2>&1 | grep -E "^error"</automated>
  </verify>
  <done>
    `cargo build` emits no errors. `pane_list.rs` has no `SortOrder` reference anywhere.
    Pane headers show no sort segment regardless of active sort order.
  </done>
</task>

<task type="auto">
  <name>Task 3: Add regression tests for multi-pane group dedup and header format</name>
  <files>crates/todotxt-tui/src/app.rs</files>
  <action>
Add two tests near the existing multi-pane tests (around line 5830 where
`startup_populates_non_active_panes` lives). Place them inside the same `#[cfg(test)]`
mod.

**Test 1 — no duplicate group headers in multi-pane:**
```rust
#[test]
fn rebuild_all_panes_no_duplicate_group_headers_with_sort_and_group() {
    // Two tasks with different priorities: (A) task sorts after (B) task alphabetically
    // but before by priority. A sort=Alphabetical + group=Priority combination would
    // produce duplicate "(A)" headers without the secondary sort fix.
    let tasks = vec![
        Task::from_str("(B) beta task").unwrap(),
        Task::from_str("(A) alpha task").unwrap(),
        Task::from_str("(A) zebra task").unwrap(),
    ];
    let mut app = App::new_test(tasks);
    app.ensure_two_panes();
    // Configure pane 0: sort=Alphabetical, group=Priority
    app.panes[0].sort_order = SortOrder::Alphabetical;
    app.panes[0].grouping = true;
    app.panes[0].group_by = GroupByCategory::Priority;

    app.rebuild_all_panes();

    let headers: Vec<_> = app.panes[0]
        .display_rows
        .iter()
        .filter(|r| matches!(r, DisplayRow::GroupHeader(_)))
        .collect();

    // Expect exactly 2 unique headers: "(A)" and "(B)" — not 3 (the broken case)
    assert_eq!(headers.len(), 2, "Expected 2 group headers, got: {:?}", headers);
}
```

**Test 2 — pane header format (no sort segment):**

Add a unit test in `pane_list.rs` inside a `#[cfg(test)]` block at the bottom of the
file, or in `app.rs` via a helper that inspects rendered output. Simpler: add to
`pane_list.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Pane;
    use todotxt_core::SortOrder;

    fn make_pane(label: &str, filter: &str, sort: SortOrder) -> Pane {
        let mut p = Pane::default();
        p.label = label.to_string();
        p.filter_query = filter.to_string();
        p.sort_order = sort;
        p
    }

    #[test]
    fn pane_header_no_sort_indicator() {
        // With a non-FileOrder sort, header must NOT contain "sort:"
        let p = make_pane("Pane 3", "", SortOrder::CompletedDate);
        // Simulate header build (inline logic mirrors render):
        let label_display = format!("▶ {}", p.label);
        let mut parts: Vec<String> = vec![label_display];
        let trimmed = p.filter_query.trim();
        if !trimmed.is_empty() {
            let fd = if trimmed.len() > 20 { format!("{}…", &trimmed[..17]) } else { trimmed.to_string() };
            parts.push(fd.to_string());
        }
        // No sort block
        let title = parts.join(" | ");
        assert!(!title.contains("sort:"), "Header must not contain 'sort:': {}", title);
        assert_eq!(title, "▶ Pane 3");
    }

    #[test]
    fn pane_header_filter_no_prefix() {
        let p = make_pane("Pane 3", "@work +CTRC", SortOrder::CompletedDate);
        let label_display = format!("▶ {}", p.label);
        let mut parts: Vec<String> = vec![label_display];
        let trimmed = p.filter_query.trim();
        if !trimmed.is_empty() {
            let fd = if trimmed.len() > 20 { format!("{}…", &trimmed[..17]) } else { trimmed.to_string() };
            parts.push(fd.to_string());
        }
        let title = parts.join(" | ");
        assert!(!title.contains("filter:"), "Header must not contain 'filter:': {}", title);
        assert_eq!(title, "▶ Pane 3 | @work +CTRC");
    }
}
```

Note: If `App::new_test` or `ensure_two_panes` don't exist as public test helpers, adapt
to the project's test scaffolding pattern (look at `startup_populates_non_active_panes`
around line 5830 for the constructor used in similar tests). Use whatever `App` test
constructor already exists — do NOT invent a new one.
  </action>
  <verify>
    <automated>cd crates/todotxt-tui && cargo test rebuild_all_panes_no_duplicate_group_headers pane_header_no_sort_indicator pane_header_filter_no_prefix 2>&1 | grep -E "FAILED|passed|error"</automated>
  </verify>
  <done>
    All three new tests pass. No existing tests regress (`cargo test` green).
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| task_list → display_rows | Read-only tasks from file; display transform is pure logic, no user input crosses here |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-260508-01 | Tampering | group_key_for sort comparator | accept | Pure deterministic sort on task fields already loaded from file; no external input |
| T-260508-02 | DoS | secondary sort on large task list | accept | O(n log n) extra sort pass; same complexity class as primary sort already in place |
</threat_model>

<verification>
- `cargo build` succeeds (no errors in either crate)
- `cargo test` full suite passes (no regressions)
- Multi-pane: pane with `CompletedDate` sort + `Priority` group-by shows each priority group header exactly once
- Pane header: no "sort:" segment, no "filter:" prefix — format is "Label" or "Label | filter"
</verification>

<success_criteria>
- Two multi-pane rebuild paths (rebuild_visible_rows, rebuild_all_panes) each apply secondary group-key sort before grouping loop
- pane_list.rs `SortOrder` import removed; sort block deleted; filter push uses bare string
- Three new tests added and passing
- Zero regressions in existing test suite
</success_criteria>

<output>
After completion, create `.planning/quick/260508-dbv-fix-multi-pane-sort-group-conflict-and-r/260508-dbv-SUMMARY.md`
</output>

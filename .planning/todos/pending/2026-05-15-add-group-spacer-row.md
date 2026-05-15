---
created: 2026-05-15T00:00:00
title: Add spacer row between TUI group headers
area: tui
files:
  - crates/todotxt-tui/src/state.rs:10-12
  - crates/todotxt-tui/src/app.rs:755-770
  - crates/todotxt-tui/src/app.rs:820-835
  - crates/todotxt-tui/src/app.rs:1071-1145
  - crates/todotxt-tui/src/app.rs:3440-3472
  - crates/todotxt-tui/src/components/pane_list.rs:115-135
---

## Problem

When grouping is enabled, group headers are placed directly adjacent to the last row of the
previous group. With many tasks this creates a "wall of text" effect where group boundaries
are hard to scan. A small visual gap between groups would improve readability without wasting
too much terminal real estate.

A true half-line is not achievable in ratatui's `List` widget (which renders whole rows), but
a blank spacer row before each group header (except the first) creates a similar effect and
is the standard terminal TUI idiom.

## Solution

### 1. Add `DisplayRow::GroupSpacer` variant (`state.rs`)

```rust
pub enum DisplayRow {
    Task(usize),
    GroupHeader(String),
    GroupSpacer,          // blank gap row before a group header (except the first)
}
```

### 2. Insert spacer before each non-first group header (`app.rs` ~line 762)

```rust
for (source_index, task) in &filtered_tasks {
    let key = group_key_for(task, &group_by);
    if last_key.as_deref() != Some(&key) {
        if last_key.is_some() {                          // not the very first group
            display_rows.push(DisplayRow::GroupSpacer);
        }
        display_rows.push(DisplayRow::GroupHeader(key.clone()));
        last_key = Some(key);
    }
    display_rows.push(DisplayRow::Task(*source_index));
}
```

The same change applies to the second grouping site around line 829.

### 3. Skip `GroupSpacer` during cursor navigation

Every `matches!(pane.display_rows[…], DisplayRow::GroupHeader(_))` guard that skips headers
during Up/Down must also skip `GroupSpacer`. Update all such guards in `app.rs` (lines ~1071,
1093, 1117, 1139, 3440, 3452, 3472) to:

```rust
matches!(pane.display_rows[idx], DisplayRow::GroupHeader(_) | DisplayRow::GroupSpacer)
```

### 4. Render spacer as a blank line (`pane_list.rs`)

In the `ListItem` build loop, add an arm for `DisplayRow::GroupSpacer`:

```rust
DisplayRow::GroupSpacer => {
    ListItem::new("")
}
```

The spacer row is intentionally un-styled (plain blank line) so it reads as whitespace, not
a UI element.

### 5. Handle `GroupSpacer` in any match-exhaustive arms

Search for existing `match row { DisplayRow::GroupHeader … DisplayRow::Task … }` patterns
(e.g. `app.rs:3639`, `app.rs:3897`) and add a `DisplayRow::GroupSpacer => …` arm returning
the appropriate default (empty `ListItem` or `None`).

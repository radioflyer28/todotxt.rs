---
created: 2026-05-15T00:00:00
title: Add OR operator support to filter engine
area: tui
resolves_phase: 46
files:
  - crates/todotxt-core/src/filter.rs:1-210
  - crates/todotxt-tui/src/app.rs:728
  - crates/todotxt-tui/src/app.rs:802
---

## Problem

All filter tokens are currently AND-combined. There is no way to express OR logic, which
makes common queries like "priority A or B" or "work context or home context" impossible in
a single pane. Users must open multiple panes as a workaround.

## Solution

### Syntax design

Use `|` as the OR separator *within* a single space-delimited token. Spaces remain the AND
separator, keeping the existing contract intact and requiring no breaking changes.

Examples:
```
(A)|(B)           → priority A OR priority B
@work|@home       → either context
(A)|(B) -DONE     → (A or B) AND not completed
+proj due:today   → existing AND behavior unchanged
```

A leading `-` before a pipe group negates the whole group:
```
-@work|@home      → NOT in work AND NOT in home (each side negated separately, DeMorgan)
```
...or optionally treat `-(@work|@home)` as a single negated OR group (simpler but requires
parenthesis parsing — defer to v2 if complexity is high).

### 1. Add `Or` variant to `FilterTerm` (`filter.rs`)

```rust
pub enum FilterTerm {
    // …existing variants…
    Or(Vec<FilterTerm>),   // passes if ANY inner term passes
}
```

### 2. Update `from_query` to split on `|` before dispatching (`filter.rs`)

In `from_query`, before the existing token dispatch, split on `|`:

```rust
let parts: Vec<&str> = token.split('|').collect();
if parts.len() > 1 {
    let inner: Vec<FilterTerm> = parts
        .iter()
        .map(|part| Self::parse_single_token(part))
        .collect();
    return FilterTerm::Or(inner);
}
// single token — fall through to existing logic (extracted into parse_single_token)
Self::parse_single_token(token)
```

Extract the current giant `map` closure body into a private
`fn parse_single_token(token: &str) -> FilterTerm` helper — no logic changes there.

### 3. Evaluate `Or` in `matches_with_date` (`filter.rs`)

```rust
FilterTerm::Or(inner) => inner.iter().any(|t| Self::eval_term(t, task, &raw, today)),
```

Extract the per-term evaluation into a private `fn eval_term(term, task, raw, today) -> bool`
helper called by both the AND loop and the `Or` arm.

### 4. No changes needed in the TUI layer

`Filter::from_query` is called at lines 728 and 802 in `app.rs` — the new `Or` variant is
transparent at those call sites.

### 5. Tests to add (`filter.rs` test module)

- `or_two_priorities` — `(A)|(B)` matches (A) task and (B) task, not (C)
- `or_two_contexts` — `@work|@home` matches both contexts, not `@gym`
- `or_and_combined` — `(A)|(B) @work` matches (A)@work and (B)@work, not (A)@home
- `or_single_part` — `(A)|` with empty second part degrades gracefully (treat empty as no-op
  or filter it out)
- `existing_and_unchanged` — spot-check that `@work +proj` still AND-combines

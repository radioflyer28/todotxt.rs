---
id: SEED-014
status: dormant
planted: 2026-05-04
planted_during: v1.5 complete / preparing v1.6
trigger_when: next milestone (v1.6)
scope: Medium
---

# SEED-014: Autocomplete coverage gaps — edit mode, filter input, and Ctrl-R incremental narrowing

## Why This Matters

Three distinct gaps reduce the usefulness of the existing `@`/`+` autocomplete system:

1. **Edit mode gap**: `update_autocomplete()` is gated to `AppMode::Adding | AppMode::Editing` — Editing mode IS included in code but user reports it doesn't trigger. Needs investigation and fix.

2. **Filter input gap**: Typing `@work` or `+myproject` in the filter input gets no suggestions. This is the most common filter pattern, and every context/project is already in memory. `handle_filtering_key` never calls `update_autocomplete`.

3. **Incremental narrowing (Ctrl-R style)**: The current popup shows the full pre-filtered list and requires `↑/↓` to navigate. There's no way to narrow the popup by continuing to type. Ctrl-R shell behavior — where each keystroke narrows the match list — would dramatically speed up selection in large task lists with many contexts or projects.

## When to Surface

**Trigger:** Next milestone (v1.6).

Matches when:
- TUI editing or filtering UX improvements are in scope
- Autocomplete / discoverability features are being extended

## Scope Estimate

**Medium** — Three work items, each independently shippable:

### 1. Confirm and fix autocomplete in Editing mode

`update_autocomplete()` already includes `AppMode::Editing { .. }` in its match arm, but there may be a call-site issue where it's not called after certain key events in edit mode. Audit `handle_editor_key` to ensure `update_autocomplete()` is called on every character input in Editing mode as it is in Adding mode.

### 2. Autocomplete in filter input

`handle_filtering_key()` forwards character keys to `state.editor.input(key)` but never calls `update_autocomplete()`. Adding autocomplete here requires:
- Calling `update_autocomplete()` after each character key in filter mode
- `update_autocomplete()` reading from `filter_state.editor` instead of `self.editor` when in Filtering mode
- Autocomplete `accept_completion()` writing back to `filter_state.editor`

### 3. Ctrl-R incremental narrowing in popup

Currently the popup pre-filters by prefix on creation and updates on each keystroke already (via `update_autocomplete`). "Incremental narrowing" means:
- When the popup is visible and focused, further characters typed narrow the visible list
- The typed characters form a sub-filter on top of the current prefix
- This could reuse the existing `prefix` field — `update_autocomplete` already re-filters on each keypress

This may already partially work via the `update_autocomplete` call after each key — needs testing. If not, the gap is that once the popup is `focused` (navigating with `↑/↓`), further character input doesn't update `prefix`.

## Breadcrumbs

| File | Notes |
|------|-------|
| `crates/todotxt-tui/src/app.rs` line 1848–1851 | `update_autocomplete()` mode guard — verify Editing is truly reaching the token branch |
| `crates/todotxt-tui/src/app.rs` line 1727–1833 | `handle_editor_key()` — audit each arm's `update_autocomplete()` call |
| `crates/todotxt-tui/src/app.rs` line 1528–1600 | `handle_filtering_key()` — add `update_autocomplete()` call after char input |
| `crates/todotxt-tui/src/app.rs` line 1597 | Filter editor input: `state.editor.input(key)` — call site to add autocomplete trigger |
| `crates/todotxt-tui/src/app.rs` line 1974–2000 | `accept_completion()` — needs to handle filter editor as target, not just `self.editor` |
| `crates/todotxt-tui/src/state.rs` line 107–129 | `AutocompleteState` — `prefix` field drives narrowing; already updated on each keypress |

## Notes

For filter autocomplete, `accept_completion()` currently inserts into `self.editor` (the task editor). A `target_editor` enum or a closure would let it target `filter_state.editor` instead. Alternatively, route the accept via a flag and handle separately in `handle_filtering_key`.

For Ctrl-R narrowing: the shell mental model is that Ctrl-R opens a reverse-search popup and each character narrows matches. In the TUI context, this could mean:
- While in `Filtering` mode, `Ctrl+R` opens a history+autocomplete popup (related to SEED-011)
- While the autocomplete popup is open, typing continues narrowing rather than requiring you to back out and retype

Cross-reference: SEED-011 (filter history) and SEED-013 (fix `+` bug) are related — fix the bug first, then extend coverage.

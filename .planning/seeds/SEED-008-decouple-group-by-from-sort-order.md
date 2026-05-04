---
id: SEED-008
status: dormant
planted: 2026-05-04
planted_during: v1.5 complete / preparing v1.6
trigger_when: next milestone (v1.6)
scope: Medium
---

# SEED-008: Decouple group-by category from sort order in the TUI

## Why This Matters

A common power-user workflow is: *group by project, sorted by due date within each group*. Today this is impossible — the same `sort_order` field drives both the group bucket and the intra-group sort. Grouping by `Project` gives you project buckets, but the tasks inside are also sorted by the project field, not by due date or priority.

The coupling is an accidental constraint from how `group_key_for(task, &sort_order)` was originally wired up, not a deliberate design decision.

## When to Surface

**Trigger:** Next milestone (v1.6).

This seed should be presented during `/gsd-new-milestone` when any of these are in scope:
- TUI pane or view feature work
- Grouping / sorting improvements
- Quality-of-life / power-user workflow polish

## Scope Estimate

**Medium** — Two orthogonal axes need to be separated:

| Axis | Current state | Target state |
|------|--------------|--------------|
| **Group-by category** | Derived from `sort_order` via `group_key_for` | New `group_by: GroupBy` field on `Pane` |
| **Intra-group sort** | Same `sort_order` (conflated) | Existing `sort_order` field, now used only for within-group ordering |

Key work items:
- Add a `GroupBy` enum (likely mirrors `SortOrder` variants: Priority, Project, Context, DueDate, Alphabetical, FileOrder) to `todotxt-core` or `todotxt-tui`
- Add `group_by: GroupBy` field to `Pane` struct (`state.rs`)
- Change `group_key_for` signature to accept `&GroupBy` instead of `&SortOrder`
- Wire `'G'` (capital, or configurable) to cycle `group_by` on the active pane — proposed default binding `group_cycle`
- `'o'` continues to cycle `sort_order` (intra-group sort) as today
- Update `rebuild_display_indices` / `rebuild_all_panes` to sort first by `group_by` key, then by `sort_order` within each bucket
- Add `group_by` to `PaneConfig` in `config.rs` for config-file definition
- Update status bar / help overlay to show both active group-by and sort

## Breadcrumbs

Relevant code in the current codebase:

| File | Notes |
|------|-------|
| `crates/todotxt-tui/src/app.rs` line 3869–3906 | `group_key_for(task, &SortOrder)` — the function to refactor to accept `GroupBy` |
| `crates/todotxt-tui/src/app.rs` line 458–476 | Grouping block in `rebuild_display_indices` — passes `pane.sort_order` to `group_key_for` |
| `crates/todotxt-tui/src/app.rs` line 514–528 | Same pattern in `rebuild_all_panes` |
| `crates/todotxt-tui/src/app.rs` line 877 | `sort_cycle` action handler — `'o'` key; `group_cycle` would be a parallel new action |
| `crates/todotxt-tui/src/app.rs` line 974 | `group_toggle` handler — `'g'` key toggles grouping on/off |
| `crates/todotxt-tui/src/config.rs` line 385–386 | Default keymap: `sort_cycle` = `'o'`, `group_toggle` = `'g'`; add `group_cycle` = `'G'` here |
| `crates/todotxt-core/src/sort.rs` line 10–27 | `SortOrder` enum — `GroupBy` variants would largely mirror this |
| `crates/todotxt-tui/src/state.rs` line 26–56 | `Pane` struct — add `group_by: GroupBy` field here |

## Notes

Proposed key scheme (all remappable via `[keymap]` config):
- `g` — toggle grouping on/off (existing `group_toggle`)
- `G` (Shift+g) — cycle through group-by categories (new `group_cycle`)
- `o` — cycle intra-group sort order (existing `sort_cycle`, unchanged)

The `GroupBy` enum can start as a subset of `SortOrder` (Priority, Project, Context, DueDate) since those produce meaningful bucket labels. `FileOrder` / `Alphabetical` are edge cases (all-in-one-bucket or per-first-letter).

Consider whether `GroupBy` lives in `todotxt-core` (reusable) or `todotxt-tui` (simpler). Given the CLI doesn't do grouping, `todotxt-tui` is fine for now.

Also consider: when SEED-007 (view state persistence) is implemented, `group_by` should be persisted alongside `sort_order`.

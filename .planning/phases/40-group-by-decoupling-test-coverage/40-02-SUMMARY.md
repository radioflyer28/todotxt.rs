---
phase: 40-group-by-decoupling-test-coverage
plan: 02
status: complete
commit: 2861696
---

# Plan 40-02 Summary: group_by_cycle action + status bar + help overlay

## What Was Built

Wired the `group_by_cycle` action and updated the status bar to display the active group-by category.

**New bindings (config.rs):**
- `group_toggle` default key changed from `'g'` → `'G'` (Shift+g) (D-09)
- `group_by_cycle` added with default `'g'` (D-08)

**New helpers (app.rs):**
- `cycle_group_by(GroupByCategory) -> GroupByCategory` — Priority→Project→Context→DueDate→Priority
- `group_by_name(GroupByCategory) -> &'static str` — returns "priority"/"project"/"context"/"duedate"

**Action handler (app.rs):**
- `group_by_cycle` arm in `handle_normal_key()` — guarded by `display_count > 0`, calls `cycle_group_by()` + `rebuild_and_reanchor()` (D-08)

**Status bar (app.rs):**
- 4-tuple extraction now includes `pane_group_by: GroupByCategory`
- Grouping indicator changed from `"| group: on"` → `"| grp:{category}"` using `group_by_name()` (D-12, D-13)
- Right hint string updated: `"G group | g grp-by"` replacing old `"g group"`

**Help overlay (app.rs):**
- View section includes `group_by_cycle` alongside `group_toggle`
- `action_labels` maps `group_by_cycle` → `"Cycle group-by"`

## Key Files Changed

- `crates/todotxt-tui/src/config.rs` — default_keymap bindings updated
- `crates/todotxt-tui/src/app.rs` — cycle_group_by, group_by_name, action handler, status bar, help overlay

## Verification Results

- `cargo test`: PASSED — 150 tests, 0 failures
- `grep group_by_cycle config.rs`: shows default binding 'g'
- `grep group_toggle config.rs`: shows 'G' (uppercase)
- `grep "grp:" app.rs`: status bar indicator present
- `grep group_by_cycle app.rs`: handler + help overlay entry present

## Deviations

None — implementation matched plan exactly.

## Self-Check: PASSED

---
id: SEED-015
status: dormant
planted: 2026-05-04
planted_during: v1.5 complete / preparing v1.6
trigger_when: next milestone (v1.6)
scope: Medium
---

# SEED-015: Numeric preset keys define full view presets, not just filters

## Why This Matters

Today `1`–`9` apply a filter query from `[presets.f1]` … `[presets.f9]` and nothing else. Every other dimension of view state — sort order, grouping, active pane, deferred toggle — has to be re-applied manually. Switching between distinct work modes (e.g., "daily review": priority sort + group on + `due:today`, vs "project focus": project sort + no group + `+myproject`) requires 3–4 separate keystrokes after the preset applies.

The fix is to expand the preset model: a preset still owns a `filter` (optional), but can also declare `sort`, `group`, `pane_focus`, and other view dimensions. Pressing `1`–`9` applies the full snapshot atomically.

## When to Surface

**Trigger:** Next milestone (v1.6).

Matches when:
- TUI preset or view configuration work is in scope
- View state persistence (SEED-007) is being implemented — the two features are complementary
- TUI quality-of-life / power-user workflow improvements are planned

## Scope Estimate

**Medium** — Two layers of work:

### Layer 1 — Extend `TuiPreset` in config

`TuiPreset` currently has one field:
```toml
[presets.f1]
filter = "@work"
```

Expand to optionally carry:
```toml
[presets.f1]
filter  = "@work"          # existing — unchanged
sort    = "priority"       # PaneSort variant
group   = true             # enable grouping
# group_by = "project"     # future: SEED-008 group-by category
```

All new fields are optional — absence means "don't change this dimension when applying the preset", so existing single-filter configs continue working without modification.

### Layer 2 — Update the preset apply handler

The `1`–`9` key handler currently sets `filter_query` and calls `rebuild_and_reanchor`. It would also:
- Apply `sort_order` if `preset.sort` is `Some`
- Apply `grouping` if `preset.group` is `Some`
- (Future) Apply `pane_focus` if `preset.pane` is `Some`

## Breadcrumbs

| File | Notes |
|------|-------|
| `crates/todotxt-tui/src/config.rs` line 31–35 | `TuiPreset` struct — add `sort`, `group` fields here |
| `crates/todotxt-tui/src/config.rs` line 37–65 | `PaneSort` enum — the type for `preset.sort` |
| `crates/todotxt-tui/src/app.rs` line 1006–1016 | `1`–`9` key handler — apply `sort_order` and `grouping` alongside filter |
| `crates/todotxt-tui/src/config.rs` line 484–520 | Default config template — add `sort` and `group` to the example preset blocks |
| `crates/todotxt-tui/src/app.rs` line 3407 | Help overlay preset line — update description text |

## Notes

**Backwards compatible by design** — `filter` remains optional on `TuiPreset`. All new fields are `Option<T>` with `None` meaning "no-op for this dimension". Existing configs with `filter`-only presets apply exactly as before.

**Relationship to other seeds:**
- SEED-007 (view state persistence) — when implementing, persisted state and preset-defined state need a clear precedence rule. Suggested: preset application overrides persisted state for the dimensions it specifies.
- SEED-008 (decouple group-by from sort) — when `group_by` exists as a separate concept, add it as a fourth optional preset field.
- SEED-011 (filter history) — applying a preset should not pollute the ad-hoc filter history (presets are intentional, not ephemeral queries).

**Future extension — pane focus:**  
`pane_focus: Option<usize>` (or a pane label string) could let a preset also bring a specific pane into focus. Useful for workspace-switching workflows (e.g., "1" → work pane, "2" → personal pane). This is lower priority than sort/group but fits naturally in the same struct.

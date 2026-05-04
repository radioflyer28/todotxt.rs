---
id: SEED-011
status: dormant
planted: 2026-05-04
planted_during: v1.5 complete / preparing v1.6
trigger_when: next milestone (v1.6)
scope: Medium
---

# SEED-011: Filter input history in the TUI

## Why This Matters

Power users type the same filter expressions repeatedly across sessions (`@work`, `+myproject due:today`, etc.). Every session starts with a blank filter input — the user re-types from memory. The TUI has saved presets (via `F` key) for long-term filters, but has no lightweight recently-used list for ad-hoc filters that aren't worth naming and saving permanently.

## When to Surface

**Trigger:** Next milestone (v1.6).

Matches when:
- TUI filter or search UX improvements are in scope
- Session productivity / power-user workflow features are planned

## Scope Estimate

**Medium** — Two interacting sub-features:

1. **Within-session history** — Ring buffer of filters used this session; `↑/↓` in the filter input cycles through them (like shell history). This is entirely in-memory, no persistence required.

2. **Cross-session history** — Persist the recent filter list to a sidecar file (e.g., alongside SEED-007 view state persistence). Optional but valuable.

The filter input already handles `↑/↓` keys — they currently cycle through named presets. History would layer underneath presets in the same key scheme, or use a separate key (e.g., `Ctrl+R` for reverse history search in the filter input).

The `ctrl-r` narrowing idea from SEED-014 (incremental autocomplete) applies here too — a history popup that narrows as you type would be ideal.

## Breadcrumbs

| File | Notes |
|------|-------|
| `crates/todotxt-tui/src/app.rs` line 1528–1600 | `handle_filtering_key()` — `↑/↓` already cycles presets; history slots in here |
| `crates/todotxt-tui/src/app.rs` line 921 | `AppMode::Filtering` entry — `f` key; could push to history when leaving this mode |
| `crates/todotxt-tui/src/app.rs` line 1540–1560 | `Enter` handler for filter — apply + potentially record history entry |
| `crates/todotxt-tui/src/state.rs` | `FilterState` or new `FilterHistory` struct would live here |
| `.planning/seeds/SEED-007-tui-view-state-persistence.md` | If cross-session history is wanted, the state persistence from SEED-007 is the right vehicle |

## Notes

Consider whether history and presets share `↑/↓` or are separately navigable. A clean separation:
- `↑/↓` = named presets (existing behavior)
- `Ctrl+R` or `Ctrl+H` = recent ad-hoc filter history popup

This avoids collision with the existing preset navigation and is consistent with the shell mental model users already have.

Dedup the history ring — if the same filter is typed twice, don't add a second entry.

---
plan: 18-03
phase: 18-validation-ship-readiness
status: complete
completed: "2026-04-24"
approved_by: human
---

# Plan 18-03: Human UAT Checkpoint — SUMMARY

## What Was Verified

Human walkthrough of all 4 TUI UAT areas against the live binary (cargo build -p todotxt-tui).

## UAT Outcome

**APPROVED** by user on 2026-04-24.

All 4 areas passed after iterative bug fixes discovered during UAT:

| Area | Scope | Result |
|------|-------|--------|
| 1 — Quick tasks | Add, edit (with cursor movement), done toggle, delete | PASS |
| 2 — Filter | `f` open, Esc cancel/restore, Enter confirm, preset navigation | PASS |
| 3 — Grouping/sort | `g` toggle, `o` sort cycle, contiguous group sort, subtle headers | PASS |
| 4 — Preset definition | `F` open, stable slot numbering, Enter save+apply, Ctrl+F toggle | PASS |

## Bugs Found and Fixed During UAT

| Bug | Fix | Commit |
|-----|-----|--------|
| `t` theme key did nothing | Added `cycle_theme` handler; saves to config | 181b287 |
| `u` edit: no cursor movement | Switched `input_without_shortcuts` → `input` for full key routing | 181b287 |
| `F` panel: slot 2 collapses to slot 1 on save | Replaced length-based padding with deterministic f1..fN numbering | 181b287 |
| `F` panel: Enter didn't apply selected row as filter | Enter now applies focused row's filter value | 181b287 |
| `F` panel: no cursor editing inside text fields | Same `input` switch as editor fix | 181b287 |
| Ctrl+F added as filter toggle (new feature) | Ctrl+F saves current filter, clears; second press restores | 181b287 |
| Group header harsh REVERSED style | Replaced with subtle `Color::Gray` label | 1a4e3a5 |
| Alpha sort: non-contiguous groups | Added stable-sort by group key before display-row build | 1a4e3a5 |
| Preset panel empty on first use | Added minimum 5 slots fallback | 1a4e3a5 |

## Key Files

- `crates/todotxt-tui/src/app.rs` — key handlers, toggle state, preset slot logic
- `crates/todotxt-tui/src/theme.rs` — `group_header` style field
- `.planning/phases/18-validation-ship-readiness/UAT.md` — corrected checklist

## Next Phase Readiness

UAT gate cleared. Plan 18-04 (milestone audit + close-out) may proceed.

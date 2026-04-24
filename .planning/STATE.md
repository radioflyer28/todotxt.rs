---
gsd_state_version: 1.0
milestone: v1.3
milestone_name: Feature/Hotkey Parity with todotxt.net
current_phase: 19
status: Defining requirements
last_updated: "2026-04-24T00:00:00.000Z"
last_activity: 2026-04-24
progress:
  total_phases: 0
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-04-24)

**Core value:** A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.
**Current focus:** Milestone v1.3 — TUI feature and hotkey parity with todotxt.net.

## Current Position

Phase: Not started (defining requirements)
Plan: -
Status: Defining requirements for v1.3 parity milestone
Last activity: 2026-04-24

## Next Step

Research todotxt.net interaction patterns and define v1.3 requirements.

## Pending Decisions

- Final v1.3 requirement set after parity research.

## Blockers

None.

## Accumulated Context

**Shipped milestones:**

- v1.0 (phases 01-08): core + CLI shipped and verified.
- v1.1 (phases 09-13): TUI shipped and verified.

**v1.2 kickoff scope:**

- todo.sh compatibility layer
- filter Esc cancel/restore behavior
- conditional theme label in status bar
- grouping/sorting parity alignment with todotxt.net behavior
- filter definition layout alignment + TOML persistence
- deferred-task parity investigation and implementation if confirmed

**v1.3 kickoff scope:**

- TUI multi-selection parity: shift-range selection and disjoint selection mode
- Bulk delete and append actions across selected tasks
- Token-aware normalization of appended/edited todo.txt metadata
- Hotkey/help parity audit against todotxt.net docs and screenshots

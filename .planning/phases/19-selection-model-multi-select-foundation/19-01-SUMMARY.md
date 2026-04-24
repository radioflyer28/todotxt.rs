---
phase: 19-selection-model-multi-select-foundation
plan: 01
subsystem: ui
tags: [ratatui, selection, hashset, tui, multi-select]

# Dependency graph
requires: []
provides:
  - "HashSet<usize> selection state on App struct (canonical file indices)"
  - "selection_anchor: Option<usize> for future shift-range operations"
  - "disjoint_select: bool flag for vi-visual-line mode"
  - "toggle_task_selection helper with GroupHeader no-op safety"
  - "v/Space/Esc disjoint mode key handlers"
  - "Selected-row rendering: BOLD + '>' prefix, REVERSED|BOLD for cursor+selected"
affects: [phase-20-bulk-actions, phase-22-keymap-config]

# Tech tracking
tech-stack:
  added: [std::collections::HashSet, tempfile (dev-dependency)]
  patterns:
    - "TDD RED/GREEN cycle per plan task"
    - "disjoint_select as bool flag on App (not new AppMode variant)"
    - "Canonical index HashSet for multi-selection — consistent with display_indices Vec<usize>"

key-files:
  created: []
  modified:
    - crates/todotxt-tui/src/app.rs
    - crates/todotxt-tui/Cargo.toml

key-decisions:
  - "D-01: HashSet<usize> of canonical file indices for selected_tasks"
  - "D-04: disjoint_select is bool flag, NOT new AppMode — normal navigation keys continue working"
  - "D-05: v toggles disjoint mode (vi visual-line mnemonic)"
  - "D-06: Space marks/unmarks cursor task only when disjoint_select=true"
  - "D-07: Esc clears entire selection AND resets disjoint_select=false"
  - "D-08: GroupHeader rows never enter selected_tasks — toggle_task_selection is a no-op on headers"
  - "D-13: Cursor row uses Modifier::REVERSED (unchanged)"
  - "D-14: Selected non-cursor: Modifier::BOLD + '> ' prefix"
  - "D-15: Cursor+selected: REVERSED | BOLD combined"
  - "D-17: GroupHeader styling unchanged regardless of selection state"

patterns-established:
  - "Toggle selection by resolving DisplayRow::Task(idx) — never mutate on GroupHeader"
  - "highlight_style on ratatui List is set dynamically based on cursor membership in selected_tasks"

requirements-completed: [SEL-02, SEL-04]

# Metrics
duration: 35min
completed: 2026-04-24
---

# Phase 19 Plan 01: Canonical Selection State + Disjoint Mode + Rendering Summary

**HashSet-based multi-select state established on App with disjoint mode toggle (v/Space/Esc) and per-row visual precedence (BOLD+prefix for selected, REVERSED|BOLD for cursor+selected).**

## Performance

- **Duration:** ~35 min
- **Started:** 2026-04-24T00:00:00Z
- **Completed:** 2026-04-24T00:00:00Z
- **Tasks:** 3 completed
- **Files modified:** 2

## Accomplishments

- Added `selected_tasks: HashSet<usize>`, `selection_anchor: Option<usize>`, `disjoint_select: bool` to App struct with proper initialization in `App::new`
- Implemented disjoint mode key flow: `v` toggles mode, `Space` marks/unmarks (with GroupHeader no-op safety), `Esc` clears selection and exits mode
- Updated `render_task_list` with explicit visual precedence: cursor=REVERSED, selected-non-cursor=BOLD+`>`, cursor+selected=REVERSED|BOLD, headers unchanged

## Task Commits

Each task was committed atomically (TDD RED/GREEN for tasks 1 and 2):

1. **Task 1 RED: failing tests for selection state** - `13eecab` (test)
2. **Task 1 GREEN: canonical selection state and helpers** - `930dd3d` (feat)
3. **Task 2 RED: failing tests for disjoint mode keys** - `be6ba8d` (test)
4. **Task 2 GREEN: disjoint selection mode keys** - `9eece36` (feat)
5. **Task 3: render selected rows with visual precedence** - `b78c650` (feat)

## Files Created/Modified

- `crates/todotxt-tui/src/app.rs` — Added selection state fields, helper methods, key handlers, and render updates
- `crates/todotxt-tui/Cargo.toml` — Added `tempfile` dev-dependency for test helpers

## Deviations from Plan

**1. [Rule 2 - Missing Critical Functionality] Added `#[allow(dead_code)]` to `clear_selection`**
- **Found during:** Task 1 GREEN
- **Issue:** `#![deny(warnings)]` in main.rs causes unused private method to fail compilation; `clear_selection` is the correct API but not yet called (Task 2 inlines the clear logic directly in the Esc handler)
- **Fix:** Added `#[allow(dead_code)]` attribute to keep the method available for future callers while Task 2 inlines the logic
- **Files modified:** `crates/todotxt-tui/src/app.rs`

## TDD Gate Compliance

- RED gate (test commit): ✅ `13eecab` (Task 1), `be6ba8d` (Task 2)
- GREEN gate (feat commit): ✅ `930dd3d` (Task 1), `9eece36` (Task 2)
- REFACTOR gate: N/A — no structural cleanup needed

## Known Stubs

None — all selection state is wired to keyboard input and rendered. No placeholder values flow to the UI.

## Threat Flags

None — selection state is in-memory only; no new network endpoints, file access patterns, or trust boundary crossings introduced.

## Self-Check: PASSED

- [x] `crates/todotxt-tui/src/app.rs` — exists and modified ✅
- [x] Commits `13eecab`, `930dd3d`, `be6ba8d`, `9eece36`, `b78c650` all exist in git log ✅
- [x] `cargo test -p todotxt-tui` → 12 passed; 0 failed ✅
- [x] `cargo check -p todotxt-tui` → Finished cleanly ✅

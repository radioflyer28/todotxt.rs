---
phase: 50-input-ergonomics
plan: 02
subsystem: tui-quick-setter-autoselect
tags: [rust, tui, autocomplete, quick-setter, testing]
requires: [50-01]
provides:
  - "Continuity-first quick context/project selection defaults"
  - "Current-token-aware selection preservation while narrowing quick-setter matches"
affects: [normal-mode-quick-setters]
tech-stack:
  added: []
  patterns: [continuity-first selection, shared token intersection, regression-first tests]
key-files:
  created: []
  modified:
    - crates/todotxt-tui/src/app.rs
key-decisions:
  - "Auto-select applies to the TUI match-driven quick setters for contexts and projects"
  - "If the current token is still a valid candidate, it stays selected even while the candidate list narrows"
  - "When no current token is available, selection falls back to the best-ranked match at index 0"
requirements-completed: [AUTO-SEL-01, AUTO-SEL-02]
duration: 15min
completed: 2026-05-19
---

# Phase 50 Plan 02: TUI Quick-Setter Auto-Select Summary

The TUI quick context/project setters no longer reset to an arbitrary first item whenever the
popup opens or narrows. They now prefer the task’s current token when one is present, and only
fall back to the ranked candidate list when there is no continuity to preserve.

## Performance

- **Duration:** 15 min
- **Completed:** 2026-05-19
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments

- Added current-token selection logic for quick context/project setters.
- Preserved that selection while typing as long as the current token remains a valid candidate.
- Kept the existing ranked-match behavior as the fallback path when continuity is unavailable.
- Added focused TUI regression tests for popup-open selection and narrowing-time selection continuity.

## Verification

Passed:

```powershell
cargo test -p todotxt-tui
```

## Deviations from Plan

- The original plan artifact referenced desktop client autocomplete code. Execution was
  corrected to the Rust TUI quick-setter popup after scope was clarified.

## Issues Encountered

None.

## User Setup Required

None.

## Next Phase Readiness

The quick-setter popup now has stable continuity behavior, which reduces navigation friction and
gives future input refinements a more predictable baseline.

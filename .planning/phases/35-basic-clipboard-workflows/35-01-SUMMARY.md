---
phase: 35-basic-clipboard-workflows
plan: 01
status: complete
completed: "2026-04-30"
---

# Plan 35-01 Summary: Clipboard Backend + `y` Copy Action

## Objective
Integrate arboard clipboard backend and implement the `y` copy action for Normal mode.

## Completed
- **arboard 3.6.1** added to `crates/todotxt-tui/Cargo.toml`
- **`App.clipboard: Option<Clipboard>`** field added — lazily initialized on first use (avoids headless startup errors, D-02)
- **`copy_selected_to_clipboard()`** method implemented:
  - Targets `selected_tasks` if non-empty, else active cursor task (D-03)
  - Skips header rows silently (D-10)
  - Collects `task.to_raw()` text; joins multi-task with `\n` in descending index order (D-08, D-17)
  - Lazy-initializes arboard; graceful no-op on init failure
  - Status feedback: "copied 1 task" / "copied N tasks" via `push_runtime_warning` (D-09)
- **`y` key binding** added to `handle_normal_key` (`KeyModifiers::NONE` guard)

## Files Modified
- `crates/todotxt-tui/Cargo.toml` — arboard dependency
- `crates/todotxt-tui/src/app.rs` — import, struct field, init, method, key binding

## Build
`cargo check -p todotxt-tui` — clean, no errors or warnings

## Requirements Covered
- CLIP-01 ✓

## Self-Check: PASSED

## Next
Plan 02 — paste operations (`p` in Normal mode, Ctrl+V in Adding mode)

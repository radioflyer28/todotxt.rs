---
phase: 11-edit-mode
plan: 01
subsystem: todotxt-tui
tags: [edit-mode, tui, tui-textarea, app-mode, input, delete-confirm]
dependency_graph:
  requires: [10-core-tui]
  provides: [AppMode enum, mode-dispatched input, add/edit/delete UI, reload guard]
  affects: [crates/todotxt-tui/src/app.rs]
tech_stack:
  added: [tui-textarea 0.7]
  patterns: [mode-dispatched event handling, footer-swap layout, single-line editor via tui-textarea]
key_files:
  created: []
  modified:
    - Cargo.toml
    - crates/todotxt-tui/Cargo.toml
    - crates/todotxt-tui/src/app.rs
decisions:
  - "tui-textarea 0.7 (not 0.8 — doesn't exist) + downgrade ratatui to 0.29 + crossterm to 0.28 to match tui-textarea's dependency range"
  - "draw() changed to &mut self for future tui-textarea rendering flexibility"
  - "render_widget(&self.editor, area) works because versions now align on ratatui 0.29"
metrics:
  duration: ~30min
  completed: 2026-04-20
  tasks_completed: 3
  files_modified: 3
---

# Phase 11 Plan 01: Edit Mode Foundation — Add/Edit/Delete/Reload Guard Summary

**One-liner:** Mode-dispatched TUI with `AppMode` enum, inline add/edit via tui-textarea single-line editor, delete confirmation panel, and silent reload guard.

## What Was Built

- **AppMode enum** — `Normal`, `Adding`, `Editing { original_idx }`, `DeleteConfirm` (Copy + Eq for direct matching)
- **App struct** — extended with `mode: AppMode`, `editor: TextArea<'static>`, `pending_reload: bool`
- **Mode-dispatched `handle_event`** — delegates to `handle_normal_key`, `handle_editor_key`, `handle_delete_confirm_key`
- **Normal mode keys** — `n` add, `u`/`e` edit, `d` delete-confirm, `x` done, `j`/`k`/`g`/`G`/Ctrl+d/Ctrl+u nav, `q`/Ctrl+c quit
- **Editor mode** — Esc cancels, Enter saves, all other keys → `input_without_shortcuts()`
- **Delete confirm** — `y` deletes + return to Normal, any other key cancels
- **Reload guard** — `FileChanged` in non-Normal mode sets `pending_reload=true`; applied in `exit_edit_mode()` / `save_and_exit()` / `handle_delete_confirm_key()`
- **Mode-aware `draw(&mut self)`** — Normal/Adding/Editing: `[Min(0), Length(1)]`; DeleteConfirm: `[Min(0), Length(1), Length(1)]`
- **Helper renderers** — `render_task_list()`, `render_status_bar()`, `render_delete_confirm()`
- **Status bar hints updated** — `"q quit | n add | u edit | d del | x done | j/k nav"`

## Files Modified

| File | Change |
|------|--------|
| `Cargo.toml` | Added `tui-textarea = "0.7"` workspace dep; downgraded `ratatui` → `0.29`, `crossterm` → `0.28` |
| `crates/todotxt-tui/Cargo.toml` | Added `tui-textarea = { workspace = true }` |
| `crates/todotxt-tui/src/app.rs` | Full rewrite: AppMode, extended App, mode dispatch, draw() overhaul |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] tui-textarea version "0.8" does not exist**
- **Found during:** Task 1
- **Issue:** Plan specified `tui-textarea = "0.8"` but the latest version is 0.7.0
- **Fix:** Used `tui-textarea = "0.7"` instead
- **Files modified:** `Cargo.toml`, `crates/todotxt-tui/Cargo.toml`

**2. [Rule 1 - Bug] crossterm/ratatui version mismatch with tui-textarea 0.7**
- **Found during:** Task 2
- **Issue:** tui-textarea 0.7 requires `ratatui ^0.29` and `crossterm ^0.28`, but workspace had `ratatui 0.30` and `crossterm 0.29`. This caused a `From<Event>` trait bound failure in `input_without_shortcuts`.
- **Fix:** Downgraded workspace to `ratatui = "0.29"` and `crossterm = "0.28"` to align all versions. Full workspace builds cleanly.
- **Files modified:** `Cargo.toml`
- **Impact:** `frame.area()` confirmed to exist in ratatui 0.29 (added in 0.26). All Phase 10 code unaffected.

**3. [Rule 2 - Compatibility] draw() changed to &mut self**
- **Found during:** Task 3
- **Issue:** Plan specified `draw()` must be `&mut self` for TextArea rendering
- **Fix:** Changed signature and updated closure calls in `run()` and `handle_event()`
- **Outcome:** `frame.render_widget(&self.editor, area)` works correctly since ratatui and tui-textarea versions now match

**4. [Rule 1 - Bug] Temporary #[allow(dead_code)] for Task 1 intermediate commit**
- **Found during:** Task 1
- **Issue:** `#![deny(warnings)]` in main.rs causes AppMode variants + new App fields to error as dead code before Task 2 wires them
- **Fix:** Added `#[allow(dead_code)]` for Task 1 commit only; removed in Task 2 when all items were used

## Commits

| Hash | Message |
|------|---------|
| `aa59f04` | feat(11-01): add tui-textarea dep and AppMode enum |
| `edbf324` | feat(11-01): mode-dispatched input — add, edit, delete, reload guard |
| `cd501ac` | feat(11-01): mode-aware draw with footer-swap and delete confirm panel |

## Final Build Status

```
cargo build -p todotxt-tui
   Compiling todotxt-tui v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.26s
```

**Zero errors. Zero warnings.**

## Self-Check: PASSED

- [x] `crates/todotxt-tui/src/app.rs` modified — verified
- [x] `Cargo.toml` modified — verified  
- [x] `crates/todotxt-tui/Cargo.toml` modified — verified
- [x] All 3 commits exist in git log — aa59f04, edbf324, cd501ac
- [x] `cargo build -p todotxt-tui` — zero warnings, zero errors

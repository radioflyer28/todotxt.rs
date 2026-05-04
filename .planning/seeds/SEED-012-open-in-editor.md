---
id: SEED-012
status: dormant
planted: 2026-05-04
planted_during: v1.5 complete / preparing v1.6
trigger_when: next milestone (v1.6)
scope: Small
---

# SEED-012: Open task in $EDITOR from the TUI

## Why This Matters

The TUI inline editor works well for simple edits, but complex rewrites — reordering tokens, fixing multi-word project tags, adding multiple metadata fields — are cumbersome in a single-line text area without cursor movement shortcuts. Terminal users expect `$EDITOR` escape as a universal escape hatch (same pattern as `git commit`, `crontab -e`, etc.).

## When to Surface

**Trigger:** Next milestone (v1.6).

Matches when:
- TUI editing UX improvements are in scope
- Power-user / advanced editing workflows are being addressed

## Scope Estimate

**Small** — The pattern is well-established in terminal apps:
1. Write current task text to a temp file
2. `std::process::Command::new(&editor).arg(&tempfile).status()` (suspend TUI, exec editor, resume)
3. Read back the temp file on editor exit
4. Validate and apply the edited text as a task update
5. Resume TUI rendering

The tricky part is suspending/resuming the ratatui terminal correctly (disable raw mode, restore on return). This is a known pattern with ratatui and has prior art.

## Breadcrumbs

| File | Notes |
|------|-------|
| `crates/todotxt-tui/src/app.rs` line 851 | `AppMode::Editing` entry point (`u` key) — add `Ctrl+E` or `E` as alternate path |
| `crates/todotxt-tui/src/app.rs` line 2400 | `AppMode::Editing` save handler — same post-save path for external edit result |
| `crates/todotxt-tui/src/config.rs` | Would read `$VISUAL` → `$EDITOR` → fallback (e.g., `notepad` on Windows) |
| `crates/todotxt-tui/src/main.rs` | Terminal init — `crossterm::terminal::disable_raw_mode()` needed before exec |

## Notes

Key sequence proposal: `Ctrl+E` in Normal mode opens cursor task in `$EDITOR`. Could also work as a fallback from within Editing mode.

Editor resolution order: `$VISUAL` → `$EDITOR` → platform fallback (`notepad.exe` on Windows, `vi` on Unix). If no editor found, show an error in the status bar.

The temp file should be cleaned up after the editor returns, even on error. Use `std::env::temp_dir()` for cross-platform temp file placement.

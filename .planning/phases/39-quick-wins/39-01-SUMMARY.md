# Phase 39-01 Summary: Archive Workflow

## Status: COMPLETE ✅

## What Was Built
- `AppMode::ArchiveConfirm` variant added to the mode enum
- `A` key (configurable via `archive` action in keymap) triggers archive confirm overlay when completed tasks exist
- `archive_path()` helper resolves `done.txt` path from config or sibling default
- `archive_tasks()` writes completed tasks to `done.txt` (append or create), removes them from the active list, pushes a single undo entry. Uses write-first atomic pattern via `tempfile::NamedTempFile::new_in()` + `persist()`.
- `handle_archive_confirm_key()` dispatches `y`/`Enter` → archive, `n`/`Esc` → cancel
- `render_archive_confirm()` renders a one-line confirmation status bar overlay
- `draw()` dispatches `ArchiveConfirm` mode to the archive confirm layout
- `tempfile` added to `[dependencies]` in `Cargo.toml` (was dev-only)

## Files Modified
- `crates/todotxt-tui/src/app.rs` — AppMode variant, key handler, archive methods, render
- `crates/todotxt-tui/src/config.rs` — `archive` keymap entry (A key)
- `crates/todotxt-tui/Cargo.toml` — tempfile in dependencies

## Tests Added (4)
- `archive_tasks_moves_completed_to_done_txt`
- `archive_tasks_pushes_undo_entry`
- `archive_tasks_appends_to_existing_done_txt`
- `archive_confirm_cancel_leaves_tasks_unchanged`

All tests use `tempfile::tempdir()` for done.txt path (Windows open-handle safe).

## AC-01 Decision (D-ARCH)
Write-first atomic via NamedTempFile in same directory as done.txt, then persist().

## Commit
`feat(39-01): add archive workflow — AppMode::ArchiveConfirm, A key, archive_tasks() write-first atomic, undo support`

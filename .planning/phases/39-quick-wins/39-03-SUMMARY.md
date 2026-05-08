# Phase 39-03 Summary: Ctrl+E External Editor

## Status: COMPLETE ✅

## What Was Built
- `struct RawModeGuard` — RAII guard that calls `disable_raw_mode()` + `LeaveAlternateScreen` on construction, `EnterAlternateScreen` + `enable_raw_mode()` on Drop. All terminal calls use `let _ = ...` (never panics in Drop).
- `fn resolve_editor() -> Option<String>` — checks `$VISUAL` → `$EDITOR` → platform fallback (`notepad.exe` on Windows, `vi` on non-Windows). Always returns `Some` (platform fallback always succeeds).
- `App::launch_external_editor()` — pushes undo entry, creates `RawModeGuard`, spawns editor with `Command::new(editor).arg(&self.todo_path).status()`, drops guard (restores TUI), reloads `TaskList::load()`, rebuilds panes, posts status message. On missing editor or spawn error: status bar message, no crash.
- `Ctrl+E` arm in `handle_normal_key` — `KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL)` dispatches to `launch_external_editor()`.

## Files Modified
- `crates/todotxt-tui/src/app.rs` — `RawModeGuard`, `resolve_editor`, `launch_external_editor`, Ctrl+E key arm

## Tests Added (3)
All 3 use a `static ENV_LOCK: Mutex<()>` to serialize env-var access (prevent parallel test pollution):
- `resolve_editor_prefers_visual_over_editor` — VISUAL takes precedence over EDITOR
- `resolve_editor_falls_back_to_editor_when_visual_unset` — EDITOR used when VISUAL unset
- `resolve_editor_falls_back_to_platform_default` — platform fallback when neither set

## Commit
`feat(39-03): add Ctrl+E external editor — RawModeGuard RAII, resolve_editor() VISUAL/EDITOR/fallback, launch_external_editor()`

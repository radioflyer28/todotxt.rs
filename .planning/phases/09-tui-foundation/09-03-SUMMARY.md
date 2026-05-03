---
phase: 09-tui-foundation
plan: "03"
status: complete
commit: 045406f
verified: true
---

# Plan 09-03 Summary: Event Loop + App + Full Main Wiring

## What Was Built

- `crates/todotxt-tui/src/event.rs` — `AppEvent` enum: `Key(KeyEvent)`, `Resize(u16, u16)`, `FileChanged`, `Error(String)`
- `crates/todotxt-tui/src/app.rs` — `App` struct (`should_quit`, `task_list`, `todo_path`); `run()` blocks on `mpsc::recv()`; `handle_event()` handles all variants; `draw()` renders plain-text numbered task list using `frame.area()` (ratatui 0.30)
- `crates/todotxt-tui/src/main.rs` — final wiring: two `thread::spawn` senders (crossterm `read()` loop + FileWatcher `Arc<dyn Fn()>` callback), single `mpsc::channel`, `TerminalGuard`, `App::run()`

## Acceptance Results (Human Verified)

- `cargo build -p todotxt-tui` → exit 0, zero warnings
- `cargo build --workspace` → exit 0, no regressions
- Binary enters full-screen, displays numbered task list ✓
- `q` exits cleanly, terminal restored ✓
- External edit to `todo.txt` causes auto-refresh within ~1s ✓

## Decisions Applied

- D-01: No tokio — sync threads + blocking `crossterm::event::read()`
- D-02: Single `mpsc::channel`, two `tx.clone()` senders
- D-03: No state mutation in sender threads (callbacks only send `AppEvent`)
- D-10: Plain-text `Paragraph` render, one line per task
- D-11: `TaskList::reload()` on `AppEvent::FileChanged`
- PITFALL avoided: `frame.area()` not `frame.size()` (ratatui 0.30)
- PITFALL avoided: `KeyEventKind::Press` filter (no key-release duplication)

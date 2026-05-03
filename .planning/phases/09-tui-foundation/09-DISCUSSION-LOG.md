# Phase 9: TUI Foundation — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `09-CONTEXT.md` — this log preserves the alternatives considered.

**Date:** 2026-04-19
**Phase:** 09-tui-foundation
**Areas discussed:** Event loop design, Config sharing, Panic safety, Phase 9 scope

---

## Event loop design — tokio async vs std threads + channels?

| Option | Description | Selected |
|--------|-------------|----------|
| Sync: std threads + mpsc channel (no tokio) | crossterm::event::poll() + std::sync::mpsc. FileWatcher callback sends to a channel; main loop polls both. No tokio needed. Simpler, fewer dependencies. | ✓ |
| Async: tokio runtime + crossterm::EventStream | tokio runtime + crossterm::EventStream (async) + tokio::sync::mpsc. Requires adding tokio. | |
| Start sync, refactor later | Pragmatic but risks mid-milestone refactor. | |

**Follow-up: Main loop mechanism**

User expressed concern about poll(timeout) adding UI latency. After explanation that keypress latency is zero (poll returns immediately on key) and file-change lag is max ~50ms with poll, user opted for the two-sender-threads approach:

| Option | Description | Selected |
|--------|-------------|----------|
| poll(50ms) + drain mpsc | Simple, ~50ms file-change lag | |
| Two-sender threads + recv() | Crossterm events thread calls blocking read(); watcher callback sends to same mpsc. Main loop calls recv(). Zero latency. | ✓ |

**User's choice:** Two-sender threads + `mpsc::recv()` — zero latency, event-driven. User asked about race conditions; confirmed that `mpsc` is inherently thread-safe and all state mutation is on the main thread.

---

## Config sharing — where does Config live?

| Option | Description | Selected |
|--------|-------------|----------|
| Move Config to todotxt-core | Clean, no duplication, requires refactor. | |
| TUI defines its own TuiConfig struct | Independent struct, reads same TOML, no crate changes needed. | → discussed further |
| TUI depends on todotxt-cli for Config | Circular coupling risk. | |

**Follow-up: How to share overlapping fields**

| Option | Description | Selected |
|--------|-------------|----------|
| Duplicate shared fields (todo_file, done_file) | Simple, independent structs. | |
| CommonConfig struct with serde flatten | Shared fields defined once, flattened into both. | → discussed further |

**Follow-up: Where does CommonConfig live?**

User intervened with free text: "we shouldn't have to modify core to support base CLI/TUI features. The same TOML file should be usable by CLI and TUI with their unique params while sharing overlapping params."

**User's choice:** No CommonConfig. Both `Config` (CLI) and `TuiConfig` (TUI) define their own fields as independent structs. Both parse the same TOML file with serde — each reads what it knows, silently ignores the rest. No changes to `todotxt-core`.

---

## Panic safety approach — color-eyre vs raw panic::set_hook

| Option | Description | Selected |
|--------|-------------|----------|
| color-eyre | Sets panic hook automatically, pretty error formatting, one dependency. Ratatui template standard. | ✓ |
| Manual std::panic::set_hook | Zero new dependencies, ~10 lines boilerplate. | |
| Drop-only (no panic hook) | Doesn't catch panics, only normal exits. | |

**User's choice:** color-eyre

---

## Phase 9 scope boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Pipeline proof — log receipt, no display | FileWatcher event received, logged to stderr. | |
| Minimal display — show raw tasks, prove watcher updates view | Task list visible, watcher auto-refreshes it. | ✓ |

**Follow-up: What's left for Phase 10?**

| Option | Description | Selected |
|--------|-------------|----------|
| Phase 9 includes minimal display; Phase 10 focuses on interaction | ✓ | ✓ |
| Phase 9 includes display + basic navigation; Phase 10 adds done/undo | | |

**Follow-up: How minimal is the display?**

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: plain text lines, no colors or selection | ✓ | ✓ |
| Slightly more: List widget with cursor highlight (no colors/theme) | | |

**User's choice:** Plain text lines only. Phase 9 proves the pipeline. Phase 10 builds the interactive widget.

---

## the agent's Discretion

- Module layout within `crates/todotxt-tui/src/`
- Which ratatui widget for the plain-text list (Phase 10 replaces it)
- `std::sync::mpsc` vs `crossbeam-channel` (prefer std)

---

## Deferred Ideas

- `[tui]` config subsection for theme — Phase 13
- Cursor highlight / selection — Phase 10
- Colors and theming — Phase 13
- Navigation keybinds — Phase 10
- Status bar — Phase 10

# Phase 9: TUI Foundation — Context

**Gathered:** 2026-04-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 9 delivers a compilable `todotxt-tui` crate in the Cargo workspace with:
- A safe terminal lifecycle (enter/exit full-screen, panic recovery)
- A running event loop connected to both keyboard input and the file watcher
- Config reading via a standalone `TuiConfig` struct that parses the same TOML file as the CLI
- A minimal plain-text task list display (no colors, no selection highlight) that proves the pipeline end-to-end

Phase 10 owns all interaction (navigation, done/undo, status bar, quit keybinds).
Phase 13 owns theming and colors.

</domain>

<decisions>
## Implementation Decisions

### Event Loop Architecture

- **D-01: Sync over async** — No tokio. The existing `FileWatcher` is callback-based (non-tokio); adding an async runtime would be unnecessary complexity for this phase.
- **D-02: Two-sender threads + `mpsc::recv()`** — Two threads both hold an `mpsc::Sender<AppEvent>`:
  1. A crossterm-events thread that calls the blocking `crossterm::event::read()` and sends key/resize events.
  2. The `FileWatcher` callback closure sends a `AppEvent::FileChanged` via the same sender.
  The main loop calls `receiver.recv()` — no polling, no timeout, fully event-driven.
- **D-03: No race conditions** — All app state mutation happens exclusively on the main thread. The two sender threads only send messages; they never touch app state.

### Config

- **D-04: TuiConfig in `todotxt-tui`** — The TUI crate defines its own `TuiConfig` struct (independent from CLI's `Config`). Both parse the same TOML file with serde; each reads what it knows and silently ignores sections it doesn't.
- **D-05: No changes to `todotxt-core` for config** — `resolve_config_path` from core is still used for portable mode, but the Config struct itself stays in the CLI crate untouched.
- **D-06: TuiConfig fields** — At Phase 9 scope: `todo_file: Option<PathBuf>`, `done_file: Option<PathBuf>`, `auto_creation_date: bool` (mirrors CLI). A `[tui]` subsection (`TuiSection`) will be added in Phase 13 for theme selection.
- **D-07: Config path resolution** — Reuse `todotxt_core::resolve_config_path` for portable mode. For the platform path, follow the same logic as CLI (`ProjectDirs::from("", "", "todotxt")`).

### Panic Safety

- **D-08: `color-eyre`** — Install `color-eyre` and call `color_eyre::install()` at startup. It sets a panic hook that calls the terminal cleanup closure before printing the panic message. Also handles `?`-propagated errors with pretty output.
- **D-09: RAII terminal guard** — Wrap terminal setup/teardown in an RAII struct so normal exits (via `?` unwinding) also restore terminal state, not just panics.

### Phase 9 Scope

- **D-10: Minimal display — plain text lines, no colors, no selection** — Phase 9 renders the task list as raw text strings (one per line), no ratatui styling, no cursor highlight. The goal is to prove the full pipeline works: config → load tasks → display → watcher fires → display updates. Phase 10 owns the interactive ratatui widget with navigation.
- **D-11: File-watcher proof** — When the todo.txt file changes externally, the display must refresh automatically. A visible update (re-render of the task list) is the acceptance criterion — no need for a status indicator at this stage.

### the agent's Discretion

- Exact module layout within `crates/todotxt-tui/src/` (main.rs, app.rs, terminal.rs, etc.) — agent decides
- Which ratatui widget to use for the plain-text list (Paragraph vs List) — agent decides, Phase 10 will replace it
- Whether to use `std::sync::mpsc` or `crossbeam-channel` for the event channel — agent decides (prefer std)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core Library APIs

- `crates/todotxt-core/src/lib.rs` — Public API surface (what TUI can import from core)
- `crates/todotxt-core/src/watcher.rs` — FileWatcher: callback signature, how to construct it, threading model
- `crates/todotxt-core/src/task_list.rs` — TaskList: how to load the file, what the API returns
- `crates/todotxt-core/src/portable.rs` — `resolve_config_path`: portable mode config path resolution

### CLI Config Reference

- `crates/todotxt-cli/src/config.rs` — Reference for Config struct shape and TOML field names to match in TuiConfig

### Requirements

- `.planning/REQUIREMENTS.md` — TUI-INFRA-01, TUI-INFRA-02 are this phase's requirements

### Research

- `.planning/research/STACK.md` — Crate versions: ratatui 0.30.0, crossterm 0.29.0, color-eyre 0.6.5
- `.planning/research/ARCHITECTURE.md` — Module structure, event loop patterns, terminal lifecycle patterns
- `.planning/research/PITFALLS.md` — Critical: terminal restore on panic, `color-eyre` hook, dependency version alignment

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `todotxt_core::FileWatcher` — Callback-based, takes `Arc<dyn Fn() + Send + Sync>`. The TUI sends to a channel inside the closure.
- `todotxt_core::TaskList` — Main API for loading and reading tasks. Call `TaskList::load(&path)` or equivalent.
- `todotxt_core::resolve_config_path` — Reuse for portable mode config resolution (same as CLI).
- `crates/todotxt-cli/src/config.rs::Config::default_path()` — Reference implementation for platform config path via `ProjectDirs`.

### Established Patterns

- `#![deny(warnings)]` — Both existing crates enforce this. The TUI crate must do the same.
- Workspace dependencies — New crates in `crates/` inherit from `[workspace.dependencies]`. Add ratatui, crossterm, color-eyre to the workspace Cargo.toml; then reference them in `todotxt-tui/Cargo.toml` with `{ workspace = true }`.
- `thiserror` for error types (core pattern) — Consider using it in TUI crate for a `TuiError` type.

### Integration Points

- `Cargo.toml` (workspace root) — Add `crates/todotxt-tui` to `members`. Add ratatui 0.30, crossterm 0.29 (with `event-stream` feature if ever needed, skip for now), color-eyre 0.6 to `[workspace.dependencies]`.
- `todotxt-core` feature flag — The TUI must enable the `watching` feature: `todotxt-core = { workspace = true, features = ["watching"] }`.

</code_context>

<specifics>
## Specific Ideas

- User explicitly chose **no tokio** — do not add tokio as a dependency.
- User wants the TOML config file to be shared naturally by both CLI and TUI, each reading its own fields — not a shared struct, just serde parsing the same file independently.
- Phase 9 ends with a plain-text list visible on screen that auto-refreshes when the file changes. That's the success bar — not a polished UI.

</specifics>

<deferred>
## Deferred Ideas

- `[tui]` config subsection (theme selection) — Phase 13
- Cursor highlight / selection — Phase 10
- Colors and theming — Phase 13
- Navigation keybinds (j/k, g/G, Ctrl+d/u) — Phase 10
- Status bar — Phase 10

</deferred>

---

*Phase: 09-tui-foundation*
*Context gathered: 2026-04-19*

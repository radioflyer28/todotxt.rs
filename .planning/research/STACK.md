# Stack Research — v1.1 TUI Interface

**Researched:** 2026-04-18  
**Confidence:** HIGH (all versions verified via crates.io)

## Core TUI Stack

| Crate | Version | Role | Rationale |
|-------|---------|------|-----------|
| `ratatui` | 0.30.0 | TUI framework — layout, widgets, rendering | Current community-maintained fork of tui-rs; active, well-documented, crossterm is its default backend. Provides `List`, `Paragraph`, `Block`, `Table`, `Popup`, `Scrollbar` widgets out of the box. |
| `crossterm` | 0.29.0 | Terminal backend for ratatui | Cross-platform (Windows + Unix), the default backend ratatui ships with. Handles raw mode, alternate screen, cursor, color. Enable the `event-stream` feature for tokio integration. |
| `tui-textarea` | 0.7.0 | Inline text editing widget | Drop-in ratatui widget for task input/editing. Supports undo/redo, Emacs shortcuts, single-line mode, validation hooks, and cursor-line highlighting. No need to hand-roll a text input field. |
| `color-eyre` | 0.6.5 | Error handling and panic hooks | Required to properly restore the terminal on panic. Call `color_eyre::install()` before `ratatui::init()` — prevents garbled terminal state on crash. Ratatui's own quickstart templates use it. |

## Optional / Situational

| Crate | Version | Role | When to Add |
|-------|---------|------|-------------|
| `tokio` | existing (re-use) | Async runtime | Already in `todotxt-core`. **Do not add a second runtime.** |
| `futures` | ~0.3 | `StreamExt` trait for `EventStream` | Only needed if using `crossterm::event::EventStream` (the tokio-compatible event stream). May already be transitively present. |

## Integration Notes

### Tokio + crossterm event loop

`crossterm` ships an `event-stream` feature that exposes `EventStream` — an async `futures::Stream<Item = Result<Event>>`. This integrates directly with `tokio::select!` so a single async task can multiplex:

- Terminal key/mouse events (from `EventStream`)  
- File-change notifications (from `todotxt-core`'s `notify`/tokio watcher channel)

```toml
# todotxt-tui/Cargo.toml
[dependencies]
ratatui        = "0.30"
crossterm      = { version = "0.29", features = ["event-stream"] }
tui-textarea   = "0.7"
color-eyre     = "0.6"
tokio          = { workspace = true }           # re-use workspace dep
todotxt-core   = { path = "../todotxt-core" }

futures        = "0.3"                          # for StreamExt on EventStream
```

```rust
// Skeleton of the TUI event loop
use crossterm::event::{EventStream, Event, KeyCode};
use futures::StreamExt;
use tokio::sync::mpsc::Receiver;

async fn run(mut file_rx: Receiver<todotxt_core::WatchEvent>) -> color_eyre::Result<()> {
    let mut terminal = ratatui::init();
    let mut events = EventStream::new();

    loop {
        terminal.draw(|f| ui(f, &state))?;

        tokio::select! {
            Some(Ok(event)) = events.next() => {
                handle_key(event, &mut state);
            }
            Some(change) = file_rx.recv() => {
                state.reload_from_disk(change);
            }
        }

        if state.should_quit { break; }
    }

    ratatui::restore();
    Ok(())
}
```

**Key constraint:** `ratatui::init()` / `ratatui::restore()` must bracket the TUI session. `color-eyre`'s panic hook ensures `restore()` is called even on crash, preventing a broken terminal.

### Theming / color

Ratatui has built-in `Style`, `Color` (16-color, 256-color ANSI, and RGB), and `Modifier` (bold, italic, dim, etc.) — no separate theming crate is needed. Define a `Theme` struct in-crate holding named `Style` constants.

### Autocomplete

No mature standalone autocomplete widget crate exists for ratatui that is worth adding. For todo.txt context (project `+tag`, context `@tag` completion), implement a simple popup `List` widget driven by a filtered `Vec<&str>` from `todotxt-core`'s tag index. This is ~50 lines and avoids a dependency for a single feature.

## Crates to Skip

| Crate | Why Skip |
|-------|---------|
| `termion` | Unix-only; crossterm already handles cross-platform. Adding both wastes compile time and creates backend conflicts. |
| `tui-rs` | Deprecated predecessor of ratatui. Do not mix. |
| `cursive` | Separate framework with its own event loop; incompatible with the existing tokio runtime approach. |
| `iocraft` | Declarative/React-style TUI, different paradigm, immature compared to ratatui for this use case. |
| `indicatif` | Progress bars for CLI output. Not a TUI widget; redundant inside ratatui's render loop. |
| `dialoguer` | CLI prompt library (stdin-based). Incompatible with full-screen TUI; use tui-textarea for input instead. |
| `console` / `ansi_term` | Low-level ANSI styling. Ratatui's `Style` API already abstracts this. |
| `crossbeam-channel` | Not needed — tokio's `mpsc` and `select!` cover all inter-task communication between the file watcher and the TUI loop. |
| `async-std` | Second async runtime. Conflicts with tokio. Never mix runtimes. |
| `egui` / `iced` | GUI frameworks, not terminal. |

## Cargo.toml Snippet (todotxt-tui)

```toml
[package]
name    = "todotxt-tui"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui      = "0.30"
crossterm    = { version = "0.29", features = ["event-stream"] }
tui-textarea = "0.7"
color-eyre   = "0.6"
futures      = "0.3"
tokio        = { workspace = true }
todotxt-core = { path = "../todotxt-core" }
```

## Sources

- crates.io/crates/ratatui — v0.30.0 (4 months ago, MSRV 1.86.0)
- crates.io/crates/crossterm — v0.29.0 (event-stream feature verified)
- crates.io/crates/tui-textarea — v0.7.0 (ratatui + crossterm support verified)
- crates.io/crates/color-eyre — v0.6.5
- ratatui.rs/concepts/event-handling — EventStream + tokio::select! pattern

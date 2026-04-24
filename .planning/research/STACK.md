# Stack Research — v1.3 Parity Work

**Researched:** 2026-04-24  
**Confidence:** HIGH (grounded in the current workspace plus the todo.txt spec)

## Reuse Existing Stack

No framework change is needed for v1.3. The parity work fits the current Rust TUI stack:

| Component | Status | Use in v1.3 |
| --------- | ------ | ----------- |
| `ratatui` | already shipped | Selection rendering, status hints, bulk-action overlays |
| `crossterm` | already shipped | Shift/Ctrl key combinations and selection-mode key handling |
| `tui-textarea` | already shipped | Token-aware normalization in append/edit flows |
| `todotxt-core::Task` | already shipped | Canonical parse/rebuild path for priority, due, threshold, projects, contexts |

## Likely Code Additions

| Area | Expected change | Why |
| ---- | --------------- | --- |
| `crates/todotxt-tui/src/app.rs` | Add multi-selection state, anchor tracking, bulk mutation handlers | Current app is single-row oriented |
| `crates/todotxt-tui/src/theme.rs` | Add styles for selected ranges / secondary selections | Need visible parity cues |
| `crates/todotxt-core/src/task.rs` | Add safe mutation helpers for recognized todo.txt metadata if existing builders are insufficient | Smart normalization should not hand-roll raw-string surgery in the TUI |
| `crates/todotxt-tui/src/help` or status text | Update user-facing key hints / help overlay text | Parity work changes discoverability contract |

## Do Not Add By Default

- No new parsing library — `todotxt-core::Task::parse()` already understands priority, dates, `@context`, `+project`, `due:`, and `t:`.
- No external selection-state crate unless complexity proves it necessary. A `BTreeSet<usize>` or canonical-task-id set is enough initially.
- No GUI-specific parity work in this milestone.

## Integration Guidance

- Keep selection state keyed to canonical task identity, not transient display rows. Group headers and filtering already decouple display rows from task rows.
- Route smart normalization through `todotxt-core` builders or new helpers so append/edit paths share one serialization policy.
- Treat todo.txt spec as authority for token placement and `todotxt.net` as authority for interaction model and hotkeys.

## Watch For

- Bulk mutations must survive sort/filter reloads without losing selection unexpectedly.
- Range selection must coexist with grouped rows and non-selectable headers.
- Smart append/edit should be optically simple but semantically conservative: normalize recognized tokens, preserve unknown text.

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

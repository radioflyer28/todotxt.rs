# todotxt.net — Rust Port

## Current State

v1.2 Compatibility + UX Alignment is active (started 2026-04-23).

The project now includes:
- todotxt-core (library)
- todotxt-cli (command-line interface)
- todotxt-tui (interactive terminal UI)

Milestone archives:
- .planning/milestones/v1.0-ROADMAP.md
- .planning/milestones/v1.1-ROADMAP.md

---

## What This Is

A Rust port of todotxt.net — a todo.txt manager that runs cross-platform (Windows, Linux, macOS). The project provides a strict todo.txt-compatible core library (`todotxt-core`) plus a full-featured CLI (`todotxt-cli`) for human and AI agent use. A TUI (interactive terminal UI) and native GUI (desktop app) are seeded for future milestones.

## Core Value

A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## Current State

**v1.0 and v1.1 shipped.** Core, CLI, and TUI are complete and tested.
**v1.2 is now in planning/execution.** Focus is todo.sh compatibility and UX alignment with todotxt.net behavior.

- `todotxt-core`: parser, Task model, TaskList CRUD, filter engine, sort engine, file watching, portable mode
- `todotxt-cli`: 25+ commands — read, write, enrichment, bulk operations, structured JSON output, TOML config, named presets, shell completions
- `todotxt-tui`: keyboard-driven terminal UI with add/edit/delete, filtering/sorting, presets, themes, and auto-reload
- 207 tests passing, 0 clippy warnings, `#![deny(warnings)]` enforced in both crates
- Cross-platform: Windows, Linux, macOS

## Requirements

### Validated (v1.0)

- ✓ todo.txt parser: all fields (priority, projects, contexts, due/threshold/creation/completion dates, body) — v1.0
- ✓ Task serializer: strict round-trip (no mutating user-authored text) — v1.0
- ✓ TaskList CRUD: atomic file writes, add/update/delete — v1.0
- ✓ File watching with 1-second debounce — v1.0
- ✓ Filter engine: substring, negation, DONE/-DONE, due:today/past/future/active — v1.0
- ✓ Sort engine: priority, due date, alphabetical, project, context — v1.0
- ✓ BOM/CRLF normalization on load; round-trip preservation on save — v1.0
- ✓ Portable mode: config beside binary takes precedence — v1.0
- ✓ CLI read commands: list, show, stats, projects, contexts — v1.0
- ✓ CLI write commands: add, do, undo, del, edit, append, prepend — v1.0
- ✓ CLI enrichment: pri, depri, due, postpone — v1.0
- ✓ CLI bulk: archive, del-done — v1.0
- ✓ Structured JSON output (--json), exit codes (0/1/2), --no-color/--quiet — v1.0
- ✓ TOML config (platform paths) + named filter presets — v1.0
- ✓ Shell completions (bash, zsh, fish, PowerShell) — v1.0
- ✓ Cross-platform builds and CI (Windows, Linux, macOS) — v1.0 (CI workflow present)

### Validated (v1.1)

- ✓ TUI interface (ratatui) — `todotxt-tui` crate and binary shipped in workspace
- ✓ Navigation and actions — list navigation, done toggle, add/edit/delete flows, status bar
- ✓ Filter/sort and presets — query filter panel, sort cycle, active context in status bar
- ✓ Autocomplete and UX guards — @/+ completion and deferred reload while editing
- ✓ Theme and polish — default/light theme, config selection, NO_COLOR behavior, terminal restoration

### Active (v1.2)

- [ ] todo.sh compatibility layer
- [ ] TUI filter Esc cancel/restore behavior
- [ ] TUI status bar theme label shown conditionally
- [ ] TUI sort/group alignment with todotxt.net grouping semantics
- [ ] TUI filter definition layout alignment and TOML persistence
- [ ] Deferred-task parity decision and support (`t:`) when confirmed

### Planned (future milestone)

- [ ] GUI interface (native desktop)
- [ ] CI/CD release pipeline and package distribution

### Out of Scope

- GUI interface — deferred to v1.2 (seed planted)
- todo.sh compatibility layer — deferred (seed planted)
- Windows-specific features (system tray) in CLI — GUI milestone handles platform-specific features
- Interactive prompts / REPL — anti-feature for agent use; CLI must be scriptable

## Context

**Shipped:** v1.0 on 2026-04-16 and v1.1 on 2026-04-23.
**Active milestone:** v1.2 Compatibility + UX Alignment.
**Tech stack:** Rust stable, Cargo workspace, winnow (parser), clap (CLI), tokio (async watching), serde_json, directories, tempfile, rstest, insta.
**Crates:** `crates/todotxt-core` (library) + `crates/todotxt-cli` (binary) + `crates/todotxt-tui` (binary).
**Test baseline:** 207 tests passing, 0 warnings, 0 clippy issues.

**Existing C# codebase:** C# .NET Framework 4.0 WPF app (Windows only). Version 3.3.1.0. Located at `Client/`, `ToDoLib/`, `TodoTests/`. Reference implementation for format and behavior.

## Constraints

- **Format**: Strict todo.txt spec — no proprietary extensions that break interop
- **Platform**: Must compile and run natively on Windows, Linux, macOS
- **CLI output**: Must support structured JSON output for agent consumption
- **Compat**: Existing C# app continues to work independently

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust for core + CLI first | Foundation needed before any UI work; most agent-usable surface | ✓ Shipped v1.0 |
| Strict todo.txt format | Interop with todo.sh and ecosystem tools | ✓ Validated |
| Cargo workspace | Separate crates for core library, CLI, future TUI/GUI | ✓ Clean separation |
| JSON output flag for CLI | Enables AI agent skills to consume structured task data | ✓ Validated via integration tests |
| winnow for parser | Zero-copy, single-pass, composable; better ergonomics than nom | ✓ Worked well |
| `#![deny(warnings)]` in both crates | Enforce code quality from day 1 | ✓ 0 warnings at ship |
| Preserve todotxt.net behavioral parity in TUI UX | Reduce migration friction for existing users | 🚧 v1.2 |

## Evolution

**After each milestone** (via `/gsd-complete-milestone`):
1. Move Active requirements shipped → Validated with version reference
2. Add new requirements for next milestone to Active
3. Update Context with current state
4. Audit Out of Scope — reasons still valid?

---
*Last updated: 2026-04-23 for v1.2 milestone start*

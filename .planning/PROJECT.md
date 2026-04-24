# todotxt.net — Rust Port

## Current State

v1.0, v1.1, and v1.2 shipped. Core library, CLI, and TUI are complete with full todo.sh
compatibility and UX alignment.

Milestone archives:

- .planning/milestones/v1.0-ROADMAP.md
- .planning/milestones/v1.1-ROADMAP.md
- .planning/milestones/v1.2-ROADMAP.md

## Current Milestone: v1.3 Feature/Hotkey Parity with todotxt.net

**Goal:** Make users switching from todotxt.net feel comfortable and familiar in the Rust TUI.

**Target features:**

- Multi-selection in the TUI: anchored shift-range selection plus a visual-line style selection mode for disjoint picks
- Bulk actions on selected tasks, especially delete and append text
- Token-aware normalization across text-edit paths so appended or edited `@context`, `+project`, priority, `t:`, `due:`, and similar fields land in the correct todo.txt positions
- Hotkey and interaction parity grounded in todotxt.net help, screenshots, and docs

---

## What This Is

A Rust port of todotxt.net — a todo.txt manager that runs cross-platform (Windows, Linux, macOS). The project provides a strict todo.txt-compatible core library (`todotxt-core`) plus a full-featured CLI (`todotxt-cli`) for human and AI agent use. A TUI (interactive terminal UI) and native GUI (desktop app) are seeded for future milestones.

## Core Value

A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## Product Snapshot

**v1.0, v1.1, and v1.2 shipped.** Core library, CLI, and TUI are complete.
**Active milestone:** none — beginning v1.3 planning.

- `todotxt-core`: parser, Task model, TaskList CRUD, filter engine, sort engine, file watching, portable mode
- `todotxt-cli`: 25+ commands — read, write, enrichment, bulk operations, todo.sh compat aliases, structured JSON output, TOML config, named presets, shell completions
- `todotxt-tui`: keyboard-driven terminal UI with add/edit/delete, filtering/sorting, grouping, deferred toggle, presets, themes, and auto-reload
- 250+ tests passing, 0 clippy warnings, `#![deny(warnings)]` enforced in all crates
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

### Validated (v1.2)

- ✓ todo.sh compatibility layer (9 aliases + listpri/listall/deduplicate + `--compat`/`--all`) — v1.2
- ✓ TUI filter Esc cancel/restore behavior — v1.2
- ✓ TUI status bar theme label removed (always omitted) — v1.2
- ✓ TUI grouping by sort key (`g` toggle, contiguous alpha groups, group headers) — v1.2
- ✓ TUI filter definition panel with TOML persistence (`F` key, numbered preset slots) — v1.2
- ✓ Deferred-task `t:` toggle (`h` key, DIM styling, suppress-by-default) — v1.2

### Active (v1.3)

- [ ] TUI multi-selection parity: shift-range selection and disjoint selection mode
- [ ] Bulk task operations on selected rows: delete, append, and related high-value parity actions
- [ ] Token-aware task text normalization across append/edit flows while preserving todo.txt semantics
- [ ] Hotkey/help parity audit against todotxt.net docs, screenshots, and observable behavior

### Planned (future milestone)

- [ ] GUI interface (native desktop)
- [ ] CI/CD release pipeline and package distribution

### Out of Scope

- Windows-specific features (system tray) in CLI — GUI milestone handles platform-specific features
- Interactive prompts / REPL — anti-feature for agent use; CLI must be scriptable

## Context

**Shipped:** v1.0 on 2026-04-16, v1.1 on 2026-04-23, v1.2 on 2026-04-24.
**Active milestone:** v1.3 Feature/Hotkey Parity with todotxt.net.
**Tech stack:** Rust stable, Cargo workspace, winnow (parser), clap (CLI), ratatui + tui-textarea + crossterm (TUI), tokio (async watching), serde_json, directories, tempfile, rstest, insta.
**Crates:** `crates/todotxt-core` (library) + `crates/todotxt-cli` (binary) + `crates/todotxt-tui` (binary).
**Test baseline:** 250+ tests passing, 0 warnings, 0 clippy issues.

**Existing C# codebase:** C# .NET Framework 4.0 WPF app (Windows only). Version 3.3.1.0. Located at `Client/`, `ToDoLib/`, `TodoTests/`. Reference implementation for format and behavior.

## Constraints

- **Format**: Strict todo.txt spec — no proprietary extensions that break interop
- **Platform**: Must compile and run natively on Windows, Linux, macOS
- **CLI output**: Must support structured JSON output for agent consumption
- **Compat**: Existing C# app continues to work independently

## Key Decisions

| Decision | Rationale | Outcome |
| -------- | --------- | ------- |
| Rust for core + CLI first | Foundation needed before any UI work; most agent-usable surface | ✓ Shipped v1.0 |
| Strict todo.txt format | Interop with todo.sh and ecosystem tools | ✓ Validated |
| Cargo workspace | Separate crates for core library, CLI, future TUI/GUI | ✓ Clean separation |
| JSON output flag for CLI | Enables AI agent skills to consume structured task data | ✓ Validated via integration tests |
| winnow for parser | Zero-copy, single-pass, composable; better ergonomics than nom | ✓ Worked well |
| `#![deny(warnings)]` in both crates | Enforce code quality from day 1 | ✓ 0 warnings at ship |
| Preserve todotxt.net behavioral parity in TUI UX | Reduce migration friction for existing users | ✓ Shipped v1.2 |
| Remove theme label from status bar entirely | Conditional show adds complexity with no user benefit | ✓ v1.2 |
| `G` jump-to-bottom removed from TUI | Hotkey conflict with grouping `g` toggle; not worth the confusion | ✓ v1.2 |
| For parity work, split authority by concern | Use todo.txt spec for task-text semantics and todotxt.net for interaction model/hotkeys | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each milestone** (via `/gsd-complete-milestone`):

1. Move Active requirements shipped → Validated with version reference
2. Add new requirements for next milestone to Active
3. Update Context with current state
4. Audit Out of Scope — reasons still valid?

---
Last updated: 2026-04-24 after v1.3 milestone kickoff.

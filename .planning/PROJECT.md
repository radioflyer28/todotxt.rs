# todotxt.net → Rust Port

## What This Is

A Rust port of todotxt.net — a todo.txt manager that runs cross-platform (Windows, Linux, macOS). The project provides a strict todo.txt-compatible core library plus three interface modes: a CLI (for humans and AI agent automation), a TUI (interactive terminal UI), and a native GUI (desktop app). The existing C# WPF app is the reference implementation.

## Core Value

A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## Current Milestone: v1.0 Rust Port — Core + CLI

**Goal:** Build the Rust core library and CLI with full feature parity to the existing C# app.

**Target features:**
- Rust `todotxt` core library: parser, writer, task model (strict todo.txt format)
- CLI: full CRUD, filtering, sorting, bulk operations, structured JSON output
- File watching, portable mode, settings/config persistence
- Cross-platform builds: Windows, Linux, macOS

## Requirements

### Validated

- ✓ todo.txt file parsing (Task model: priority, projects, contexts, due date, threshold date, completion) — existing C#
- ✓ Task list CRUD (add, edit, delete, complete, uncomplete) — existing C#
- ✓ Filtering by project, context, priority, due date — existing C#
- ✓ Sorting (priority, due date, alphabetical) — existing C#
- ✓ Autocomplete for @contexts, +projects, (priorities) — existing C#
- ✓ File watching (reload on external change) — existing C#
- ✓ Portable mode (settings beside exe) — existing C#
- ✓ System tray integration (Windows) — existing C#
- ✓ Settings persistence (font, window size, filter presets, sort order) — existing C#
- ✓ Multiple filter presets — existing C#

### Active

- [ ] Rust core library: todo.txt parser/writer (strict format)
- [ ] Rust Task model: priority, projects, contexts, due date, threshold date, body, completion
- [ ] Rust TaskList: CRUD operations, file I/O, file watching
- [ ] CLI: add, list, complete, delete, edit commands
- [ ] CLI: filter by project, context, priority, due date, text
- [ ] CLI: sort by priority, due date, alphabetical
- [ ] CLI: bulk operations (complete-all, delete-done, archive)
- [ ] CLI: structured JSON output mode for agent consumption
- [ ] CLI: settings/config persistence (cross-platform paths)
- [ ] CLI: portable mode (config beside binary)
- [ ] Cross-platform builds and CI (Windows, Linux, macOS)

### Out of Scope (v1.0)

- TUI interface — deferred to v1.1 (seed planted)
- GUI interface — deferred to v1.2 (seed planted)
- todo.sh compatibility layer — deferred (seed planted)
- Windows-specific features (system tray) in CLI — CLI is cross-platform; GUI milestone handles platform-specific features

## Context

**Existing codebase:** C# .NET Framework 4.0 WPF app (Windows only, x86). Version 3.3.1.0. Located at `Client/`, `ToDoLib/`, `ToDoTests/`, `CommonExtensions/`, `ColorFont/`. Architecture is hybrid MVVM + code-behind. Domain logic in `ToDoLib/` has no WPF dependency — it's the natural model for the Rust core library.

**todo.txt format:** Strict compatibility required — parseable by todo.sh and other todo.txt tools. Format: `(A) 2024-01-15 Task body +Project @Context due:2024-01-31`

**CLI output:** JSON mode (machine-readable) + human-readable default. The CLI is intended for use by AI agent skills (GitHub Copilot CLI, etc.).

**Technology:** Rust (stable), Cargo workspace. No existing Rust code — greenfield.

## Constraints

- **Format**: Strict todo.txt spec — no proprietary extensions that break interop
- **Platform**: Must compile and run natively on Windows, Linux, macOS
- **CLI output**: Must support structured JSON output for agent consumption
- **Compat**: Existing C# app continues to work independently — this is an additive port, not a migration

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust for core + CLI first | Foundation needed before any UI work; most agent-usable surface | — Pending |
| Strict todo.txt format | Interop with todo.sh and ecosystem tools | — Pending |
| Cargo workspace | Separate crates for core library, CLI, future TUI/GUI | — Pending |
| JSON output flag for CLI | Enables AI agent skills to consume structured task data | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-15 after v1.0 milestone initialization*

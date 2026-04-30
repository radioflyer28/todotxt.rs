# todotxt.net - Rust Port

## Current State

v1.0 through v1.4 are shipped. The Rust port now has a fully functional CLI, interactive TUI, and Kanban-style vertical pane layout — all verified and archived.

Milestone archives:

- .planning/milestones/v1.0-ROADMAP.md
- .planning/milestones/v1.1-ROADMAP.md
- .planning/milestones/v1.2-ROADMAP.md
- .planning/milestones/v1.3-ROADMAP.md
- .planning/milestones/v1.4-ROADMAP.md

## Current Milestone

None — v1.4 archived 2026-04-29. Run `/gsd-new-milestone` to begin the next milestone.

## Next Milestone Goals

- Decide which future-scope items move forward (GUI, packaging, CI/CD, or further TUI workflow expansion)
- Preserve the parity and discoverability standards established through v1.4

---

## What This Is

A Rust port of todotxt.net - a todo.txt manager that runs cross-platform (Windows, Linux, macOS). The project provides a strict todo.txt-compatible core library (todotxt-core) and a full-featured CLI (todotxt-cli), plus an interactive TUI (todotxt-tui).

## Core Value

A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## Product Snapshot

Shipped:

- todotxt-core: parser, Task model, TaskList CRUD, filter/sort engines, file watching, portable mode
- todotxt-cli: read/write/enrichment/bulk commands, todo.sh compatibility, JSON output, TOML config, completions
- todotxt-tui: keyboard-driven task workflows, grouping, deferred toggle, presets, parity keymaps/help
- Milestone v1.3 parity additions: canonical multi-selection, bulk delete/append, normalization integration, configurable keymaps, help overlay, and app-wide error log discoverability

## Requirements

### Validated (v1.0)

- Core parser/serializer CRUD/filter/sort/config/completion/cross-platform baseline delivered

### Validated (v1.1)

- TUI foundation delivered: navigation, task actions, filtering/sorting, presets, theme behavior

### Validated (v1.2)

- todo.sh compatibility layer and TUI UX alignment milestone delivered

### Validated (v1.3)

- SEL-01 through SEL-04 complete (selection foundation)
- BULK-01 through BULK-03 complete (bulk delete/append + selection count)
- NORM-01 through NORM-06 complete (shared normalization behavior)
- PAR-01 through PAR-03 complete (parity/discoverability/deviation documentation)
- KEY-01 through KEY-03 complete (configurable keymaps + fallback safety)

### Validated (v1.4)

- PANE-01 through PANE-05 (layout, focus, lifecycle, pane-scoped behavior)
- VIEW-01 and VIEW-02 (single-pane fallback and quick global toggle)
- CFG-01 through CFG-03 (config-defined panes and fallback safety)
- PATH-01 through PATH-03 (CLI path overrides and archive-path resolution semantics)

### Planned (future milestone)

- GUI interface (native desktop)
- CI/CD release pipeline and package distribution

## Context

Shipped: v1.0 (2026-04-16), v1.1 (2026-04-23), v1.2 (2026-04-24), v1.3 (2026-04-28)
Tech stack: Rust stable, Cargo workspace, winnow, clap, ratatui, tui-textarea, crossterm, tokio, serde_json
Crates: crates/todotxt-core, crates/todotxt-cli, crates/todotxt-tui

## Constraints

- Format: strict todo.txt interop
- Platform: native Windows/Linux/macOS support
- CLI output: structured mode for agent tooling

## Key Decisions

| Decision | Outcome |
| -------- | ------- |
| Rust for core + CLI first | Shipped and validated |
| Strict todo.txt compatibility | Preserved across milestones |
| Workspace split by crate | Scaled through v1.3 |
| Parity split authority (text semantics vs UX parity) | Applied through v1.3 |

## Archived Planning Snapshot

<details>
<summary>v1.3 kickoff snapshot (archived)</summary>

Previous active milestone scope targeted parity migration comfort: multi-selection,
bulk operations, normalization semantics, and help/hotkey parity grounded in todotxt.net references.
This scope is now complete and archived in .planning/milestones/v1.3-ROADMAP.md.

</details>

## Evolution

This document evolves at phase transitions and milestone boundaries.

After each phase transition:
1. Requirements invalidated move to Out of Scope with reason.
2. Requirements validated move to Validated with phase reference.
3. New requirements discovered are added to Active.
4. Key decisions are appended with rationale and outcome.
5. What This Is is updated if product reality drifts.

After each milestone:
1. Full review of all sections.
2. Core Value check.
3. Out of Scope audit.
4. Context refresh with current state.

---
Last updated: 2026-04-28 after v1.4 milestone kickoff.

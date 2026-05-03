# todotxt.net - Rust Port

## Current State

v1.0 through v1.5 are shipped. The Rust port now includes fast capture/edit flows, picker-driven metadata workflows, safer bulk operations, clipboard flows, and short-horizon undo in addition to the earlier CLI/TUI/pane foundations.

Milestone archives:

- .planning/milestones/v1.0-ROADMAP.md
- .planning/milestones/v1.1-ROADMAP.md
- .planning/milestones/v1.2-ROADMAP.md
- .planning/milestones/v1.3-ROADMAP.md
- .planning/milestones/v1.4-ROADMAP.md
- .planning/milestones/v1.5-ROADMAP.md

## Current Milestone

No active milestone is in execution. Next step is to define v1.6 via /gsd-new-milestone and generate a fresh .planning/REQUIREMENTS.md.

## Next Milestone Goals

- Complete runtime manual validation debt recorded in v1.5 audit (clipboard runtime checks).
- Decide whether to keep or replace UNDO-03 silent-feedback override behavior.
- Improve Nyquist validation coverage for phases with missing or partial validation artifacts.
- Define and prioritize post-v1.5 product goals (GUI track, release automation, or focused TUI/CLI hardening).

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

### Archived (v1.5)

- CAP-01 through CAP-05 (fast capture/edit and property picker workflows)
- DATE-01 through DATE-04 (month-aware date autocomplete and weekday-labeled suggestions)
- TAG-01 through TAG-05 (quick context/project setters, autocomplete, and token safety)
- BULK-01 through BULK-03 (high-impact bulk action safety)
- CLIP-01 through CLIP-04 (basic clipboard cut/copy/paste workflows)
- UNDO-01 through UNDO-03 (short-horizon recovery path)
- META-01 through META-02 (todo.txt-native metadata with hierarchical token conventions)
- VIEW-03 (view consistency across capture and mutation flows)

### Validated (v1.5)

- CAP-01 through CAP-05 — shipped and verified (phase 38 backfill closed evidence gaps)
- DATE-01 through DATE-04 — shipped and verified
- TAG-01 through TAG-05 — shipped and verified
- BULK-01 through BULK-03 — shipped and verified
- CLIP-01 through CLIP-04 — shipped, with runtime human-check debt accepted at close
- UNDO-01 through UNDO-03 — shipped, UNDO-03 closed via accepted override
- META-01 through META-02 — shipped and verified (META-01 orphan resolved)
- VIEW-03 — shipped and verified

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
Last updated: 2026-05-01 after v1.5 milestone completion.

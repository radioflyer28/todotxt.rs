# todotxt.net - Rust Port

## Current State

v1.0 through v1.6 are shipped. The Rust port now features a full-featured power-user TUI with archive workflow, bulk mark-done, external `$EDITOR` integration, filter history, multi-dimensional view presets, view state persistence, independent group-by controls, filter input autocomplete, and pane task movement — all on top of the earlier CLI/TUI/pane foundations.

Milestone archives:

- .planning/milestones/v1.0-ROADMAP.md
- .planning/milestones/v1.1-ROADMAP.md
- .planning/milestones/v1.2-ROADMAP.md
- .planning/milestones/v1.3-ROADMAP.md
- .planning/milestones/v1.4-ROADMAP.md
- .planning/milestones/v1.5-ROADMAP.md
- .planning/milestones/v1.6-ROADMAP.md

## Next Milestone

**v1.7 — (to be planned)**

Run `/gsd-new-milestone` to define v1.7 requirements and roadmap.

Known candidate features for v1.7 consideration:
- Recurring task support (`rec:` extension) — SEED-010
- done.txt rotation (log-file style) — SEED-016
- Seed registry cleanup (mark addressed seeds as shipped)

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

### Validated (v1.6)

- ARCH-01 through ARCH-03 — archive workflow shipped and verified (Phase 39)
- BDONE-01, BDONE-02 — bulk mark-done shipped and verified (Phase 39)
- XEDIT-01 through XEDIT-03 — external editor with `RawModeGuard` shipped and verified (Phase 39)
- AC-01 — `+` project autocomplete verified correct (Phase 39)
- GRP-01 through GRP-04 — independent group-by per pane shipped and verified (Phase 40)
- TST-01, TST-02 — all Phase 22 manual gaps automated; `make_app_with_config` helper added (Phase 40)
- PRST-01, PRST-02 — multi-dimensional view presets shipped and verified (Phase 41)
- FHIST-01 through FHIST-03 — session filter history + `Ctrl+R` shipped and verified (Phase 41)
- PMOVE-01 through PMOVE-03 — pane task movement via tag mutation shipped and verified (Phase 41 + Phase 44 dispatch fix)
- AC-02 through AC-04 — filter input autocomplete with cursor-aware narrowing shipped and verified (Phase 42)
- PRSV-01 through PRSV-03 — `tui-state.toml` view state persistence shipped and verified (Phase 43)

### Planned (future milestone)

- GUI interface (native desktop)
- CI/CD release pipeline and package distribution

## Context

Shipped: v1.0 (2026-04-16), v1.1 (2026-04-23), v1.2 (2026-04-24), v1.3 (2026-04-28), v1.4 (2026-04-29), v1.5 (2026-05-01), v1.6 (2026-05-06)
Tech stack: Rust stable, Cargo workspace, winnow, clap, ratatui, tui-textarea, crossterm, tokio, serde_json, tempfile
Crates: crates/todotxt-core, crates/todotxt-cli, crates/todotxt-tui
Tests: 215 passing (todotxt-tui); 0 failing
v1.6 additions: archive workflow, bulk mark-done, external editor, filter autocomplete, view state persistence (tui-state.toml), group-by decoupling, full view presets, filter history, pane task movement

## Constraints

- Format: strict todo.txt interop
- Platform: native Windows/Linux/macOS support
- CLI output: structured mode for agent tooling

## Key Decisions

| Decision | Outcome |
| -------- | ------- |
| Rust for core + CLI first | Shipped and validated |
| Strict todo.txt compatibility | Preserved across milestones |
| Workspace split by crate | Scaled through v1.6 |
| Parity split authority (text semantics vs UX parity) | Applied through v1.3 |
| Group-by category separate from sort order | Validated v1.6 — independent per pane |
| View state in sidecar `tui-state.toml` (not config.toml) | Validated v1.6 — config = defaults, state = session overrides |
| Filter history session-only (not persisted) | Validated v1.6 — session ring covers use case; cross-session deferred |
| `accept_filter_completion` uses local `AcceptResult` enum | v1.6 — required by Rust borrow checker to extract action before dropping autocomplete borrow |
| BUG-41-01 fix: `KeyModifiers::NONE` guard on Left/Right nav arms | v1.6 — Phase 44 TDD; plain Right still navigates, Ctrl+Right now reaches pane_move_task |

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
*Last updated: 2026-05-06 after v1.6 milestone completion.*

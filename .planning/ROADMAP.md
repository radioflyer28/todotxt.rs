# Roadmap: todotxt.net — Rust Port

## Milestones

- ✅ v1.0 Core Library + CLI — shipped 2026-04-16
  Archive: .planning/milestones/v1.0-ROADMAP.md
- ✅ v1.1 TUI Interface — shipped 2026-04-23
  Archive: .planning/milestones/v1.1-ROADMAP.md
- 🚧 v1.2 Compatibility + UX Alignment — active

## v1.2 Scope

- todo.sh compatibility layer
- TUI UX polish: filter Esc cancel/restore behavior
- TUI UX polish: conditionally show theme label in status bar
- TUI UX polish: align grouping/sorting behavior with todotxt.net UI (group tasks with identical project/context/sort keys)
- TUI UX polish: align filter definition layout with todotxt.net UI and persist configured filters to TOML
- TUI UX polish: investigate deferred-task (`t:`) support parity and implement when confirmed

## Planned Phases

- [x] **Phase 14: Compat Discovery + Spec Lock** — complete 2026-04-23
  - Confirmed todo.sh compatibility surface (CLI semantics, edge cases, output conventions)
  - Confirmed deferred-task parity requirements (`t:` behavior) and locked decision
  - Produced 14-COMPAT-SPEC.md and 14-DEFER-SPEC.md implementation contracts
  - **Plans:** 1 plan
  - Plans:
    - [x] 14-01-PLAN.md — Produce COMPAT-SPEC.md and DEFER-SPEC.md implementation contracts

- [x] **Phase 15: todo.sh Compatibility Layer** *(completed 2026-04-23)*
  - Implement compatibility commands/aliases and argument behavior
  - Add regression tests for compatibility contracts and exit-code behavior
  - **Plans:** 3 plans
  - Plans:
    - [x] 15-01-PLAN.md — Add 9 compat aliases + new CLI variants (Listpri, Listall, Deduplicate) + --all/--compat flags to cli.rs
    - [x] 15-02-PLAN.md — Implement listpri, listall, deduplicate + wire --all/--compat in list.rs + register in mod.rs + dispatch in main.rs
    - [x] 15-03-PLAN.md — Regression tests for full compat surface (aliases, --all, --compat, listpri, listall, deduplicate)

- [ ] **Phase 16: TUI Filter UX Alignment**
  - Implement Esc cancel/restore behavior in filter flows
  - Rework filter-definition panel layout to match todotxt.net mental model
  - Persist configured filters in TOML and reload reliably
  - **Requirements:** V12-TUI-FILTER-01, V12-TUI-FILTER-02, V12-TUI-FILTER-03
  - **Plans:** 3 plans
  - Plans:
    - [ ] 16-01-PLAN.md — Esc snapshot/restore for quick filter panel (V12-TUI-FILTER-01)
    - [ ] 16-02-PLAN.md — TOML serialize + TuiConfig::save() atomic write (V12-TUI-FILTER-03)
    - [ ] 16-03-PLAN.md — F-key definition panel: layout, key handlers, preset persistence wiring (V12-TUI-FILTER-01, V12-TUI-FILTER-02, V12-TUI-FILTER-03)

- [ ] **Phase 17: TUI Grouping/Sorting Alignment + Status Polish**
  - Group tasks by shared sort keys (project/context/other configured key)
  - Conditionally show theme label in status bar (hide when default/no-value)
  - Preserve keyboard and rendering performance expectations

- [ ] **Phase 18: Validation + Ship Readiness**
  - Integration and UAT pass for compatibility and TUI UX changes
  - Milestone audit, docs updates, and close-out prep

## Backlog

- GUI interface (native desktop)
- CI/CD release pipeline and package distribution

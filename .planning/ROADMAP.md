# Roadmap: todotxt.net — Rust Port

## Milestones

- ✅ v1.0 Core Library + CLI — shipped 2026-04-16 (archive: .planning/milestones/v1.0-ROADMAP.md)
- ✅ v1.1 TUI Interface — shipped 2026-04-23 (archive: .planning/milestones/v1.1-ROADMAP.md)
- ✅ v1.2 Compatibility + UX Alignment — shipped 2026-04-24 (archive: .planning/milestones/v1.2-ROADMAP.md)
- ✅ v1.3 Feature/Hotkey Parity with todotxt.net — shipped 2026-04-28 (archive: .planning/milestones/v1.3-ROADMAP.md)
- 🚧 v1.4 Kanban-Style Vertical Panes — active

## Active Milestone

v1.4 Kanban-Style Vertical Panes

## v1.4 Scope

- Multi-pane Kanban-style vertical layout in the TUI
- Per-pane sort/group/filter state and independent list view behavior
- Pane lifecycle hotkeys for creation and deletion
- One-key toggle to hide/show panes and restore default single-pane view
- Config-defined panes in config.toml with per-pane defaults for sort/group/filter

## Planned Phases

- [ ] **Phase 24: Pane Model + Layout Foundation**
	- Introduce pane data model and active-pane focus mechanics
	- Render vertical pane containers with robust fallback to single-pane view
	- **Requirements:** PANE-01, PANE-02, VIEW-01
	- **Plans:** 3 plans
	- Plans:
		- [ ] 24-01-PLAN.md — Pane state model and focus selection behavior
		- [ ] 24-02-PLAN.md — Vertical pane layout rendering in TUI task view
		- [ ] 24-03-PLAN.md — Single-pane fallback path and layout safety guards

- [ ] **Phase 25: Per-Pane Query Behavior (Sort/Group/Filter)**
	- Track sort/group/filter independently for each pane
	- Route existing query hotkeys to the active pane context
	- **Requirements:** PANE-03, PANE-04
	- **Plans:** 3 plans
	- Plans:
		- [ ] 25-01-PLAN.md — Pane-scoped filter query and preset application
		- [ ] 25-02-PLAN.md — Pane-scoped sort/group state and rendering status
		- [ ] 25-03-PLAN.md — Navigation and action safety across pane boundaries

- [ ] **Phase 26: Pane Management + Quick Hide/Show**
	- Add hotkeys for pane creation and deletion
	- Add one-key global pane hide/show that restores default single-pane view
	- **Requirements:** PANE-05, VIEW-02
	- **Plans:** 3 plans
	- Plans:
		- [ ] 26-01-PLAN.md — Create/delete pane hotkeys and guardrails
		- [ ] 26-02-PLAN.md — Global pane visibility toggle and restore semantics
		- [ ] 26-03-PLAN.md — Help/status updates for pane controls and discoverability

- [ ] **Phase 27: Config-Defined Panes + Validation + Ship Readiness**
	- Load pane definitions from config.toml with per-pane sort/group/filter defaults
	- Validate config fallback behavior and ship-readiness docs/tests
	- **Requirements:** CFG-01, CFG-02, CFG-03
	- **Plans:** 3 plans
	- Plans:
		- [ ] 27-01-PLAN.md — Config schema and startup pane materialization
		- [ ] 27-02-PLAN.md — Invalid pane config fallback/warning behavior
		- [ ] 27-03-PLAN.md — Verification, validation, and milestone close-out

## Backlog

- GUI interface (native desktop)
- CI/CD release pipeline and package distribution

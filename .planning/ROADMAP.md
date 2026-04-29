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
- CLI overrides for todo/archive/config file locations with deterministic fallback behavior

## Planned Phases

- [x] **Phase 24: Pane Model + Layout Foundation**
	- Introduce pane data model and active-pane focus mechanics
	- Render vertical pane containers with robust fallback to single-pane view
	- **Requirements:** PANE-01, PANE-02, VIEW-01
	- **Plans:** 3 plans
	- Plans:
		- [x] 24-01-PLAN.md — Pane state model and focus selection behavior
		- [x] 24-02-PLAN.md — Vertical pane layout rendering in TUI task view
		- [x] 24-03-PLAN.md — Single-pane fallback path and layout safety guards

- [x] **Phase 25: Per-Pane Query Behavior (Sort/Group/Filter)** — completed 2026-04-28
	- Track sort/group/filter independently for each pane
	- Route existing query hotkeys to the active pane context
	- **Requirements:** PANE-03, PANE-04
	- **Plans:** 3 plans
	- Plans:
		- [x] 25-01-PLAN.md — Pane-scoped filter query and preset application
		- [x] 25-02-PLAN.md — Pane-scoped sort/group state and rendering status
		- [x] 25-03-PLAN.md — Navigation and action safety across pane boundaries

- [x] **Phase 26: Pane Management + Quick Hide/Show** — completed 2026-04-28
	- Add hotkeys for pane creation and deletion
	- Add one-key global pane hide/show that restores default single-pane view
	- **Requirements:** PANE-05, VIEW-02
	- **Plans:** 3 plans
	- Plans:
		- [x] 26-01-PLAN.md — Create/delete pane hotkeys and guardrails
		- [x] 26-02-PLAN.md — Global pane visibility toggle and restore semantics
		- [x] 26-03-PLAN.md — Help/status updates for pane controls and discoverability

- [x] **Phase 27: Config-Defined Panes + Validation + Ship Readiness** — completed 2026-04-28
	- Load pane definitions from config.toml with per-pane sort/group/filter defaults
	- Add CLI file-path override flags and archive path defaulting for alternate todo.txt paths
	- Validate config/path fallback behavior and ship-readiness docs/tests
	- **Requirements:** CFG-01, CFG-02, CFG-03, PATH-01, PATH-02, PATH-03
	- **Plans:** 3 plans
	- Plans:
		- [x] 27-01-PLAN.md — Config schema updates for panes and CLI path override inputs
		- [x] 27-02-PLAN.md — CLI override resolution rules (todo/archive/config) and fallback behavior
		- [x] 27-03-PLAN.md — Verification, validation, and milestone close-out

- [x] **Phase 28: Per-Pane State Consistency Fixes** — gap closure — completed 2026-04-29
	- Fix FilterDefining dialog (`F` key) to write filter query to the active pane, not global state
	- Route non-Normal mode draw() dispatch to per-pane render path (eliminates global task list bleed-through)
	- Rebuild all visible panes on task mutations and FileChanged events (sibling panes no longer stale)
	- Fix cursor reanchor to use per-pane `pane.selected` instead of global `self.selected`
	- Add integration test for FilterDefining in multi-pane mode
	- **Requirements:** PANE-03, PANE-04
	- **Gap Closure:** Closes FAIL-1 + WARN-1 + WARN-2 + WARN-3 + WARN-4 from v1.4-MILESTONE-AUDIT.md
	- **Plans:** 1 plan
	- Plans:
		- [x] 28-01-PLAN.md — Per-pane state fixes (FilterDefining, status bar, draw dispatch, rebuild_all_panes, cursor reanchor) + integration test

- [x] **Phase 29: Verification Artifacts + Status Bar Fix + Metadata Cleanup** — gap closure — completed 2026-04-29
	- Produce Phase 24 VERIFICATION.md (covers PANE-01, PANE-02, VIEW-01)
	- Produce Phase 25 VERIFICATION.md (covers PANE-03, PANE-04)
	- Fix `render_status_bar` panes-hidden guard (WARN-1: status bar misleading when panes hidden)
	- Update REQUIREMENTS.md: all 13 v1.4 checkboxes and traceability table to reflect verified deliverables
	- Update ROADMAP.md: Phase 25 plan checkboxes `[ ]` → `[x]` (stale since execution)
	- **Requirements:** PANE-01, PANE-02, PANE-03, PANE-04, VIEW-01, VIEW-02
	- **Gap Closure:** Closes partial-status for phases 24/25; closes WARN-1 from v1.4-MILESTONE-AUDIT.md
	- **Plans:** 2 plans
	- Plans:
		- [x] 29-01-PLAN.md — Phase 24 VERIFICATION.md (PANE-01/02/VIEW-01) + Phase 25 VERIFICATION.md (PANE-03/04/VIEW-02)
		- [x] 29-02-PLAN.md — ROADMAP.md Phase 25 checkbox fix + REQUIREMENTS.md all-13-complete update

- [x] **Phase 30: Nyquist Validation — All v1.4 Phases** — gap closure
	- Run Nyquist validation for Phase 24, 25, 26, 27
	- Produce VALIDATION.md for each phase
	- **Requirements:** (all v1.4)
	- **Gap Closure:** Closes all four Nyquist compliance gaps from v1.4-MILESTONE-AUDIT.md
	- **Plans:** 4 plans
	- Plans:
		- [x] 30-01-PLAN.md — Phase 24 VALIDATION.md (PANE-01/02/VIEW-01) — nyquist_compliant: false, 12 tasks, 4 manual-only
		- [x] 30-02-PLAN.md — Phase 25 VALIDATION.md (PANE-03/04/VIEW-02) — nyquist_compliant: true, 18 integration tests
		- [x] 30-03-PLAN.md — Phase 26 VALIDATION.md (PANE-05) — nyquist_compliant: false, pane lifecycle manual-only
		- [x] 30-04-PLAN.md — Phase 27 VALIDATION.md (CFG-01/02/03/PATH-01/02/03) — nyquist_compliant: true, 13 automated tests

- [ ] **Phase 31: Single-Pane Filter/Sort/Group Bridge Fix** — gap closure
	- Fix PANE-03 + PANE-04 regression: query actions (filter/sort/group) write to per-pane state but render_task_list() reads stale global fields in single-pane and panes_hidden modes
	- Sync global filter_query/sort_order/grouping from active pane in rebuild_and_reanchor() when in single-pane or panes_hidden mode
	- Fix FilterDefining panel (F key) pre-fill to read from active_pane().filter_query instead of global self.filter_query
	- **Requirements:** PANE-03, PANE-04
	- **Gap Closure:** Closes GAP-1 and GAP-2 from v1.4-MILESTONE-AUDIT.md Rev 2
	- **Plans:** 1 plan
	- Plans:
		- [ ] 31-01-PLAN.md — Bridge fix: sync global fields + filter pre-fill fix + regression test

## Backlog

- GUI interface (native desktop)
- CI/CD release pipeline and package distribution







# Roadmap: todotxt.net — Rust Port

## Milestones

- ✅ v1.0 Core Library + CLI — shipped 2026-04-16
  Archive: .planning/milestones/v1.0-ROADMAP.md
- ✅ v1.1 TUI Interface — shipped 2026-04-23
  Archive: .planning/milestones/v1.1-ROADMAP.md
- ✅ v1.2 Compatibility + UX Alignment — shipped 2026-04-24
  Archive: .planning/milestones/v1.2-ROADMAP.md
- 🚧 v1.3 Feature/Hotkey Parity with todotxt.net — active

## v1.3 Scope

- TUI multi-selection parity: shift-range selection and keyboard-driven disjoint selection
- Bulk delete and append actions over selected tasks
- Todo.txt-aware normalization across append and edit flows
- Configurable keymap overrides in `config.toml`
- Help and hotkey parity grounded in todotxt.net docs and screenshots

## Planned Phases

- [ ] **Phase 19: Selection Model + Multi-Select Foundation**
  - Add canonical task selection state that survives grouping, sorting, filtering, and reloads
  - Support contiguous range selection and keyboard-driven disjoint selection
  - Ensure non-task rows such as group headers are never selected
  - **Requirements:** SEL-01, SEL-02, SEL-03, SEL-04
  - **Plans:** 3 plans
  - Plans:
    - [ ] 19-01-PLAN.md — Canonical selection model + anchor tracking + grouped-row safety
    - [ ] 19-02-PLAN.md — Shift-range selection + disjoint selection mode key handling and rendering
    - [ ] 19-03-PLAN.md — Selection persistence across regroup, refilter, resort, and reload

- [ ] **Phase 20: Bulk Actions + Selection UX**
  - Turn multi-selection into safe bulk delete and bulk append flows
  - Surface selection count and selection mode clearly in the TUI
  - Preserve deterministic behavior when visible-row order differs from task order
  - **Requirements:** BULK-01, BULK-02, BULK-03
  - **Plans:** 3 plans
  - Plans:
    - [ ] 20-01-PLAN.md — Bulk delete confirmation and descending-index mutation safety
    - [ ] 20-02-PLAN.md — Bulk append workflow over selected tasks
    - [ ] 20-03-PLAN.md — Selection count/status/help UX polish for bulk actions

- [ ] **Phase 21: Smart Text Normalization**
  - Normalize recognized todo.txt metadata during append and edit flows
  - Preserve plain text and unknown metadata verbatim
  - Centralize normalization rules in `todotxt-core`
  - **Requirements:** NORM-01, NORM-02, NORM-03, NORM-04, NORM-05, NORM-06
  - **Plans:** 3 plans
  - Plans:
    - [ ] 21-01-PLAN.md — Extend `todotxt-core` normalization/build helpers for recognized metadata
    - [ ] 21-02-PLAN.md — Route TUI append flows through shared normalization helpers
    - [ ] 21-03-PLAN.md — Route TUI edit/update flows through shared normalization helpers and preserve unknown text

- [ ] **Phase 22: Keymap + Help Parity**
  - Align implemented hotkeys with todotxt.net where practical
  - Support configurable key overrides in `config.toml`
  - Make active bindings and deliberate deviations discoverable in-app
  - **Requirements:** PAR-01, PAR-02, PAR-03, KEY-01, KEY-02, KEY-03
  - **Plans:** 3 plans
  - Plans:
    - [ ] 22-01-PLAN.md — Keymap schema in config and runtime binding resolution
    - [ ] 22-02-PLAN.md — Safe fallback behavior for invalid or conflicting key overrides
    - [ ] 22-03-PLAN.md — Help/status parity pass and deviation documentation

- [ ] **Phase 23: Validation + Ship Readiness**
  - Parity-focused UAT for selection, bulk actions, normalization, and keymap overrides
  - Regression coverage, docs updates, milestone audit, and close-out
  - **Plans:** 4 plans
  - Plans:
    - [ ] 23-01-PLAN.md — Write UAT checklist for parity workflows and confirm regression suite green
    - [ ] 23-02-PLAN.md — Requirements close-out + help/README/CHANGELOG updates
    - [ ] 23-03-PLAN.md — Human UAT checkpoint against todotxt.net parity expectations
    - [ ] 23-04-PLAN.md — Milestone audit and close-out

## Phase Detail Sections

### Phase 19: Selection Model + Multi-Select Foundation

Goal: add canonical multi-selection to the TUI without breaking grouped rendering or filtered/reloaded views.
Status: planned
Plans:

- [ ] 19-01-PLAN.md — Canonical selection model + anchor tracking + grouped-row safety
- [ ] 19-02-PLAN.md — Shift-range selection + disjoint selection mode key handling and rendering
- [ ] 19-03-PLAN.md — Selection persistence across regroup, refilter, resort, and reload

### Phase 20: Bulk Actions + Selection UX

Goal: expose selected-task workflows that feel safe and obvious to todotxt.net users.
Status: planned
Plans:

- [ ] 20-01-PLAN.md — Bulk delete confirmation and descending-index mutation safety
- [ ] 20-02-PLAN.md — Bulk append workflow over selected tasks
- [ ] 20-03-PLAN.md — Selection count/status/help UX polish for bulk actions

### Phase 21: Smart Text Normalization

Goal: make append/edit flows todo.txt-aware while preserving user text.
Status: planned
Plans:

- [ ] 21-01-PLAN.md — Extend `todotxt-core` normalization/build helpers for recognized metadata
- [ ] 21-02-PLAN.md — Route TUI append flows through shared normalization helpers
- [ ] 21-03-PLAN.md — Route TUI edit/update flows through shared normalization helpers and preserve unknown text

### Phase 22: Keymap + Help Parity

Goal: make parity workflows discoverable by default and configurable when users need different bindings.
Status: planned
Plans:

- [ ] 22-01-PLAN.md — Keymap schema in config and runtime binding resolution
- [ ] 22-02-PLAN.md — Safe fallback behavior for invalid or conflicting key overrides
- [ ] 22-03-PLAN.md — Help/status parity pass and deviation documentation

### Phase 23: Validation + Ship Readiness

Goal: verify v1.3 selection, bulk action, normalization, and keymap behavior before ship.
Status: planned
Plans:

- [ ] 23-01-PLAN.md — Write UAT checklist for parity workflows and confirm regression suite green
- [ ] 23-02-PLAN.md — Requirements close-out + help/README/CHANGELOG updates
- [ ] 23-03-PLAN.md — Human UAT checkpoint against todotxt.net parity expectations
- [ ] 23-04-PLAN.md — Milestone audit and close-out

## Backlog

- GUI interface (native desktop)
- CI/CD release pipeline and package distribution

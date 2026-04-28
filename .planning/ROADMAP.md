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

- [x] **Phase 19: Selection Model + Multi-Select Foundation**
  - Add canonical task selection state that survives grouping, sorting, filtering, and reloads
  - Support contiguous range selection and keyboard-driven disjoint selection
  - Ensure non-task rows such as group headers are never selected
  - **Requirements:** SEL-01, SEL-02, SEL-03, SEL-04
  - **Plans:** 3 plans
  - Plans:
    - [x] 19-01-PLAN.md — Canonical selection model + anchor tracking + grouped-row safety
    - [x] 19-02-PLAN.md — Shift-range selection + disjoint selection mode key handling and rendering
    - [x] 19-03-PLAN.md — Selection persistence across regroup, refilter, resort, and reload

- [x] **Phase 20: Bulk Actions + Selection UX**
  - Turn multi-selection into safe bulk delete and bulk append flows
  - Surface selection count and selection mode clearly in the TUI
  - Preserve deterministic behavior when visible-row order differs from task order
  - **Requirements:** BULK-01, BULK-02, BULK-03
  - **Plans:** 3 plans
  - Plans:
    - [x] 20-01-PLAN.md — Bulk delete confirmation and descending-index mutation safety
    - [x] 20-02-PLAN.md — Bulk append workflow over selected tasks
    - [x] 20-03-PLAN.md — Selection count/status/help UX polish for bulk actions

- [x] **Phase 21: Smart Text Normalization**
  - Normalize recognized todo.txt metadata during append and edit flows
  - Preserve plain text and unknown metadata verbatim
  - Centralize normalization rules in `todotxt-core`
  - **Requirements:** NORM-01, NORM-02, NORM-03, NORM-04, NORM-05, NORM-06
  - **Plans:** 3 plans
  - Plans:
    - [x] 21-01-PLAN.md — Extend `todotxt-core` normalization/build helpers for recognized metadata
    - [x] 21-02-PLAN.md — Route TUI append flows through shared normalization helpers
    - [x] 21-03-PLAN.md — Route TUI edit/update flows through shared normalization helpers and preserve unknown text

- [x] **Phase 22: Keymap + Help Parity**
  - Align implemented hotkeys with todotxt.net where practical
  - Support configurable key overrides in `config.toml`
  - Make active bindings and deliberate deviations discoverable in-app
  - **Requirements:** PAR-01, PAR-02, PAR-03, KEY-01, KEY-02, KEY-03
  - **Plans:** 3 plans
  - Plans:
    - [x] 22-01-PLAN.md — Keymap schema in config and runtime binding resolution
    - [x] 22-02-PLAN.md — Safe fallback behavior for invalid or conflicting key overrides
    - [x] 22-03-PLAN.md — Help/status parity pass and deviation documentation

- [x] **Phase 23: Validation + Ship Readiness**
  - Run phase verifications for phases 20/21/22, Nyquist validation, docs close-out, UAT, and final audit
  - **Plans:** 5 plans
  - Plans:
    - [x] 23-01-PLAN.md — Phase verification run: gsd-verify-work for phases 20, 21, 22
    - [x] 23-02-PLAN.md — Nyquist validation close-out: gsd-validate-phase for phases 19 and 20
    - [x] 23-03-PLAN.md — Requirements + docs close-out: REQUIREMENTS.md, ROADMAP.md, CHANGELOG
    - [x] 23-04-PLAN.md — Human UAT checkpoint against todotxt.net parity expectations
    - [x] 23-05-PLAN.md — Final re-audit and milestone close-out

## Phase Detail Sections

### Phase 19: Selection Model + Multi-Select Foundation

Goal: add canonical multi-selection to the TUI without breaking grouped rendering or filtered/reloaded views.
Status: Complete
Plans:

- [x] 19-01-PLAN.md — Canonical selection model + anchor tracking + grouped-row safety
- [x] 19-02-PLAN.md — Shift-range selection + disjoint selection mode key handling and rendering
- [x] 19-03-PLAN.md — Selection persistence across regroup, refilter, resort, and reload

### Phase 20: Bulk Actions + Selection UX

Goal: expose selected-task workflows that feel safe and obvious to todotxt.net users.
Status: Complete
Plans:

- [x] 20-01-PLAN.md — Bulk delete confirmation and descending-index mutation safety
- [x] 20-02-PLAN.md — Bulk append workflow over selected tasks
- [x] 20-03-PLAN.md — Selection count/status/help UX polish for bulk actions

### Phase 21: Smart Text Normalization

Goal: make append/edit flows todo.txt-aware while preserving user text.
Status: Complete
**Plans:** 3 plans

Plans:
- [x] 21-01-PLAN.md — Implement `normalize_append` + `normalize_line` in todotxt-core with test suite
- [x] 21-02-PLAN.md — Add TUI config toggles and wire append flow through normalize_append
- [x] 21-03-PLAN.md — Wire TUI edit flow through normalize_line and add CLI `--normalize` flag

### Phase 22: Keymap + Help Parity

Goal: make parity workflows discoverable by default and configurable when users need different bindings.
Status: Complete
Plans:

- [x] 22-01-PLAN.md — Keymap schema in config and runtime binding resolution
- [x] 22-02-PLAN.md — Safe fallback behavior for invalid or conflicting key overrides
- [x] 22-03-PLAN.md — Help/status parity pass and deviation documentation

### Phase 23: Validation + Ship Readiness

Goal: close verification gaps from audit (phases 20/21/22), run Nyquist validation, docs close-out, UAT, and final milestone audit.
Status: Complete
Plans:

- [x] 23-01-PLAN.md — Phase verification run: gsd-verify-work for phases 20, 21, 22
- [x] 23-02-PLAN.md — Nyquist validation close-out: gsd-validate-phase for phases 19 and 20
- [x] 23-03-PLAN.md — Requirements + docs close-out: REQUIREMENTS.md, ROADMAP.md, CHANGELOG
- [x] 23-04-PLAN.md — Human UAT checkpoint against todotxt.net parity expectations
- [x] 23-05-PLAN.md — Final re-audit and milestone close-out

## Backlog

- GUI interface (native desktop)
- CI/CD release pipeline and package distribution

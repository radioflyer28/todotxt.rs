# Roadmap: todotxt.net — Rust Port

## Milestones

- ✅ v1.0 Core Library + CLI — shipped 2026-04-16 (archive: .planning/milestones/v1.0-ROADMAP.md)
- ✅ v1.1 TUI Interface — shipped 2026-04-23 (archive: .planning/milestones/v1.1-ROADMAP.md)
- ✅ v1.2 Compatibility + UX Alignment — shipped 2026-04-24 (archive: .planning/milestones/v1.2-ROADMAP.md)
- ✅ v1.3 Feature/Hotkey Parity with todotxt.net — shipped 2026-04-28 (archive: .planning/milestones/v1.3-ROADMAP.md)
- ✅ v1.4 Kanban-Style Vertical Panes — shipped 2026-04-29 (archive: .planning/milestones/v1.4-ROADMAP.md)
- 🚧 v1.5 Capture Flow + Bulk Safety + Clipboard + Undo — active

## Active Milestone

v1.5 Capture Flow + Bulk Safety + Clipboard + Undo — active

## v1.5 Scope

- One fast capture flow with predictable keybindings and minimal mode switching
- Quick context/project setters from normal mode (`@` and `+`) for active/selected tasks
- Autocomplete pickers for `@`/`+` tokens with match list, arrow navigation, and tab completion
- Date autocomplete for partial `due:`/`t:` tokens with month-valid day suggestions and weekday labels
- Safe bulk actions with affected-count preview and explicit cancel behavior
- Basic clipboard workflow: cut/copy selected tasks and paste to create new tasks
- New-task entry supports paste to enable fast duplicate-and-tweak workflows
- Short-horizon undo for destructive/high-impact actions
- Preserve todo.txt-native metadata model while supporting hierarchical token conventions
- Keep existing view flexibility (filter/sort/group) consistent across these flows

## Planned Phases

- [x] **Phase 33: Fast Capture + Property Pickers (`s` due, `i` priority)** (completed 2026-04-29)
  - Keep add/edit flows fast with predictable key behavior and minimal mode switching
  - Add due-date and priority pickers with overwrite semantics for active or selected tasks
  - Add quick context/project setters triggered by `@` and `+` in normal mode
  - Add autocomplete match list with arrow-key navigation and tab-to-complete for context/project setters
  - Add date autocomplete for partial `due:`/`t:` inputs, including valid day options and weekday labels, and align `s` picker suggestions
  - **Requirements:** CAP-01, CAP-02, CAP-03, CAP-04, TAG-01, TAG-02, TAG-04, TAG-05, DATE-01, DATE-02, DATE-03, DATE-04
  - **Plans:** 2 plans
    - [x] 33-01-PLAN.md — Date autocomplete and due-date picker
    - [x] 33-02-PLAN.md — Quick context/project setters with autocomplete

- [x] **Phase 34: Bulk Action Safety + Metadata Preservation**
 (completed 2026-04-30)

  - Add affected-count preview and cancel path for high-impact actions
  - Preserve non-target metadata, avoid duplicate tag tokens, and keep stable selection targeting in bulk operations
  - Add `i` priority picker overlay (CAP-04 gap from Phase 33)
  - **Requirements:** CAP-04, CAP-05, TAG-03, BULK-01, BULK-02, BULK-03
  - **Plans:** 3 plans
    - [x] 34-01-PLAN.md — `i` priority picker overlay (PriorityPickerState + handler + render + binding)
    - [x] 34-02-PLAN.md — Metadata preservation tests (TDD: with_priority/with_due_date round-trips)
    - [x] 34-03-PLAN.md — Count preview for T + s setter D-13 refactor + D wording

- [x] **Phase 35: Basic Clipboard Workflows**
 (completed 2026-04-30)
  - Implement cut/copy selected task text and paste-as-new-task behavior
  - Support paste during new-task entry (`n`) for duplicate-and-tweak workflows
  - **Requirements:** CLIP-01, CLIP-02, CLIP-03, CLIP-04
  - **Plans:** 2 plans
    - [x] 35-01-PLAN.md — Clipboard backend integration + `y` copy action
    - [x] 35-02-PLAN.md — Paste workflows (`p` in Normal mode + Ctrl+V in Adding mode)

- [x] **Phase 36: Recovery Path (Short-Horizon Undo)**
  - Implement lightweight undo for destructive/high-impact actions
  - Provide clear undo feedback and safe behavior when undo history is empty
  - **Requirements:** UNDO-01, UNDO-02, UNDO-03
  - **Plans:** 2 plans
    - [x] 36-01-PLAN.md — UndoEntry type + push_undo_entry/apply_undo core logic + Ctrl+Z dispatch (TDD)
    - [x] 36-02-PLAN.md — Wire push_undo_entry into all 10 mutation sites + integration tests

- [ ] **Phase 37: Metadata Flexibility + View Continuity**
  - Keep metadata todo.txt-native while supporting hierarchical tag conventions
  - Validate filter/sort/group behavior remains predictable across capture/bulk/clipboard/undo flows
  - **Requirements:** META-01, META-02, VIEW-03
  - **Plans:** 2 plans

## Backlog

- GUI interface (native desktop)
- CI/CD release pipeline and package distribution

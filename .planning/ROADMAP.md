# Roadmap: todotxt.net — Rust Port

## Milestones

- ✅ v1.0 Core Library + CLI — shipped 2026-04-16 (archive: .planning/milestones/v1.0-ROADMAP.md)
- ✅ v1.1 TUI Interface — shipped 2026-04-23 (archive: .planning/milestones/v1.1-ROADMAP.md)
- ✅ v1.2 Compatibility + UX Alignment — shipped 2026-04-24 (archive: .planning/milestones/v1.2-ROADMAP.md)
- ✅ v1.3 Feature/Hotkey Parity with todotxt.net — shipped 2026-04-28 (archive: .planning/milestones/v1.3-ROADMAP.md)
- ✅ v1.4 Kanban-Style Vertical Panes — shipped 2026-04-29 (archive: .planning/milestones/v1.4-ROADMAP.md)
- 🚧 v1.5 Task Properties + Workspace Quick Picker — active

## Active Milestone

v1.5 Task Properties + Workspace Quick Picker — active

## v1.5 Scope

- Fast metadata updates with picker-driven due-date and priority helpers
- Multi-select-safe property overwrites for due date and priority
- Workspace quick picker backed by labeled todo/done file pairs from config.toml
- Basic clipboard transfer flow across workspaces (copy, cut, paste raw task lines)
- Preserve todo.txt-native metadata model while supporting hierarchical token conventions
- Keep existing view flexibility (filter/sort/group) consistent across workspace switches

## Planned Phases

- [ ] **Phase 33: Property Picker Foundations (`s` due, `i` priority)**
  - Add due-date picker and priority picker interactions for active task
  - Define overwrite semantics for existing `due:` and `(A-Z)` values
  - **Requirements:** PROP-01, PROP-02
  - **Plans:** 2 plans

- [ ] **Phase 34: Multi-Select Property Application Safety**
  - Apply picker actions to selected task sets with stable targeting
  - Preserve non-target metadata while overwriting due/priority fields
  - **Requirements:** PROP-03, PROP-04
  - **Plans:** 2 plans

- [ ] **Phase 35: Config Workspaces + Quick File Picker**
  - Add config schema for labeled workspace entries (`label`, `todo_path`, `done_path`)
  - Implement quick picker to switch active workspace and reload task sources
  - **Requirements:** WS-01, WS-02, WS-03, WS-04
  - **Plans:** 3 plans

- [ ] **Phase 36: Clipboard Workflow Across Workspaces**
  - Implement copy/cut/paste task-line text behavior using basic clipboard semantics
  - Ensure copy/cut in one workspace can paste into another after switching
  - **Requirements:** CLIP-01, CLIP-02, CLIP-03, CLIP-04
  - **Plans:** 2 plans

- [ ] **Phase 37: Metadata Flexibility + View Continuity**
  - Keep metadata todo.txt-native while supporting hierarchical tag conventions
  - Validate filter/sort/group behavior remains predictable across workspace and bulk-edit flows
  - **Requirements:** META-01, META-02, VIEW-03
  - **Plans:** 2 plans

## Backlog

- GUI interface (native desktop)
- CI/CD release pipeline and package distribution

# Requirements: v1.4 Kanban-Style Vertical Panes

Defined: 2026-04-28
Core Value: A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## v1 Requirements

### Pane Layout and Focus

- [ ] PANE-01: User can view multiple vertical panes in the TUI, each pane showing a task list.
- [ ] PANE-02: User can switch active focus between panes using keyboard-only controls.

### Pane-Scoped Query Behavior

- [ ] PANE-03: Each pane maintains its own filter query independent from other panes.
- [ ] PANE-04: Each pane maintains its own sort and grouping state independent from other panes.

### Pane Lifecycle and Visibility

- [ ] PANE-05: User can create and delete panes using dedicated hotkeys.
- [ ] VIEW-01: When panes are hidden, the UI reverts to the default single-pane task view.
- [ ] VIEW-02: User can toggle all panes visible/hidden with one hotkey.

### Config-Defined Panes

- [ ] CFG-01: User can predefine panes in config.toml.
- [ ] CFG-02: Each config-defined pane can set default sort, group, and filter behavior.
- [ ] CFG-03: Invalid pane definitions fail safely with warnings and fallback behavior.

## v2 Requirements

### Pane Workflow Expansion

- PANE-06: User can reorder panes interactively.
- PANE-07: User can pin pane layouts as named workspaces.
- PANE-08: User can move tasks between panes with bulk actions.

## Out of Scope

| Feature | Reason |
| ------- | ------ |
| Native GUI Kanban board | This milestone is TUI-only. |
| Drag-and-drop pane interactions | Keyboard-first interactions remain the milestone focus. |
| Network-synced collaborative pane state | Local-first behavior only for this milestone. |

## Traceability

| Requirement | Phase | Status |
| ----------- | ----- | ------ |
| PANE-01 | Phase 24 | Pending |
| PANE-02 | Phase 24 | Pending |
| PANE-03 | Phase 25 | Pending |
| PANE-04 | Phase 25 | Pending |
| PANE-05 | Phase 26 | Pending |
| VIEW-01 | Phase 24/26 | Pending |
| VIEW-02 | Phase 26 | Pending |
| CFG-01 | Phase 27 | Pending |
| CFG-02 | Phase 27 | Pending |
| CFG-03 | Phase 27 | Pending |

Coverage:

- v1 requirements: 10 total
- Mapped to phases: 10
- Unmapped: 0

---
Requirements defined: 2026-04-28
Last updated: 2026-04-28 after milestone v1.4 initialization

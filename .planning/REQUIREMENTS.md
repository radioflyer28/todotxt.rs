# Requirements: v1.4 Kanban-Style Vertical Panes

Defined: 2026-04-28
Core Value: A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## v1 Requirements

### Pane Layout and Focus

- [x] PANE-01: User can view multiple vertical panes in the TUI, each pane showing a task list.
- [x] PANE-02: User can switch active focus between panes using keyboard-only controls.

### Pane-Scoped Query Behavior

- [x] PANE-03: Each pane maintains its own filter query independent from other panes.
- [x] PANE-04: Each pane maintains its own sort and grouping state independent from other panes.

### Pane Lifecycle and Visibility

- [x] PANE-05: User can create and delete panes using dedicated hotkeys.
- [x] VIEW-01: When panes are hidden, the UI reverts to the default single-pane task view.
- [x] VIEW-02: User can toggle all panes visible/hidden with one hotkey.

### Config-Defined Panes

- [x] CFG-01: User can predefine panes in config.toml.
- [x] CFG-02: Each config-defined pane can set default sort, group, and filter behavior.
- [x] CFG-03: Invalid pane definitions fail safely with warnings and fallback behavior.

### CLI Path Overrides and File Resolution

- [x] PATH-01: User can pass a CLI flag to open an alternate todo.txt file instead of the path defined in config.toml.
- [x] PATH-02: When an alternate todo.txt path is used and no explicit archive path is provided, archive.txt defaults to the same directory as that todo.txt path.
- [x] PATH-03: User can pass dedicated CLI flags for alternate archive.txt and alternate config.toml paths.

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
| PANE-01 | Phase 24 (implementation) + Phase 29 (verification) | Complete |
| PANE-02 | Phase 24 (implementation) + Phase 29 (verification) | Complete |
| PANE-03 | Phase 25 (infrastructure) + Phase 28 (FAIL-1 fix) + Phase 29 (verification) | Complete |
| PANE-04 | Phase 25 (implementation) + Phase 28 (consistency fixes) + Phase 29 (verification) | Complete |
| PANE-05 | Phase 26 | Complete |
| VIEW-01 | Phase 24 (fallback) + Phase 26 (panes_hidden toggle) | Complete |
| VIEW-02 | Phase 26 (Ctrl+P toggle) + Phase 28 (status bar guard) + Phase 29 (verification) | Complete |
| CFG-01 | Phase 27 | Complete |
| CFG-02 | Phase 27 | Complete |
| CFG-03 | Phase 27 | Complete |
| PATH-01 | Phase 27 | Complete |
| PATH-02 | Phase 27 | Complete |
| PATH-03 | Phase 27 | Complete |

Coverage:

- v1 requirements: 13 total
- Mapped to phases: 13
- Unmapped: 0
- Satisfied (verified): 13
- Partial (gap closure planned): 0

---
Requirements defined: 2026-04-28
Last updated: 2026-04-29 after Phase 29 gap closure — all 13 v1.4 requirements verified complete

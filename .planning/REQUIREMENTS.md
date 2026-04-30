# Requirements: v1.5 Task Properties + Workspace Quick Picker

Defined: 2026-04-29
Core Value: A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## v1 Requirements

### Task Property Helpers

- [ ] PROP-01: Pressing `s` opens a due-date picker that can set or overwrite `due:` on the active task.
- [ ] PROP-02: Pressing `i` opens a priority picker that can set or overwrite priority `(A-Z)` on the active task.
- [ ] PROP-03: Due-date and priority pickers apply to all selected tasks when multi-selection is active.
- [ ] PROP-04: Bulk property edits preserve non-target metadata (`@context`, `+project`, creation/completion fields).

### Workspace File Picker

- [ ] WS-01: `config.toml` can define an array of workspace entries with `label`, `todo_path`, and `done_path`.
- [ ] WS-02: A quick file picker lets users switch the active workspace by label.
- [ ] WS-03: Switching workspace reloads task data from the selected todo/done paths without app restart.
- [ ] WS-04: Workspace picker clearly shows current workspace and target workspace before switching.

### Clipboard Workflows Across Workspaces

- [ ] CLIP-01: Copy action copies selected task line text in todo.txt-compatible raw form.
- [ ] CLIP-02: Cut action copies selected task line text, then removes those tasks from source list after confirmation rules are applied.
- [ ] CLIP-03: Paste action creates new task entries from clipboard lines in the current workspace.
- [ ] CLIP-04: Clipboard behavior works across workspace switches (for example, copy in Work, paste in Home).

### Metadata Flexibility and Views

- [ ] META-01: Context and project metadata remain plain todo.txt tokens (`@context`, `+project`) with no new custom schema.
- [ ] META-02: Hierarchical tag conventions like `@email/waiting` and `+client/acme` are accepted as ordinary tokens and remain queryable.
- [ ] VIEW-03: Existing filter/sort/group views continue to work consistently after workspace switching and property bulk edits.

## v2 Requirements

### Optional Expansion (Defer Unless Needed)

- META-03: Saved metadata templates/snippets for fast task entry.
- WS-05: Optional workspace-specific default filters/views.
- CLIP-05: Explicit "move to workspace" shortcut (beyond cut + switch + paste workflow).

## Out of Scope

| Feature | Reason |
| ------- | ------ |
| Task dependency graph (task A blocks task B) | High complexity and not aligned with low-friction todo.txt workflows. |
| Custom metadata schema beyond todo.txt conventions | Would reduce interoperability and increase parsing complexity. |
| Rich clipboard/object payloads with hidden metadata | Raw text copy/cut/paste keeps behavior transparent and predictable. |
| Automatic cross-workspace conflict resolution | Not required for current single-user/local-file workflow. |

## Traceability

| Requirement | Phase | Status |
| ----------- | ----- | ------ |
| PROP-01 | Phase 33 | Pending |
| PROP-02 | Phase 33 | Pending |
| PROP-03 | Phase 34 | Pending |
| PROP-04 | Phase 34 | Pending |
| WS-01 | Phase 35 | Pending |
| WS-02 | Phase 35 | Pending |
| WS-03 | Phase 35 | Pending |
| WS-04 | Phase 35 | Pending |
| CLIP-01 | Phase 36 | Pending |
| CLIP-02 | Phase 36 | Pending |
| CLIP-03 | Phase 36 | Pending |
| CLIP-04 | Phase 36 | Pending |
| META-01 | Phase 37 | Pending |
| META-02 | Phase 37 | Pending |
| VIEW-03 | Phase 37 | Pending |

Coverage:

- v1 requirements: 15 total
- Mapped to phases: 15
- Unmapped: 0

---
Requirements defined: 2026-04-29
Last updated: 2026-04-29 after v1.5 milestone initialization

# Requirements: v1.5 Capture Flow + Bulk Safety + Clipboard + Undo

Defined: 2026-04-29
Core Value: A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## v1 Requirements

### Capture Friction and Fast Entry

- [ ] CAP-01: Add-task flow remains instant with minimal mode switching and predictable keybindings.
- [ ] CAP-02: Edit-task flow remains fast and consistent with add-task key behavior.
- [ ] CAP-03: Pressing `s` opens a due-date picker that can set or overwrite `due:` on active or selected tasks.
- [ ] CAP-04: Pressing `i` opens a priority picker that can set or overwrite priority `(A-Z)` on active or selected tasks.
- [ ] CAP-05: Property edits preserve non-target metadata (`@context`, `+project`, creation/completion fields).

### Quick Context and Project Setters

- [ ] TAG-01: Pressing `@` in normal mode opens a quick context setter for active or selected tasks.
- [ ] TAG-02: Pressing `+` in normal mode opens a quick project setter for active or selected tasks.
- [ ] TAG-03: Quick tag setters avoid duplicate tokens and preserve all non-target metadata.
- [ ] TAG-04: Context/project setters show autocomplete matches while typing, including potentially redundant near-matches.
- [ ] TAG-05: Autocomplete supports arrow-key selection and tab-to-complete for fast token entry.

### Date Autocomplete and Picker Guidance

- [ ] DATE-01: Typing partial due tokens such as `due:2026-` or `t:2026-07-` shows valid numeric day suggestions for the target month.
- [ ] DATE-02: Date suggestions display weekday labels next to each suggested day.
- [ ] DATE-03: Date autocomplete supports arrow-key selection and tab-to-complete in date entry flows.
- [ ] DATE-04: The `s` due-date setter uses the same month-aware day options and weekday labels as typed date autocomplete.

### Safe Bulk Actions

- [ ] BULK-01: High-impact bulk actions (overwrite, cut, delete) display affected-count preview before execution.
- [ ] BULK-02: Bulk actions provide a clear cancel path and leave data unchanged on cancel.
- [ ] BULK-03: Bulk action targeting remains stable for multi-selection and grouped/pane views.

### Basic Clipboard Workflows

- [ ] CLIP-01: Copy action copies selected task line text in todo.txt-compatible raw form.
- [ ] CLIP-02: Cut action copies selected task line text, then removes selected tasks after confirmation rules are applied.
- [ ] CLIP-03: Paste action creates new task entries from one or more clipboard lines.
- [ ] CLIP-04: Pasting is supported during new-task entry (`n`) to quickly duplicate and tweak tasks.

### Recovery Path

- [ ] UNDO-01: Short-horizon undo is available for recent destructive/high-impact actions.
- [ ] UNDO-02: Undo restores both task content and selection state for the reverted action where feasible.
- [ ] UNDO-03: Undo feedback is clear (what was reverted) and safe (no-op message when history is empty).

### Metadata Flexibility and Views

- [ ] META-01: Context and project metadata remain plain todo.txt tokens (`@context`, `+project`) with no new custom schema.
- [ ] META-02: Hierarchical tag conventions like `@email/waiting` and `+client/acme` are accepted as ordinary tokens and remain queryable.
- [ ] VIEW-03: Existing filter/sort/group views continue to work consistently after capture, bulk, clipboard, and undo flows.

## v2 Requirements

### Optional Expansion (Defer Unless Needed)

- META-03: Saved metadata templates/snippets for fast task entry.
- WS-01: `config.toml` workspace entries (`label`, `todo_path`, `done_path`) for fast switching.
- WS-02: Workspace quick picker UI and runtime source switching.
- WS-03: Optional workspace-specific default filters/views.
- CLIP-05: Explicit "move to workspace" shortcut.

## Out of Scope

| Feature | Reason |
| ------- | ------ |
| Task dependency graph (task A blocks task B) | High complexity and not aligned with low-friction todo.txt workflows. |
| Custom metadata schema beyond todo.txt conventions | Would reduce interoperability and increase parsing complexity. |
| Rich clipboard/object payloads with hidden metadata | Raw text copy/cut/paste keeps behavior transparent and predictable. |
| Workspace switching in v1.5 | Deferred to a later milestone to keep current scope focused and shippable. |

## Traceability

| Requirement | Phase | Status |
| ----------- | ----- | ------ |
| CAP-01 | Phase 38 | Pending |
| CAP-02 | Phase 38 | Pending |
| CAP-03 | Phase 38 | Pending |
| CAP-04 | Phase 38 | Pending |
| CAP-05 | Phase 38 | Pending |
| TAG-01 | Phase 38 | Pending |
| TAG-02 | Phase 38 | Pending |
| TAG-03 | Phase 38 | Pending |
| TAG-04 | Phase 38 | Pending |
| TAG-05 | Phase 38 | Pending |
| DATE-01 | Phase 38 | Pending |
| DATE-02 | Phase 38 | Pending |
| DATE-03 | Phase 38 | Pending |
| DATE-04 | Phase 38 | Pending |
| BULK-01 | Phase 38 | Pending |
| BULK-02 | Phase 38 | Pending |
| BULK-03 | Phase 38 | Pending |
| CLIP-01 | Phase 35 | Pending |
| CLIP-02 | Phase 35 | Pending |
| CLIP-03 | Phase 35 | Pending |
| CLIP-04 | Phase 35 | Pending |
| UNDO-01 | Phase 36 | Pending |
| UNDO-02 | Phase 36 | Pending |
| UNDO-03 | Phase 36 | Pending |
| META-01 | Phase 38 | Pending |
| META-02 | Phase 38 | Pending |
| VIEW-03 | Phase 38 | Pending |

Coverage:

- v1 requirements: 27 total
- Mapped to phases: 27
- Unmapped: 0

---
Requirements defined: 2026-04-29
Last updated: 2026-04-30 after milestone gap-closure phase planning

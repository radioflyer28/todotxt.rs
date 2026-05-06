# Requirements: v1.6 TUI Fixes and Power User Improvements

Defined: 2026-05-04
Core Value: A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## Scope Decision Summary

- **Pane task movement:** Move selected task(s) between panes via tag mutation — IN SCOPE
- **Validation debt:** SEED-005 Phase 22 tests — IN SCOPE
- **Quick wins:** SEED-006 archive, SEED-009 bulk done, SEED-012 $EDITOR, SEED-013 autocomplete fix — IN SCOPE
- **Autocomplete & Filter UX:** SEED-014 coverage + narrowing, SEED-011 filter history — IN SCOPE
- **View & Presets:** SEED-015 full presets, SEED-007 view persistence, SEED-008 group-by decoupling — IN SCOPE
- **Recurring tasks (SEED-010):** DEFERRED — v1.7 or later

## Key Decisions Made During Requirements

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Archive scope | All completed tasks (not just visible) | Matches C# todotxt.net behavior |
| Archive confirmation | Yes — show count before archiving | Non-reversible action; user protection |
| View state file location | Config directory (XDG/platform config dir) | Separates config from data |
| Filter history recall key | `Ctrl+R` | Shell reverse-search mental model |

---

## v1 Requirements

### Archive Workflow (SEED-006)

- [ ] **ARCH-01**: User can archive all completed tasks to done.txt with a single keypress from the TUI (default key: `A`)
- [ ] **ARCH-02**: User sees a confirmation dialog showing the count of tasks to be archived before the action completes; pressing Enter confirms, Esc cancels
- [ ] **ARCH-03**: User sees a status bar message confirming how many tasks were archived, or an error if the archive path is not writable

### Bulk Mark-Done (SEED-009)

- [ ] **BDONE-01**: User can mark all tasks in an active multi-selection as done with a single `x` keypress (when `selected_tasks` is non-empty, `x` targets the selection rather than the cursor task)
- [ ] **BDONE-02**: When bulk-marking done on a mixed selection (some tasks already done), all incomplete tasks in the selection are completed (not toggled); already-done tasks are left unchanged

### External Editor Integration (SEED-012)

- [ ] **XEDIT-01**: User can open the cursor task in an external editor via `Ctrl+E` from Normal mode; editor is resolved as `$VISUAL` → `$EDITOR` → platform fallback (`notepad` on Windows, `vi` on Unix)
- [ ] **XEDIT-02**: The TUI suspends ratatui rendering and disables crossterm raw mode before launching the editor, and fully restores terminal state after the editor exits — including on editor crash
- [ ] **XEDIT-03**: If no external editor is found or the editor exits with an error, the TUI shows an error message in the status bar and continues operating normally without data loss

### Autocomplete Fixes and Coverage (SEED-013 + SEED-014)

- [ ] **AC-01**: When typing `+` in the task editor (Adding or Editing mode), a suggestion popup appears with existing project tags from the task list (fixes the SEED-013 regression where `+` triggered no suggestions)
- [x] **AC-02**: When typing `@` or `+` in the filter input, a suggestion popup appears with known contexts or projects from the task list
- [x] **AC-03**: Selecting a suggestion from the autocomplete popup while in filter input mode inserts it into the filter input field (not the task editor)
- [x] **AC-04**: While the autocomplete popup is visible, additional characters typed narrow the suggestion list incrementally (each keypress re-filters the candidate list by the updated prefix)

### Filter Input History (SEED-011)

- [ ] **FHIST-01**: The filter input maintains a ring buffer of recently used filter expressions for the current session; expressions are added when the user applies a filter (presses Enter)
- [ ] **FHIST-02**: User can cycle through recent filter history using `Ctrl+R` in the filter input; each press recalls the previous entry
- [ ] **FHIST-03**: The filter history ring is deduplicated — applying the same filter expression twice records only one entry

### View State Persistence (SEED-007)

- [ ] **PRSV-01**: The TUI saves the current view state (sort order, grouping, active filter, and per-pane settings) to a `tui-state.toml` sidecar in the platform config directory on clean exit
- [ ] **PRSV-02**: The TUI loads `tui-state.toml` at startup and applies saved state; if the file is absent, unreadable, or contains unknown fields, config.toml defaults are used silently (no error shown)
- [ ] **PRSV-03**: Only pane dimensions that were interactively modified during the session are written to the state file; panes that were not changed retain config.toml as their source of truth

### Group-by Decoupling (SEED-008)

- [ ] **GRP-01**: Each TUI pane has an independent group-by category (Project, Context, Priority, DueDate) that is separate from the intra-group sort order; the two can be configured independently
- [ ] **GRP-02**: User can toggle grouping on/off with `G` (Shift+g) by default (configurable); `g` cycles through group-by categories; `o` continues to cycle intra-group sort order
- [ ] **GRP-03**: The status bar displays both the active group-by category and the active sort order simultaneously when grouping is enabled
- [ ] **GRP-04**: Group-by category can be defined per pane in `config.toml` (e.g., `group_by = "project"`) and is respected at startup independently from sort order

### Full View Presets (SEED-015)

- [ ] **PRST-01**: Numeric preset config blocks (`[presets.f1]` … `[presets.f9]`) can optionally declare `sort`, `group`, `group_by`, `panes` (count and layout), and per-pane settings in addition to `filter`; all new fields are optional and existing filter-only presets continue to work without modification
- [ ] **PRST-02**: Pressing a preset key (1–9) applies all defined preset dimensions atomically — including filter, sort, grouping, group-by category, active pane count, pane positions, and per-pane view settings — in a single operation

### Test Coverage Debt (SEED-005)

- [ ] **TST-01**: All 11 Phase 22 manual-only validation gaps (KEY-01, KEY-02, PAR-01, PAR-02 test cases covering mode transitions, key dispatch, filter mutations, preset application, and help overlay behavior) are covered by automated unit tests in the `todotxt-tui` crate
- [ ] **TST-02**: A `make_app_with_keymap` (or equivalent) test helper exists that constructs an `App` instance with a custom keymap config, enabling isolated testing of keymap-dependent behavior

### Move Tasks Between Panes

- [ ] **PMOVE-01**: User can move the cursor task (or all tasks in the current multi-selection) to an adjacent pane with `Ctrl+Left` / `Ctrl+Right` (default, configurable via keymap)
- [ ] **PMOVE-02**: When moving a task between two filter-based panes where each pane's filter is a single tag token (e.g., `@waiting`, `+project`), the move operation removes the source pane's filter token from the task and appends the destination pane's filter token — making the task disappear from the source pane and appear in the destination pane
- [ ] **PMOVE-03**: If the source or destination pane has no filter, or its filter is not a single tag token (e.g., a compound query like `due:today @work`), the move is declined and the status bar explains why; no task data is modified

---

## Future Requirements (Deferred)

| Requirement | Rationale |
|-------------|-----------|
| Recurring task support (`rec:` extension) — SEED-010 | Requires core data model changes and deep CLI+TUI coordination; deferred to v1.7 |
| Cross-session filter history persistence | Session history (FHIST-01–03) covers the core use case; persistence is low priority |
| View preset `pane_focus` field | Depends on further pane layout work; extend PRST in a later milestone |

---

## Out of Scope (v1.6)

| Feature | Reason |
|---------|--------|
| Native GUI interface (SEED-002) | Different delivery track; high effort |
| CI/CD pipeline + release binaries (SEED-004) | Not TUI/CLI feature work |
| todo.sh compatibility additions (SEED-003) | Already addressed in v1.2; no new gaps identified |

---

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| ARCH-01, ARCH-02, ARCH-03 | Phase 39 | Pending |
| BDONE-01, BDONE-02 | Phase 39 | Pending |
| XEDIT-01, XEDIT-02, XEDIT-03 | Phase 39 | Pending |
| AC-01 | Phase 39 | Pending |
| AC-02, AC-03, AC-04 | Phase 42 | Pending |
| FHIST-01, FHIST-02, FHIST-03 | Phase 41 | Pending |
| PRSV-01, PRSV-02, PRSV-03 | Phase 43 | Pending |
| GRP-01, GRP-02, GRP-03, GRP-04 | Phase 40 | Pending |
| PRST-01, PRST-02 | Phase 41 | Pending |
| PMOVE-01, PMOVE-02, PMOVE-03 | Phase 44 | Pending (gap closure — BUG-41-01 fix) |
| TST-01, TST-02 | Phase 40 | Pending |

Coverage:

- v1 requirements: 29 total
- Mapped to phases: 29
- Unmapped: 0

---
Requirements defined: 2026-05-04
Last updated: 2026-05-06 — PMOVE-01/02/03 reassigned to Phase 44 (gap closure)

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
| CAP-01 | Phase 33 | Pending |
| CAP-02 | Phase 33 | Pending |
| CAP-03 | Phase 33 | Pending |
| CAP-04 | Phase 33 | Pending |
| CAP-05 | Phase 34 | Pending |
| TAG-01 | Phase 33 | Pending |
| TAG-02 | Phase 33 | Pending |
| TAG-03 | Phase 34 | Pending |
| TAG-04 | Phase 33 | Pending |
| TAG-05 | Phase 33 | Pending |
| DATE-01 | Phase 33 | Pending |
| DATE-02 | Phase 33 | Pending |
| DATE-03 | Phase 33 | Pending |
| DATE-04 | Phase 33 | Pending |
| BULK-01 | Phase 34 | Pending |
| BULK-02 | Phase 34 | Pending |
| BULK-03 | Phase 34 | Pending |
| CLIP-01 | Phase 35 | Pending |
| CLIP-02 | Phase 35 | Pending |
| CLIP-03 | Phase 35 | Pending |
| CLIP-04 | Phase 35 | Pending |
| UNDO-01 | Phase 36 | Pending |
| UNDO-02 | Phase 36 | Pending |
| UNDO-03 | Phase 36 | Pending |
| META-01 | Phase 37 | Pending |
| META-02 | Phase 37 | Pending |
| VIEW-03 | Phase 37 | Pending |

Coverage:

- v1 requirements: 27 total
- Mapped to phases: 27
- Unmapped: 0

---
Requirements defined: 2026-04-29
Last updated: 2026-04-29 after v1.5 milestone initialization

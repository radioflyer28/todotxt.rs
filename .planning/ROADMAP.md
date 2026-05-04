# Roadmap: todotxt.net — Rust Port

## Milestones

- ✅ v1.0 Core Library + CLI — shipped 2026-04-16 (archive: .planning/milestones/v1.0-ROADMAP.md)
- ✅ v1.1 TUI Interface — shipped 2026-04-23 (archive: .planning/milestones/v1.1-ROADMAP.md)
- ✅ v1.2 Compatibility + UX Alignment — shipped 2026-04-24 (archive: .planning/milestones/v1.2-ROADMAP.md)
- ✅ v1.3 Feature/Hotkey Parity with todotxt.net — shipped 2026-04-28 (archive: .planning/milestones/v1.3-ROADMAP.md)
- ✅ v1.4 Kanban-Style Vertical Panes — shipped 2026-04-29 (archive: .planning/milestones/v1.4-ROADMAP.md)
- ✅ v1.5 Capture Flow + Bulk Safety + Clipboard + Undo — shipped 2026-05-01 (archive: .planning/milestones/v1.5-ROADMAP.md)

## Active Milestone

**v1.6 — TUI Fixes and Power User Improvements** (in progress)

### Phases

- [ ] **Phase 39: Quick Wins** — Archive workflow, bulk mark-done, external editor escape hatch, and `+` autocomplete bug fix
- [ ] **Phase 40: Group-By Decoupling + Test Coverage** — Independent group-by controls per pane and automated Phase 22 test coverage
- [ ] **Phase 41: Full Presets, Filter History, Pane Task Movement** — Multi-dimensional view presets, session filter history, and tag-mutation pane moves
- [ ] **Phase 42: Filter Autocomplete Coverage** — Autocomplete in the filter input with incremental narrowing
- [ ] **Phase 43: View State Persistence** — Save and restore TUI view state across restarts

---

## Phase Details

### Phase 39: Quick Wins
**Goal:** Users gain four previously-missing TUI capabilities (archive, bulk mark-done, external editor, autocomplete `+` fix) in a single pass of independent feature work
**Requirements:** ARCH-01, ARCH-02, ARCH-03, BDONE-01, BDONE-02, XEDIT-01, XEDIT-02, XEDIT-03, AC-01
**Depends on:** Nothing
**Success criteria:**
1. User presses `A` from the TUI, sees a confirmation dialog showing the count of completed tasks, and after confirming receives a status bar message reporting how many tasks were archived to done.txt
2. With one or more tasks selected, pressing `x` marks all incomplete tasks in the selection as done and leaves already-done tasks unchanged
3. User presses `Ctrl+E` on a cursor task and the task opens in `$VISUAL` / `$EDITOR` / platform fallback; TUI suspends ratatui rendering and raw mode before launch and fully restores terminal state after exit — including on editor crash
4. If no editor is found or the editor exits with an error, a status bar error message appears and the TUI continues operating without data loss
5. Typing `+` in the task editor (Add or Edit mode) shows an autocomplete popup with existing project tags, and the popup narrows as additional characters are typed
**Plans:** TBD

### Phase 40: Group-By Decoupling + Test Coverage
**Goal:** Group-by category and sort order become fully independent controls per pane, and Phase 22 manual validation gaps are closed with automated tests
**Requirements:** GRP-01, GRP-02, GRP-03, GRP-04, TST-01, TST-02
**Depends on:** Phase 39
**Success criteria:**
1. Each TUI pane can have a distinct group-by category (Project, Context, Priority, DueDate) that is configured independently from its intra-group sort order
2. `G` (Shift+g) toggles grouping on/off; `g` cycles through group-by categories; `o` cycles intra-group sort order — each key affects only its own dimension without resetting the others
3. When grouping is enabled the status bar displays both the active group-by category and the active sort order simultaneously
4. A `group_by` field in a pane's `config.toml` block is respected at startup independently from sort order
5. All 11 previously manual-only Phase 22 test cases pass as automated unit tests; a `make_app_with_keymap` (or equivalent) test helper exists in the `todotxt-tui` crate for constructing an `App` with a custom keymap
**Plans:** TBD

### Phase 41: Full Presets, Filter History, Pane Task Movement
**Goal:** Power users can activate multi-dimensional view presets with one key, recall recent filter expressions, and move tasks between filter-based panes via tag mutation
**Requirements:** PRST-01, PRST-02, FHIST-01, FHIST-02, FHIST-03, PMOVE-01, PMOVE-02, PMOVE-03
**Depends on:** Phase 40
**Success criteria:**
1. Pressing a preset key (1–9) atomically applies all declared preset dimensions (filter, sort, group, group_by, pane count/layout, per-pane settings) in one operation; existing filter-only presets continue to work without modification
2. The filter input maintains a deduplicated session ring buffer; pressing `Ctrl+R` in the filter input recalls the previous applied filter expression, cycling backward through history on each press
3. User presses `Ctrl+Left` or `Ctrl+Right` to move the cursor task (or all tasks in an active multi-selection) to an adjacent pane
4. When both source and destination panes have single-token tag filters, the move operation removes the source tag and appends the destination tag — the task disappears from the source pane and appears in the destination pane
5. If the source or destination pane has no filter or a compound filter, the move is declined and the status bar explains why; no task data is modified
**Plans:** TBD
**UI hint**: yes

### Phase 42: Filter Autocomplete Coverage
**Goal:** Autocomplete works consistently in the filter input as well as the task editor, with incremental narrowing on each keypress
**Requirements:** AC-02, AC-03, AC-04
**Depends on:** Phase 39 (AC-01 `+` fix), Phase 41 (filter input UX established)
**Success criteria:**
1. Typing `@` or `+` in the filter input shows an autocomplete suggestion popup with known contexts or projects from the task list
2. Selecting a suggestion from the popup while the filter input is active inserts it into the filter field — not the task editor
3. Each character typed after the trigger (`@` or `+`) re-filters the candidate list, narrowing the suggestions incrementally
**Plans:** TBD

### Phase 43: View State Persistence
**Goal:** TUI view state (sort order, grouping, active filter, pane settings) is saved on exit and restored at startup
**Requirements:** PRSV-01, PRSV-02, PRSV-03
**Depends on:** Phase 40, Phase 41, Phase 42
**Success criteria:**
1. On clean exit, the TUI writes a `tui-state.toml` file to the platform config directory capturing the current sort order, group-by category, grouping toggle, active filter expression, and per-pane settings for panes that were interactively modified during the session
2. On startup, the TUI loads `tui-state.toml` and restores the saved view state; if the file is absent, unreadable, or contains unknown fields, config.toml defaults are applied silently with no error shown to the user
3. Pane settings that were not changed during the session are not written to the state file and continue to be governed by config.toml
**Plans:** TBD

---

## Progress

| Phase | Name | Plans Complete | Status | Completed |
|-------|------|----------------|--------|-----------|
| 39 | Quick Wins | 0/? | Not started | - |
| 40 | Group-By Decoupling + Test Coverage | 0/? | Not started | - |
| 41 | Full Presets, Filter History, Pane Task Movement | 0/? | Not started | - |
| 42 | Filter Autocomplete Coverage | 0/? | Not started | - |
| 43 | View State Persistence | 0/? | Not started | - |

---

## Archived Milestone Summary

- v1.5 archived: phases 33-38, 14/14 plans complete, requirements archived at .planning/milestones/v1.5-REQUIREMENTS.md
- milestone audit: .planning/v1.5-MILESTONE-AUDIT.md (status: tech_debt, no unsatisfied blockers)

## Backlog

- GUI interface (native desktop)
- CI/CD release pipeline and package distribution

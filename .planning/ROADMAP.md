# Roadmap: todotxt.net — Rust Port

## Milestones

- ✅ **v1.0 Core Library + CLI** — Phases 1–8 (shipped 2026-04-16)
- 🔄 **v1.1 TUI Interface** — Phases 9–13 (in progress)

## Phases

<details>
<summary>✅ v1.0 Core Library + CLI (Phases 1–8) — SHIPPED 2026-04-16</summary>

- [x] Phase 1: Workspace Bootstrap + Core Library Foundation (2/2 plans) — completed 2026-04-15
- [x] Phase 2: Core Library Completion (3/3 plans) — completed 2026-04-15
- [x] Phase 3: CLI Foundation — Config + Output + Read Commands (5/5 plans) — completed 2026-04-15
- [x] Phase 4: CLI Write Commands (5/5 plans) — completed 2026-04-15
- [x] Phase 5: Task Enrichment + Bulk Operations (6/6 plans) — completed 2026-04-15
- [x] Phase 6: Cross-Platform Polish + Integration Tests (4/4 plans) — completed 2026-04-15
- [x] Phase 7: Retroactive Core Library Verification (2/2 plans) — completed 2026-04-16
- [x] Phase 8: Retroactive CLI Verification (3/3 plans) — completed 2026-04-16

See full phase details: .planning/milestones/v1.0-ROADMAP.md

</details>

## v1.1 Phases

- [ ] **Phase 9: TUI Foundation** — New crate scaffold, terminal lifecycle, event loop, config + file-watcher integration
- [ ] **Phase 10: Core TUI** — Task list display, keyboard navigation, mark done/undo, status bar, quit
- [ ] **Phase 11: Edit Mode** — Add task, inline edit, delete with confirmation, autocomplete, deferred reload guard
- [ ] **Phase 12: Filter + Sort** — Filter panel, multi-filter AND logic, sort toggle, filter/sort in status bar
- [ ] **Phase 13: Theming + Polish** — Built-in themes, config-selectable theme, NO_COLOR, auto-reload UX, terminal restore

## Phase Details

### Phase 9: TUI Foundation

**Goal:** The `todotxt-tui` crate exists in the workspace, compiles cleanly, and establishes safe terminal lifecycle with a running event loop connected to the core library.
**Milestone:** v1.1 TUI Interface
**Requirements:** TUI-INFRA-01, TUI-INFRA-02
**Depends on:** Phase 8 complete

**Success Criteria:**
1. `cargo build -p todotxt-tui` succeeds with no warnings on all three platforms
2. `todotxt-tui` reads the same TOML config file as `todotxt-cli` and resolves the todo.txt path
3. Running `todotxt-tui` enters full-screen terminal mode; pressing `q` exits and fully restores the terminal (no leftover raw mode or alternate screen)
4. The event loop is connected to `todotxt-core`'s `FileWatcher` and receives file-change events
5. A panic in the TUI (simulated) does not leave the terminal in a broken state

**Plans:** TBD

---

### Phase 10: Core TUI

**Goal:** Users can view the full task list in the terminal and navigate, mark tasks done, undo, and quit using keyboard shortcuts.
**Milestone:** v1.1 TUI Interface
**Requirements:** TUI-NAV-01, TUI-NAV-02, TUI-NAV-03, TUI-NAV-04, TUI-ACT-01, TUI-ACT-02, TUI-UX-01, TUI-UX-05, TUI-INFRA-03
**Depends on:** Phase 9

**Success Criteria:**
1. User can scroll the task list with `j`/`k`, arrow keys, `g`/`G` (jump first/last), and `Ctrl+d`/`Ctrl+u` (half-page)
2. Task IDs shown in the TUI match the source file line numbers, not the filtered display index
3. User can mark a task done with `x` (toggles done/undone in-place) and undo a completion with `u`
4. Status bar shows total, visible, due-today, and overdue task counts at all times
5. User can quit with `q` or `Ctrl+c`; binary compiles and runs correctly on Windows, Linux, and macOS

**Plans:** TBD
**UI hint**: yes

---

### Phase 11: Edit Mode

**Goal:** Users can add new tasks, inline-edit existing tasks, and delete tasks with confirmation — all from within the TUI — and the display stays consistent during edits.
**Milestone:** v1.1 TUI Interface
**Requirements:** TUI-ACT-03, TUI-ACT-04, TUI-ACT-05, TUI-ACT-06, TUI-UX-03
**Depends on:** Phase 10

**Success Criteria:**
1. Pressing `a` opens an input field; typing and pressing `Enter` adds the task to the file; `Esc` cancels with no change
2. Pressing `e` opens the selected task for inline edit pre-populated with its raw text; `Enter` saves, `Esc` cancels
3. `@context` and `+project` autocomplete popup appears when typing `@` or `+` in add/edit mode
4. Pressing `d` shows a confirmation prompt with a task preview; `y` deletes, any other key cancels
5. A file-change event received while the user is in add/edit mode is queued and applied automatically upon returning to normal mode

**Plans:** TBD
**UI hint**: yes

---

### Phase 12: Filter + Sort

**Goal:** Users can narrow the task list by context, project, or due date and cycle through sort orders, with all active filters and the current sort visible in the status bar.
**Milestone:** v1.1 TUI Interface
**Requirements:** TUI-FILTER-01, TUI-FILTER-02, TUI-FILTER-03, TUI-FILTER-04
**Depends on:** Phase 11

**Success Criteria:**
1. Pressing the filter key opens a filter panel; user can toggle context, project, and due-date filters using `Space` and reset all with `Ctrl+R`
2. When multiple filters are active, only tasks matching ALL active filters are shown (AND semantics)
3. User can cycle sort order (priority → due date → alphabetical, and back) with a single keybind
4. The status bar shows all active filters and the current sort order at all times

**Plans:** TBD
**UI hint**: yes

---

### Phase 13: Theming + Polish

**Goal:** The TUI is visually polished with two built-in themes, user-configurable color selection, and correct terminal restoration and reload UX under all exit paths.
**Milestone:** v1.1 TUI Interface
**Requirements:** TUI-THEME-01, TUI-THEME-02, TUI-THEME-03, TUI-UX-02, TUI-UX-04
**Depends on:** Phase 12

**Success Criteria:**
1. Two built-in themes (`default` dark and `light`) render task list and status bar with distinct, readable color palettes
2. Setting `[tui] theme = "light"` in the TOML config switches to the light theme on next launch
3. Setting `NO_COLOR=1` in the environment disables all color output; the TUI remains fully functional in monochrome
4. When the todo.txt file changes externally in normal mode, the TUI silently reloads within 500ms and re-anchors the cursor to the same task
5. Terminal state (raw mode, alternate screen) is fully restored on exit — including on panic — leaving no terminal artifacts

**Plans:** TBD
**UI hint**: yes

---

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Workspace Bootstrap | v1.0 | 2/2 | Complete | 2026-04-15 |
| 2. Core Library Completion | v1.0 | 3/3 | Complete | 2026-04-15 |
| 3. CLI Foundation | v1.0 | 5/5 | Complete | 2026-04-15 |
| 4. CLI Write Commands | v1.0 | 5/5 | Complete | 2026-04-15 |
| 5. Task Enrichment + Bulk | v1.0 | 6/6 | Complete | 2026-04-15 |
| 6. Cross-Platform Polish | v1.0 | 4/4 | Complete | 2026-04-15 |
| 7. Retroactive Core Verification | v1.0 | 2/2 | Complete | 2026-04-16 |
| 8. Retroactive CLI Verification | v1.0 | 3/3 | Complete | 2026-04-16 |
| 9. TUI Foundation | v1.1 | 0/? | Not started | — |
| 10. Core TUI | v1.1 | 0/? | Not started | — |
| 11. Edit Mode | v1.1 | 0/? | Not started | — |
| 12. Filter + Sort | v1.1 | 0/? | Not started | — |
| 13. Theming + Polish | v1.1 | 0/? | Not started | — |

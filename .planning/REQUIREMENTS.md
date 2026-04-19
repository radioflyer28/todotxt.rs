# Requirements — v1.1 TUI Interface

## Milestone v1.1 Requirements

### Navigation

- [ ] **TUI-NAV-01**: User can scroll the task list with j/k and arrow keys
- [ ] **TUI-NAV-02**: User can jump to first/last task with g/G
- [ ] **TUI-NAV-03**: Task IDs shown in the TUI match the source file line number (not the filtered index)
- [ ] **TUI-NAV-04**: User can half-page scroll with Ctrl+d / Ctrl+u

### Task Actions

- [ ] **TUI-ACT-01**: User can mark a task done by pressing x (toggles done/undone in-place)
- [ ] **TUI-ACT-02**: User can undo a completed task with u
- [ ] **TUI-ACT-03**: User can add a new task by pressing a (enters input mode; committed on Enter, cancelled on Esc)
- [ ] **TUI-ACT-04**: User can inline-edit a task by pressing e (exits on Enter to save, Esc to cancel)
- [ ] **TUI-ACT-05**: User can delete a task with d (confirmation prompt shown; y confirms, any other key cancels)
- [ ] **TUI-ACT-06**: @context and +project autocomplete is available when adding or editing a task

### Filter and Sort

- [ ] **TUI-FILTER-01**: User can open a filter panel to filter tasks by context, project, or due date
- [ ] **TUI-FILTER-02**: Multiple active filters are ANDed together
- [ ] **TUI-FILTER-03**: User can cycle sort order (priority → due date → alphabetical) with a keybind
- [ ] **TUI-FILTER-04**: Active filters and current sort order are shown in the status bar

### Status Bar and UX

- [ ] **TUI-UX-01**: Status bar shows total, visible, due today, and overdue task counts
- [ ] **TUI-UX-02**: TUI auto-reloads when the todo.txt file is changed externally (500ms debounce)
- [ ] **TUI-UX-03**: Auto-reload is blocked while the user is in add/edit mode; reload is queued and applied on exit
- [ ] **TUI-UX-04**: Terminal state (raw mode, alternate screen) is fully restored on exit, including on panic
- [ ] **TUI-UX-05**: User can quit the TUI with q or Ctrl+c

### Theming

- [ ] **TUI-THEME-01**: Two built-in themes: `default` (dark terminal) and `light`
- [ ] **TUI-THEME-02**: Theme is selectable in TOML config (`[tui] theme = "default"`)
- [ ] **TUI-THEME-03**: `NO_COLOR` environment variable disables all color output

### Infrastructure

- [ ] **TUI-INFRA-01**: `todotxt-tui` is a new crate in the Cargo workspace (`crates/todotxt-tui`) producing a `todotxt-tui` binary
- [ ] **TUI-INFRA-02**: `todotxt-tui` binary reads the same TOML config file as `todotxt-cli`
- [ ] **TUI-INFRA-03**: `todotxt-tui` compiles and runs correctly on Windows, Linux, and macOS

---

## Future Requirements

- Mouse support (click to select, scroll) — deferred to v1.2
- Fuzzy search / quick search bar — deferred (nice-to-have, not blocking)
- Help overlay (`?` key) — v1.1 stretch goal
- Scrollbar widget — v1.1 polish (stretch)

---

## Out of Scope (v1.1)

- GUI interface (native desktop) — v1.2 milestone
- todo.sh compatibility layer — future milestone
- Mouse support — anti-feature for keyboard-centric power users in v1.1
- Vim `:` command line inside TUI — over-engineering
- Cross-session undo history — out of scope
- Publishing to crates.io / Homebrew / winget — CI/CD milestone (SEED-004)

---

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| TUI-INFRA-01 | — | Pending |
| TUI-INFRA-02 | — | Pending |
| TUI-INFRA-03 | — | Pending |
| TUI-NAV-01 | — | Pending |
| TUI-NAV-02 | — | Pending |
| TUI-NAV-03 | — | Pending |
| TUI-NAV-04 | — | Pending |
| TUI-ACT-01 | — | Pending |
| TUI-ACT-02 | — | Pending |
| TUI-ACT-03 | — | Pending |
| TUI-ACT-04 | — | Pending |
| TUI-ACT-05 | — | Pending |
| TUI-ACT-06 | — | Pending |
| TUI-FILTER-01 | — | Pending |
| TUI-FILTER-02 | — | Pending |
| TUI-FILTER-03 | — | Pending |
| TUI-FILTER-04 | — | Pending |
| TUI-UX-01 | — | Pending |
| TUI-UX-02 | — | Pending |
| TUI-UX-03 | — | Pending |
| TUI-UX-04 | — | Pending |
| TUI-UX-05 | — | Pending |
| TUI-THEME-01 | — | Pending |
| TUI-THEME-02 | — | Pending |
| TUI-THEME-03 | — | Pending |

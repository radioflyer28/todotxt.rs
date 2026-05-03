# Phase 10: Core TUI — Context

**Gathered:** 2026-04-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 10 delivers interactive keyboard navigation, done/undo task toggling, and a persistent status bar on top of the Phase 9 foundation.

The user can:
- Scroll the task list (j/k, arrows, g/G, Ctrl+d/Ctrl+u)
- See the selected task highlighted
- Mark a task done or undo it (x toggles both ways)
- See live task counts and the file path in a status bar
- Quit with q or Ctrl+C

Phase 11 owns add/edit/delete (input mode).
Phase 12 owns filter and sort UI.
Phase 13 owns theming and colors.

</domain>

<decisions>
## Implementation Decisions

### Task Row Display

- **D-01: Raw text with line number prefix** — Each row shows `N: <raw task text>` (e.g., `1: Buy milk`, `2: x 2024-01-01 Archive me`). No structured column parsing in Phase 10. Keeps display close to Phase 9 format; Phase 13 adds color.
- **D-02: Line number is source file line number** — Per TUI-NAV-03: the line number shown matches the file line, not the display index. `TaskList::tasks()` preserves insertion order which matches file order.
- **D-03: Completed tasks dimmed with Modifier::DIM** — Done tasks rendered with `Modifier::DIM` to visually differentiate them before Phase 13 theming. Phase 13 overrides this with theme colors.
- **D-04: Upgrade to ratatui List widget** — Phase 9 used `Paragraph`. Phase 10 switches to ratatui's `List` + `ListState` to get built-in selection support and scroll tracking. The `List` widget owns the rendered items.

### Selection State

- **D-05: Selection index on App struct** — `App.selected: usize` holds the 0-based index into `task_list.tasks()`. `ListState` is constructed in `draw()` from this value — not stored as a persistent field. Simple, easy to test.
- **D-06: ratatui ListState + reversed highlight** — `ListState::with_selected(Some(self.selected))` is passed to `List::render_stateful`. The reversed-colors highlight is the default ratatui behavior. Phase 13 overrides the highlight style with theme colors.
- **D-07: Selection stays clamped** — After reload or done/undo, `selected` is clamped to `[0, task_count - 1]`. If the list becomes empty, `selected` is set to 0 (safe sentinel).

### Navigation Keybinds

- **D-08: Navigation keybinds** (from TUI-NAV-01/02/04):
  - `j` / `↓` — move down 1
  - `k` / `↑` — move up 1
  - `g` — jump to first task (0)
  - `G` — jump to last task (task_count - 1)
  - `Ctrl+d` — scroll down half page (terminal height / 2)
  - `Ctrl+u` — scroll up half page (terminal height / 2)
- **D-09: Half-page scrolls use visible list area height** — Terminal height minus status bar row (1) gives the list height. Half-page = list_height / 2, integer division.

### Done/Undo Write Strategy

- **D-10: Immediate save on x** — `x` calls `task_list.update(idx, toggled)` then `task_list.save()` immediately. No dirty-flag buffer. Per `Task::with_completed()` API.
- **D-11: x toggles both ways** — If selected task is incomplete, `x` marks it done (with today's date). If already done, `x` marks it incomplete (removes `x` prefix and completion date). One key, full toggle.
- **D-12: u is an alias for x** — Per TUI-ACT-02, `u` undoes a completed task. Since `x` already toggles both ways, `u` is implemented as the same toggle action. The semantic distinction matters for user mental model but the code path is identical.
- **D-13: Own-write FileChanged is harmless — ignore it** — Saving calls `task_list.save()` which triggers FileWatcher → `AppEvent::FileChanged`. In Phase 10, let the reload happen — the task list reloads to the same content (no data loss, no visible flicker for local files). Phase 13's TUI-UX-02 debounce requirement handles suppression.

### Status Bar

- **D-14: Layout split — last 1 row is status bar** — `ratatui::Layout::vertical([Constraint::Min(0), Constraint::Length(1)])` splits the frame into list area + 1-row footer. Standard ratatui pattern.
- **D-15: Status bar content** — Left side: `path/to/todo.txt | N tasks | V visible | D due today | O overdue`. Right side: `q quit | x done | j/k nav`.
- **D-16: Status bar left/right via Paragraph alignment** — Use two `Paragraph` widgets: one left-aligned (counts), one right-aligned (hints), both rendered in the same 1-row layout. Or a single `Line` built with `Span` spacers. Agent's discretion on implementation.
- **D-17: Monochrome in Phase 10** — No color on status bar counts. Phase 13 adds theme-aware colors (e.g., red overdue). Status bar uses same terminal default style.

### Quit

- **D-18: q and Ctrl+C to quit** — Carried forward from Phase 9. No change.

### the Agent's Discretion

- Exact module layout within `app.rs` vs splitting into `nav.rs`, `actions.rs` sub-modules — agent decides
- Whether `draw()` is split into `draw_task_list()` + `draw_status_bar()` helpers — agent decides
- How counts (due today, overdue) are computed — iterate `task_list.tasks()` comparing `.due_date` to today — agent implements
- Whether to pass `today: NaiveDate` into draw or compute it inline — agent decides

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 9 Foundation (existing code to extend)

- `crates/todotxt-tui/src/app.rs` — Current `App` struct and event loop to extend
- `crates/todotxt-tui/src/event.rs` — `AppEvent` enum to extend with new key variants if needed
- `crates/todotxt-tui/src/tui.rs` — `Tui` type alias and `TerminalGuard`
- `crates/todotxt-tui/src/main.rs` — Entry point wiring

### Core Library APIs

- `crates/todotxt-core/src/task_list.rs` — `update()`, `save()`, `tasks()`, `len()` signatures
- `crates/todotxt-core/src/task.rs` — `Task::with_completed()`, `Task.completed`, `Task.due_date`
- `crates/todotxt-core/src/lib.rs` — Public exports (what TUI crate can import)

### Phase 9 Decisions

- `.planning/phases/09-tui-foundation/09-CONTEXT.md` — All D-01..D-11 from Phase 9 remain in force

### Requirements

- `.planning/REQUIREMENTS.md` — TUI-NAV-01/02/03/04, TUI-ACT-01/02, TUI-UX-01/05, TUI-INFRA-03

### Research

- `.planning/research/PITFALLS.md` — ratatui 0.30 API notes (frame.area(), List/ListState patterns)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `App` struct (`app.rs`) — Has `should_quit`, `task_list`, `todo_path`. Phase 10 adds `selected: usize` and a `terminal_height: u16` field (needed for Ctrl+d/Ctrl+u).
- `AppEvent::Key(KeyEvent)` — Already wired in Phase 9. Phase 10 extends `handle_event()` with navigation and done/undo key branches.
- `TerminalGuard` — Unchanged. Phase 10 reads terminal dimensions for half-page scroll via `terminal.size()` in `run()`.
- `TaskList::update(index, task)` + `TaskList::save()` — Core write path for done/undo.
- `Task::with_completed(bool)` — Returns a new `Task` with toggled completion and today's date.

### Established Patterns

- `#![deny(warnings)]` — Must continue in all TUI crate files
- Event handling: match on `AppEvent::Key(key)` with `KeyEventKind::Press` filter — established in Phase 9
- `frame.area()` (not `frame.size()`) — ratatui 0.30 API, established in Phase 9

### Integration Points

- Phase 10 replaces `App::draw()` entirely — `Paragraph` → `List` + `ListState` + status bar `Layout`
- Phase 10 extends `App::handle_event()` — adds nav, done/undo branches
- `run()` loop gets `terminal_height` passed in or derived from `terminal.size()` for half-page scroll

</code_context>

<specifics>
## Specific Ideas

- Status bar format: `path/to/todo.txt | N tasks | V visible | D due today | O overdue` (left) + `q quit | x done | j/k nav` (right)
- x toggles both ways — user expectation: x on a done task undoes it, same key as marking done

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 10-core-tui*
*Context gathered: 2026-04-19*

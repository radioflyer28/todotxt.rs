# Research Summary — v1.1 TUI Interface

**Synthesized:** 2026-04-18  
**Sources:** STACK.md · FEATURES.md · ARCHITECTURE.md · PITFALLS.md  
**Confidence:** HIGH — all crate versions verified on crates.io; feature patterns drawn from comparable TUI projects (taskwarrior-tui, gitui, lazygit)

---

## Executive Summary

Building a ratatui-based TUI for todotxt.net is a well-trodden path with a clear, stable stack. The community has converged on `ratatui 0.30` + `crossterm 0.29` + `tui-textarea 0.7` + `color-eyre 0.6` — all verified current, cross-platform, and tokio-compatible. The existing `todotxt-core` crate already provides the `TaskList`, `FileWatcher`, and filter APIs the TUI needs; the new crate is a thin async shell around those primitives.

The biggest architectural risks are not rendering complexity but state-correctness: reloading the file while an editor buffer is open, mutating tasks by visible row position rather than by `source_index`, and letting the watcher's own-save events trigger spurious reloads. These are understood, preventable, and must be addressed in Foundation and Core TUI phases — not retrofitted later. Terminal lifecycle (raw mode, alternate screen, panic restore) must also be solved completely in Foundation before any feature work begins.

Feature scope is clear and well-bounded for v1.1: 10 table-stakes features (navigation, mark done, add, edit, delete, filter panel, sort toggle, status bar, theming, file-watch reload) plus a set of differentiators that are genuinely optional. The interaction model follows vim-like conventions, crossterm handles Windows/Unix parity natively, and no exotic dependencies are needed. An opinionated flat module structure (`tui.rs` / `app.rs` / `state.rs` / `ui.rs` / `components/`) is enough architecture without over-engineering.

---

## Stack Additions

### Add to `todotxt-tui/Cargo.toml`

| Crate | Version | Role |
|-------|---------|------|
| `ratatui` | `0.30` | Layout, widgets, rendering |
| `crossterm` | `0.29` + `event-stream` feature | Terminal backend + async event stream |
| `tui-textarea` | `0.7` | Inline task input/edit widget |
| `color-eyre` | `0.6` | Panic-safe terminal restore + rich errors |
| `futures` | `0.3` | `StreamExt` for `EventStream` |
| `tokio` | `{ workspace = true }` | Re-use existing workspace dep |
| `todotxt-core` | `{ path = "../todotxt-core" }` | Task model, file I/O, watcher |

**Action:** Promote `ratatui`, `crossterm`, `tui-textarea`, `color-eyre`, `futures` to `[workspace.dependencies]` so versions are pinned once. Run `cargo tree -d` after Foundation to verify no duplicate `crossterm` lines.

### Do NOT add

`termion` (Unix-only), `tui-rs` (deprecated), `cursive` / `iocraft` (different frameworks), `dialoguer` / `indicatif` (CLI-only), `async-std` (second runtime), `egui` / `iced` (GUI). No external autocomplete crate is needed — a ~50-line custom popup `List` covers todo.txt tag completion.

---

## Feature Table Stakes

All 10 must ship for v1.1 to feel usable. Missing any = product feels broken.

| # | Feature | Key Bindings | Notes |
|---|---------|-------------|-------|
| 1 | **Task list navigation** | `j`/`k`, `g`/`G`, `Ctrl+d`/`Ctrl+u`, arrows | `scroll_padding(2)`, truncate long text with `…` |
| 2 | **Mark done / undo** | `x` (toggle) | In-memory update first, then write; cursor stays on task |
| 3 | **Add new task** | `a` → type → `Enter` / `Esc` | `tui-textarea` single-line; `@`/`+` autocomplete popup |
| 4 | **Inline edit** | `e` → edit → `Enter` / `Esc` | Pre-populated with raw text; defer file reload during edit |
| 5 | **Delete with confirmation** | `d` → `y`/`n` | Modal overlay with task preview; `Esc` = cancel |
| 6 | **Filter panel** | `f` toggle; `Tab` focus cycle; `Space` toggle; `Ctrl+R` reset | Sidebar (>=60 cols) or full-screen overlay (<60 cols); live ANDed filters |
| 7 | **Sort toggle** | `s` (forward) / `S` (backward) | 6 modes: Priority -> Due -> File -> Alpha -> Project -> Context |
| 8 | **Status bar** | — | Counts, mode indicator, transient flash (1.5s) |
| 9 | **Themeable colors** | — | 2 built-ins (`default`/`light`); `[tui] theme` in config; `NO_COLOR` honored |
| 10 | **File-watch auto-reload** | `r` (force) | Silent in Normal mode; deferred in Add/Edit; cursor re-anchored by task ID |

**Task IDs always show original todo.txt line number** (not filtered index) to stay consistent with CLI semantics.

---

## Feature Differentiators

Nice-to-have. Do not block v1.1 ship on any of these.

| # | Feature | Key | Effort |
|---|---------|-----|--------|
| D1 | Task detail pane | `i` / `Enter` | Medium — right-side split with word-wrap, parsed fields |
| D2 | Scrollbar in task list | — | Low — 5 lines: `ScrollbarState` bound to `ListState.offset` |
| D3 | Sort persistence in config | — | Low — write `[tui] sort` on change |
| D4 | Session undo stack | `u` | Medium — `Vec<Task>` snapshot per mutation |
| D5 | Quick search via `/` | `/` | Low — bottom-bar live substring filter |
| D6 | Help overlay | `?` | Low — `Paragraph` in centered `Block::bordered()` popup |
| D7 | Bulk multi-select | `v` | High — visual mode, bulk mark/delete — defer to v1.2 |

**D2, D5, D6** are the easiest wins if time allows within a Polish phase.

---

## Architecture Highlights

### Module structure (flat and sufficient)

```
crates/todotxt-tui/src/
  main.rs          bootstrap: color-eyre, config, terminal init, tokio runtime
  tui.rs           terminal lifecycle guard (Drop restore), async event loop, watcher wiring
  app.rs           AppState owner, Action dispatch, core API calls
  action.rs        Internal intent enum (MoveDown, BeginEdit, SaveEdit, ReloadFromDisk …)
  mode.rs          Modal state enum (Normal, Add, Edit, Filter, DeleteConfirm)
  state.rs         AppState + VisibleTask + FilterState + StatusMessage + AutocompleteState
  event.rs         Unified AppEvent enum (Input, FileChanged, Tick, Resize, Quit)
  ui.rs            Frame composition — splits screen into list, filter panel, overlays, status bar
  config.rs        Loads same TOML schema as CLI; TUI-only keys under [tui] table
  autocomplete.rs  Tag suggestion extraction + popup state for +/@ completion
  components/
    task_list.rs   Render-only list widget
    editor.rs      tui-textarea wrapper (single-line, popup placement)
    filter_panel.rs Filter sidebar + focus rendering
    confirm.rs     Delete confirmation modal
    status_bar.rs  Footer widget
```

### Event loop

```
tokio::select! {
    terminal_event  -> map to AppEvent::Input
    watch_rx.recv() -> AppEvent::FileChanged
    tick_interval   -> AppEvent::Tick        (flash-message expiry)
    resize          -> redraw signal
}
```
Key events filtered to `KeyEventKind::Press` only. Watcher callback sends only `AppEvent::FileChanged` — no state mutation on the watcher thread.

### State management

- `AppState` owns `TaskList` (source of truth), `visible_tasks: Vec<VisibleTask>` (derived), selection as index into `visible_tasks`.
- `VisibleTask` carries `source_index` (into `TaskList`) and `display_id` (`source_index + 1`).
- All mutations resolve through `source_index`, never through visible row position or display ID.
- `rebuild_visible_tasks()` runs after every mutation, reload, filter change, or sort change; re-anchors selection by prior `source_index`.
- `pending_reload: bool` defers file-watch reloads while in Add / Edit / DeleteConfirm mode.

### Overlay rendering order (hard rule)

Delete confirmation -> autocomplete popup -> editor -> base task list. Rendered last = drawn on top. Input routed to top active layer only.

---

## Watch Out For

Top 5 pitfalls — each has a one-line prevention.

| # | Pitfall | Prevention |
|---|---------|------------|
| 1 | **Reload during active edit clobbers user input** | In Add/Edit/DeleteConfirm, set `pending_reload = true`; apply reload only after save or cancel |
| 2 | **Mutating by visible row instead of `source_index`** | Every write call uses `visible_tasks[sel].source_index`; never use `sel` or `display_id` directly |
| 3 | **Panic / early return leaves terminal in raw mode** | `tui.rs` guard type with `Drop` restore; `color-eyre` panic hook calls `ratatui::restore()` first |
| 4 | **TUI's own saves trigger spurious file-watch reloads** | After each local mutation, mark one watcher event as self-originated before coalescing events |
| 5 | **Workspace dependency skew (duplicate `crossterm`)** | Add `ratatui`, `crossterm`, `tui-textarea` to `[workspace.dependencies]`; `cargo tree -d` check in Foundation |

**Honorable mentions:** Filter `KeyEventKind::Press` only (prevents key duplication on Windows). Configure `tui-textarea` in single-line mode explicitly (prevents Enter/newline leaking into add/edit). Always recompute layout from `frame.area()` per draw (prevents stale geometry after resize). Start from ratatui 0.30 docs only, not community blog posts (prevents `Frame::size()` and import breakage).

---

## Build Order Recommendation

Suggested phase sequence for the roadmapper, in dependency order:

| Phase | Name | Delivers | Rationale |
|-------|------|----------|-----------|
| 1 | **Foundation** | New `todotxt-tui` crate, workspace deps pinned, `tui.rs` terminal guard, `color-eyre` panic hook, tokio event loop skeleton, watcher bridge, config loader | Must be airtight before any visible work. Terminal restore bugs are the hardest to retrofit. |
| 2 | **Core TUI** | Task list rendering, navigation (`j`/`k`/`g`/`G`/half-page), mark-done toggle, status bar, sort toggle, visible-list rebuild with `source_index` anchoring | Gets the app usable for read + mark-done. All identity and selection patterns established here. |
| 3 | **Edit Mode** | Add new task (`a`), inline edit (`e`), delete confirmation (`d`), `@`/`+` autocomplete popup, deferred reload during edit | Layered on top of the stable state model. Overlay ordering and single-line textarea config solved here. |
| 4 | **Filter Panel** | Filter sidebar (`f`), text search, context/project toggles, due-date bucket, show-done toggle, live ANDed filtering, `Ctrl+R` reset, narrow-terminal overlay fallback | Depends on stable visible-list rebuild from Phase 2. |
| 5 | **Theming** | `Theme` struct, `default`/`light` built-ins, `[tui] theme` config, custom theme TOML, `NO_COLOR` support, priority/done/overdue/selected color slots | Isolated rendering concern; no state dependencies. Can run in parallel with Phase 4 if needed. |
| 6 | **Polish** | Scrollbar (D2), quick search `/` (D5), help overlay `?` (D6), Unicode width fixes, narrow-terminal edge cases, `TestBackend` render smoke tests, state-machine unit tests | Everything that raises quality without blocking core functionality. |

**Research flags for planning:**
- Phase 1 (Foundation): standard patterns — no deep research needed; ratatui quickstart template covers >80%.
- Phase 3 (Edit Mode): consider a brief research pass on `tui-textarea` single-line configuration to confirm the exact API for disabling multiline defaults in v0.7.
- Phase 4 (Filter Panel): standard patterns — ratatui `List` + `Block` sidebar is well-documented.
- Phase 6 (Polish): `ratatui::backend::TestBackend` usage worth a quick research pass before writing tests.

---

## Sources

- STACK.md: crates.io (ratatui 0.30.0, crossterm 0.29.0, tui-textarea 0.7.0, color-eyre 0.6.5), ratatui.rs event-handling docs
- FEATURES.md: taskwarrior-tui, gitui, lazygit UX patterns; ratatui widget docs; todo.txt format spec
- ARCHITECTURE.md: ratatui component architecture guides; todotxt-core existing API surface
- PITFALLS.md: crossterm 0.29 docs (KeyEventKind, resize); ratatui 0.30 breaking changes; tui-textarea 0.7 docs; todotxt-core watcher and TaskList source
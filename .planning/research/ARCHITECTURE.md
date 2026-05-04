# Architecture Research — v1.6

**Researched:** 2026-05-04

## Summary

The v1.6 feature set is TUI-local: all ten seeds touch `crates/todotxt-tui/` and none require new public API in `todotxt-core`. The widest structural impact is SEED-008 (decouple GroupBy), which adds a new enum and replaces the `grouping: bool` field on `Pane` — every feature that reads or writes pane state must be built or reviewed after that change. View state persistence (SEED-007) sits at the end of the dependency chain because it must serialize whatever schema GroupBy and FilterHistory land on.

---

## Component Impact Map

| Feature (Seed) | Crate | Files Changed | New Types | Integration Point |
|----------------|-------|---------------|-----------|-------------------|
| SEED-006 TUI archive | todotxt-tui | `app.rs`, `config.rs` | — | Add `App::archive()` method using in-memory `task_list` + `config.done_file`; register `"archive"` action in `default_keymap()` |
| SEED-007 View state | todotxt-tui | `app.rs`, `config.rs`, new `view_state.rs` | `ViewState`, `PaneViewState` | Load in `App::new()`; save on quit alongside `persist_panes_on_quit()` |
| SEED-008 Decouple GroupBy | todotxt-tui | `app.rs`, `state.rs`, `config.rs` | `GroupBy` enum | Replace `Pane.grouping: bool` with `Pane.group_by: GroupBy`; replace `PaneConfig.group: bool` with `PaneConfig.group_by: GroupBy`; add `"group_cycle"` action to keymap |
| SEED-009 Bulk mark-done | todotxt-tui | `app.rs` | — | Extend `pane_toggle_done()` to check `selected_tasks` first; mirror `bulk_delete` dispatch pattern at line 855 |
| SEED-011 Filter history | todotxt-tui | `state.rs`, `app.rs` | `FilterHistory` struct | Embed in `FilteringState`; persist in `tui-state.toml` (SEED-007) |
| SEED-012 Open in $EDITOR | todotxt-tui | `app.rs`, `tui.rs` | — | `Tui::suspend()` / `Tui::resume()`; call `std::process::Command` to exec `$EDITOR`; trigger reload via `FileChanged` on return |
| SEED-013 Fix `+` autocomplete | todotxt-tui | `app.rs` | — | `collect_tokens()` dedup is case-naive (`sort+dedup`) vs `get_existing_projects()` in `state.rs` (case-insensitive HashMap); align them |
| SEED-014 Autocomplete in filter | todotxt-tui | `app.rs` | — | New `update_filter_autocomplete()` + `accept_filter_completion()` methods; call from `handle_filtering_key()` |
| SEED-015 Expand presets | todotxt-tui | `config.rs`, `app.rs` | extended `TuiPreset` fields | Add `sort`, `group_by`, `label` to `TuiPreset`; apply all fields in the `'1'..='9'` arm of `handle_filtering_key()` |
| SEED-005 Phase 22 tests | todotxt-tui | `app.rs` (test module) | extended `make_app_with_tasks` | Add builder variants: `make_app_with_config()`, `make_app_with_panes()` to support new field combinations |

---

## New Components Required

### `GroupBy` enum — `crates/todotxt-tui/src/state.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupBy {
    #[default]
    None,
    Priority,
    Project,
    Context,
    DueDate,
    Status,       // completed vs active
}
```

**Location rationale:** `todotxt-tui` only. Grouping is a display concern; `SortOrder` in `todotxt-core` governs data order. `GroupBy` governs visual bucketing. Mixing them in `todotxt-core` would pollute the library with rendering semantics.

`group_key_for(task, sort)` in `app.rs` becomes `group_key_for(task, group_by: GroupBy)` — decoupled from sort order. The boolean `pane.grouping` flag is removed; `GroupBy::None` is the "off" state.

`PaneConfig.group: bool` → `PaneConfig.group_by: GroupBy`. This is a **breaking config change** — add a migration shim in `TuiConfig::load()` that detects `group = true` with no `group_by` key and defaults to the previous behavior (group by sort order). Practically: default `GroupBy::None`; if old `group = true` was set, map it to `GroupBy::Priority` as the conservative fallback.

---

### `FilterHistory` struct — `crates/todotxt-tui/src/state.rs`

```rust
pub struct FilterHistory {
    pub entries: Vec<String>,   // most-recent last
    pub cursor: Option<usize>,  // None = not navigating
    pub max: usize,             // cap at 50
}
```

Embed inside `FilteringState` so history is active only while the filter panel is open. On `Enter` (filter applied): push the non-empty query to history. Use a dedicated key (e.g., `Ctrl+P`/`Ctrl+N`) to navigate history — do NOT reuse `Up`/`Down` which already navigates the preset list. Persist in `tui-state.toml` across sessions (SEED-007).

---

### `ViewState` + `tui-state.toml` — new `crates/todotxt-tui/src/view_state.rs`

```rust
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ViewState {
    pub active_pane: usize,
    pub filter_history: Vec<String>,   // FilterHistory.entries serialized
}
```

**Why separate from `config.toml`?** `config.toml` is user-edited; `tui-state.toml` is machine-written session state. Mixing them creates merge conflicts and user confusion. `persist_panes_on_quit()` already writes pane layouts to `config.toml`; `tui-state.toml` carries ephemeral runtime state that shouldn't pollute config.

**Path resolution:** Same directory as `config.toml`, named `tui-state.toml`. Use the existing `resolve_config_path()` from `todotxt-core::portable` — identical to how `config_path` is resolved in `main.rs`. Add `ViewState::path_from_config(config_path: &Path) -> PathBuf` helper.

---

### Extended `TuiPreset` — `crates/todotxt-tui/src/config.rs`

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TuiPreset {
    pub filter:   Option<String>,
    pub sort:     Option<PaneSort>,
    pub group_by: Option<GroupBy>,   // added after SEED-008
    pub label:    Option<String>,    // status bar label while preset is active
}
```

The `'1'..='9'` arm in `handle_filtering_key()` currently only applies `filter`. Extend it to also set `pane.sort_order` and `pane.group_by` if the preset specifies them — making presets act as complete "view snapshots."

---

### `Tui::suspend()` / `Tui::resume()` — `crates/todotxt-tui/src/tui.rs`

For SEED-012 (open in `$EDITOR`), the TUI must restore the terminal to cooked mode, exec the editor, and re-enter raw mode on return:

```rust
impl Tui {
    pub fn suspend(&mut self) -> color_eyre::Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(self.terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen)?;
        Ok(())
    }
    pub fn resume(&mut self) -> color_eyre::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(self.terminal.backend_mut(),
            crossterm::terminal::EnterAlternateScreen)?;
        self.terminal.clear()?;
        Ok(())
    }
}
```

After editor exits, send a synthetic `AppEvent::FileChanged` so `App` reloads `task_list` from disk (the editor may have changed any task, not just the selected one).

---

## Suggested Build Order

```
Wave 1 — No deps, self-contained
  1. SEED-013  Fix + project autocomplete   (collect_tokens dedup alignment)
  2. SEED-012  Open in $EDITOR              (tui.rs suspend/resume, independent)
  3. SEED-009  Bulk mark-done               (toggle_done + selected_tasks check)
  4. SEED-006  TUI archive hotkey           (App::archive(), keymap entry)

Wave 2 — Structural (must land together or in order)
  5. SEED-008  Decouple GroupBy             (GroupBy enum, Pane.group_by replaces .grouping)
  6. SEED-005  Extend test helper           (add make_app_with_config/panes after GroupBy is real)

Wave 3 — Builds on GroupBy
  7. SEED-015  Expand numeric presets       (TuiPreset gains sort/group_by; requires GroupBy)
  8. SEED-011  Filter history               (FilterHistory in state.rs; needed by SEED-014)

Wave 4 — Builds on history + fixed autocomplete
  9. SEED-014  Autocomplete in filter       (requires SEED-013 fix + SEED-011 history)

Wave 5 — Serializes everything
  10. SEED-007  View state persistence      (serializes GroupBy from SEED-008, FilterHistory from SEED-011)
```

**Dependency rationale:**
- SEED-008 before SEED-015: `TuiPreset.group_by: Option<GroupBy>` can't compile until `GroupBy` exists.
- SEED-008 before SEED-007: `ViewState` schema depends on whether `GroupBy` is a field in `Pane`.
- SEED-011 before SEED-014: history navigation in filter panel must be designed before autocomplete adds another key consumer.
- SEED-013 before SEED-014: filter autocomplete calls the same `collect_tokens` machinery; fix the data layer first.
- SEED-005 after SEED-008: test helper creating a `Pane` must construct `group_by` correctly.

---

## Key Architectural Decisions

### 1. Archive logic: TUI-local, not moved to `todotxt-core`

**Decision:** Implement `App::archive()` in `app.rs`. Do NOT move `run_archive()` to `todotxt-core`.

**Rationale:** `run_archive()` in the CLI takes `&Config` (CLI-only) and `&Renderer` (CLI-only). The TUI already has `task_list` loaded in memory and `config.done_file` resolved — its archive method is ~35 lines of atomic file I/O, simpler than the CLI version (no JSON renderer, no reload). Code duplication is minimal and the two callers have different contracts.

### 2. GroupBy lives in `todotxt-tui`, not `todotxt-core`

**Decision:** `GroupBy` enum in `todotxt-tui/src/state.rs`. `SortOrder` stays in `todotxt-core`.

**Rationale:** Grouping is a rendering strategy (how to draw visual buckets). `SortOrder` controls query-time data ordering — a library concern. `GroupBy` added to `todotxt-core` would introduce rendering semantics into a library with no other TUI consumers.

### 3. `tui-state.toml` is a separate sidecar from `config.toml`

**Decision:** Session/runtime state in `~/.todotxt.rs/tui-state.toml`; user config stays in `config.toml`.

**Rationale:** `config.toml` is intentionally user-editable; writing machine state into it creates noise in diffs. `persist_panes_on_quit()` already writes pane layouts to config — that's intentional (user-visible preference). Filter history and active pane index are runtime artifacts, not preferences.

### 4. `accept_completion` is NOT generalized — two methods

**Decision:** Add `accept_filter_completion()` targeting `filter_state.editor`. Keep `accept_completion()` targeting `self.editor`.

**Rationale:** Rust's borrow checker makes a shared helper require passing `&mut TextArea` + `&mut Option<AutocompleteState>` separately, which is more awkward than two ~30-line methods. The filter completion may also diverge (e.g., no date autocomplete in filter mode).

### 5. `collect_tokens` dedup is the SEED-013 bug root cause

`collect_tokens()` uses `vec.sort() + vec.dedup()` which is byte-exact — `"Work"` and `"work"` appear as two entries. `get_existing_projects()` in `state.rs` uses a case-insensitive HashMap. **Fix:** replace `collect_tokens` internals to call `get_existing_contexts`/`get_existing_projects` from `state.rs`, then pass through `rank_matches()` for ordering. This unifies dedup logic to one implementation across the codebase.

---

## Risks & Warnings

### Risk 1: `GroupBy` config migration (SEED-008) — HIGH

`PaneConfig.group: bool` → `PaneConfig.group_by: GroupBy` is a breaking TOML schema change. Any `config.toml` with `group = true` under a `[[panes]]` block will fail to deserialize. **Mitigation:** Keep `group: bool` as a deprecated `#[serde(default)]` field; in `panes_from_config()` map `group = true` → `GroupBy::Priority` (the pre-SEED-008 behavior when sort was Priority) before the new field takes over.

### Risk 2: `persist_panes_on_quit` + `tui-state.toml` write race (SEED-007)

Both writes happen on quit in `App::run()`. If one fails, the other has already been committed. Ensure both use temp-file rename (atomic) and run both even if one fails — don't use `?` between them; capture both results and return the first error after both have attempted.

### Risk 3: `$EDITOR` suspension corrupts terminal on error (SEED-012)

If the editor crashes or `resume()` fails, the terminal is left in a bad state. Wrap the edit-in-editor call in a guard that calls `resume()` even on `Err`. The existing `color_eyre` panic hook restores the terminal for panics; extend the same pattern for editor errors.

### Risk 4: Filter history vs. preset navigation key conflict (SEED-011 + SEED-014)

`handle_filtering_key()` already uses `Up`/`Down` to navigate the preset list. History navigation MUST use a different key (e.g., `Ctrl+P`/`Ctrl+N`). Lock this decision before SEED-014 implementation to avoid re-doing the key handler when autocomplete is added.

### Risk 5: `make_app_with_tasks` leaks temp files (SEED-005)

The helper calls `file.keep()` to prevent deletion while `TaskList` holds the path. Each test leaks a temp file. Fix in SEED-005 when extending the helper: wrap in a struct that implements `Drop` and deletes the file.

### Risk 6: `TuiPreset` expansion + `persist_panes_on_quit` (SEED-015)

When a preset is applied (sort + group_by + filter all set), `persist_panes_on_quit` will write those values back to `PaneConfig`. On next load the pane starts with the preset's values, which may not be desired. **Decision needed before SEED-015:** preset application is session-only (reset on quit) or persistent (written to config). Leaving it ambiguous will cause user confusion.

---

# Architecture Research — v1.3 Parity Work

**Researched:** 2026-04-24

## Existing Integration Points

| Layer | Existing behavior | v1.3 impact |
| ----- | ----------------- | ----------- |
| `App` selection | Single selected row over `display_rows` | Needs anchor + multi-select state |
| `display_rows` | Task rows plus non-selectable group headers | Selection logic must skip headers cleanly |
| Task mutation handlers | Single canonical-selected task for delete/edit/toggle | Needs bulk mutation paths keyed to canonical task indices |
| `todotxt-core::Task` | Parses and rebuilds priority, dates, projects, contexts, body | Best place for token-aware helpers or normalization logic |

## Recommended Build Order

1. Introduce a canonical selection model in the TUI.
2. Add bulk delete / append plumbing using that model.
3. Add smart normalization helpers in core and route append/edit through them.
4. Close with hotkey/help parity and regression coverage.

## Data-Flow Guidance

- Maintain selection as canonical task indices or stable task identity, not visible row positions.
- When grouping/filtering changes visible rows, recompute row highlights from canonical selection.
- Bulk operations should collect target canonical indices, sort them safely, then mutate in a deterministic order.

## UI Model Guidance

- Support two related modes:
    - anchored range extension via Shift+movement
    - explicit selection mode for disjoint picks, similar in spirit to visual-line selection
- Selected rows need distinct styling from the currently focused row.
- Status/help surface should show when selection mode is active and how many tasks are selected.

## Dependency Notes

- The current TUI already handles grouped non-task rows, filter panels, and modal dialogs. Reuse those patterns rather than introducing a second navigation abstraction.
- Bulk delete confirmation should preview count and maybe the first few tasks, not just a single row.
# Architecture Research — v1.1 TUI Interface

## Module Structure

Recommended crate layout:

```text
crates/
  todotxt-tui/
    Cargo.toml
    src/
      main.rs
      app.rs
      action.rs
      mode.rs
      state.rs
      event.rs
      tui.rs
      ui.rs
      config.rs
      autocomplete.rs
      components/
        mod.rs
        task_list.rs
        editor.rs
        filter_panel.rs
        confirm.rs
        status_bar.rs
```

Purpose of each module:

- `main.rs`: bootstrap. Install `color-eyre`, load config, resolve `todo.txt` path, create terminal, start tokio runtime, run the app, and always restore the terminal on exit.
- `app.rs`: high-level application controller. Owns `AppState`, handles `Action`s, calls core APIs, decides when to quit and when to redraw.
- `action.rs`: internal intent enum. Converts raw terminal or watcher events into app-level actions such as `MoveDown`, `BeginEdit`, `SaveEdit`, `ReloadFromDisk`, `ToggleSort`, `ConfirmDelete`.
- `mode.rs`: modal state definitions. Keeps mode switching explicit and prevents keybinding ambiguity.
- `state.rs`: `AppState` plus small helper structs for selection, flash messages, overlay state, and derived visible rows.
- `event.rs`: event unification layer. Defines the async event enum received by the app loop, combining crossterm input, watcher notifications, resize, tick, and optional render tick.
- `tui.rs`: terminal lifecycle and async event loop. Encapsulates raw mode, alternate screen, `EventStream`, teardown, and watcher channel wiring.
- `ui.rs`: top-level frame composition. Splits the screen into list area, optional filter panel, transient overlays, and status bar.
- `config.rs`: TUI-facing config loader and adapter. Prefer reusing the same TOML schema as CLI, either by extracting it into shared code later or initially duplicating only the deserialization shape.
- `autocomplete.rs`: tag suggestion extraction and popup state for `+project` and `@context` completion inside add/edit inputs.
- `components/task_list.rs`: render-only task table/list widget. Knows how to draw selected row, completion styling, due styling, and empty states.
- `components/editor.rs`: add/edit input widget wrapper around `tui-textarea`, including single-line behavior and popup placement.
- `components/filter_panel.rs`: filter sidebar widget plus focus rendering for filter sections.
- `components/confirm.rs`: delete confirmation modal.
- `components/status_bar.rs`: footer widget for mode, counts, sort, filters, and transient success/error messages.

This is intentionally flatter than the full ratatui component template. For a focused workspace binary, the important split is:

- terminal lifecycle and event ingestion in `tui.rs`
- business state and mutations in `app.rs` and `state.rs`
- pure rendering in `ui.rs` and `components/*`

That is enough structure without drifting into a mini framework.

## App State

Recommended state shape:

```rust
pub struct AppState {
    pub task_list: todotxt_core::TaskList,
    pub visible_tasks: Vec<VisibleTask>,
    pub selected: Option<usize>,
    pub mode: Mode,
    pub filter_state: FilterState,
    pub sort_order: SortChoice,
    pub status: StatusMessage,
    pub config: TuiConfig,
    pub should_quit: bool,
    pub dirty_render: bool,
    pub pending_reload: bool,
    pub autocomplete: Option<AutocompleteState>,
}

pub struct VisibleTask {
    pub source_index: usize,
    pub display_id: usize,
}

pub enum Mode {
    Normal,
    Add(EditorState),
    Edit(EditSession),
    Filter(FilterFocus),
    DeleteConfirm(DeleteTarget),
}
```

Additional supporting structs:

- `FilterState`: text query, selected projects, selected contexts, due bucket, show/hide completed, show/hide hidden, show/hide future threshold.
- `EditorState`: `tui_textarea::TextArea`, prompt label, cursor anchor for popup placement, optional validation error.
- `EditSession`: original `source_index`, original raw text, current `EditorState`. Keep the original index and text together so cancel and save are trivial.
- `FilterFocus`: which panel section is focused, plus per-list cursor positions for projects and contexts.
- `DeleteTarget`: source index and a short preview string for the modal.
- `StatusMessage`: persistent summary plus optional transient flash message with expiry.
- `TuiConfig`: resolved todo path plus a small set of TUI-only knobs such as keymap choice, watch enabled, show_done default, and perhaps initial sort.
- `AutocompleteState`: trigger kind (`Project` or `Context`), query fragment, suggestions, highlighted suggestion index.

What lives in state versus what is derived on render:

Keep in state:

- `TaskList`, because all mutations and reloads go through it.
- Current mode and overlay state.
- Current selection as an index into `visible_tasks`, not into `TaskList` directly.
- Filter inputs and sort choice.
- Textareas for add, edit, and filter text input.
- Pending reload flag when an external file change arrives during add or edit.
- Transient status message state.

Derive on refresh, not per-widget:

- `visible_tasks`: recompute after any mutation, reload, filter change, or sort change.
- Counts for visible, overdue, due today.
- Active filter summary string.
- Current task reference from `selected` plus `visible_tasks[source_index]`.
- Autocomplete candidates from existing tasks, unless profiling later shows this needs caching.

Why this split works:

- Core remains the source of truth for file-backed tasks.
- The TUI never mutates a filtered or sorted copy of tasks directly.
- Selection is resilient because it is tracked against a derived visible list but can be re-anchored by `source_index` after reloads and resorting.

Recommended visible-list refresh method:

```rust
fn rebuild_visible_tasks(&mut self) {
    let filter = self.filter_state.to_core_filter();
    let mut rows: Vec<VisibleTask> = self
        .task_list
        .filter(&filter)
        .into_iter()
        .map(|(source_index, _task)| VisibleTask {
            source_index,
            display_id: source_index + 1,
        })
        .collect();

    self.sort_order.sort_visible(&mut rows, self.task_list.tasks());
    self.reconcile_selection();
}
```

Important detail: use `source_index + 1` as the displayed task id so the TUI stays aligned with existing CLI semantics.

## Event Loop

Recommended runtime model:

- Use `#[tokio::main]` in `main.rs`.
- Use `crossterm::event::EventStream` with the `event-stream` feature enabled.
- Convert file-watch callbacks into a tokio `mpsc::UnboundedSender<AppEvent>`.
- Centralize all event intake in `tui.rs` and send normalized `AppEvent`s to the app loop.

Suggested event enum:

```rust
pub enum AppEvent {
    Init,
    Input(crossterm::event::Event),
    FileChanged,
    Tick,
    Render,
    Error(String),
    Quit,
}
```

Recommended loop shape:

```rust
loop {
    let event = tokio::select! {
        maybe_term = terminal_events.next() => map_terminal_event(maybe_term),
        maybe_watch = watch_rx.recv() => map_watch_event(maybe_watch),
        _ = tick_interval.tick() => AppEvent::Tick,
        _ = render_interval.tick() => AppEvent::Render,
    };

    let action = app.handle_event(event)?;
    if action.requests_draw() {
        terminal.draw(|frame| ui::render(frame, &app.state))?;
    }
    if app.state.should_quit {
        break;
    }
}
```

Design recommendations:

- Filter terminal key events to `KeyEventKind::Press`. Ratatui's own guidance calls this out because Windows emits both press and release events.
- Keep the watcher callback dumb. It should only send `AppEvent::FileChanged`; actual reload logic belongs in `app.rs`.
- Do not call `TaskList::reload()` inside the watcher thread.
- Use `Tick` for flash-message expiry and minor housekeeping.
- Use `Render` only if you want a bounded frame rate; otherwise a simpler dirty-render model is enough for this app. For a focused todo TUI, redraw-on-change plus resize is sufficient and simpler.

Recommended practical simplification:

- Start without a separate render FPS.
- Redraw after every handled input, watcher reload, and resize event.
- Keep only a low-frequency `Tick` for message expiry.

That gives a simpler loop:

```rust
loop {
    let event = tokio::select! {
        maybe_term = terminal_events.next() => map_terminal_event(maybe_term),
        maybe_watch = watch_rx.recv() => map_watch_event(maybe_watch),
        _ = tick_interval.tick() => AppEvent::Tick,
    };

    let redraw = app.handle_event(event)?;
    if redraw {
        terminal.draw(|frame| ui::render(frame, &app.state))?;
    }
    if app.state.should_quit {
        break;
    }
}
```

File watch behavior:

- In `Normal` or `Filter` mode: reload immediately, rebuild visible rows, preserve selection by prior `source_index` when possible, flash `Reloaded from disk`.
- In `Add` or `Edit` mode: set `pending_reload = true` and flash `File changed on disk; reload pending`. Apply the reload after the user confirms or cancels the editor.
- In `DeleteConfirm`: also defer reload until the modal closes. This avoids deleting against a just-shifted index.

## Component Breakdown

Recommended UI components and responsibilities:

- `task_list` widget: main scrollable task view. Takes `visible_tasks`, selected row, current terminal area, and styling helpers. It should not know anything about file I/O or modes besides whether inline editing is active for the selected row.
- `editor` widget: reused by add and edit modes. Owns single-line textarea rendering, popup placement for autocomplete, and input hint text.
- `filter_panel` widget: sidebar or narrow-screen overlay. Renders current filters and focus state. It does not own filter application logic.
- `confirm` modal: delete confirmation overlay with clear/cancel background and explicit yes/no prompt.
- `status_bar` widget: always-visible footer. Shows total and visible counts, due stats, sort order, active filter summary, and mode label.
- `ui::render`: screen layout coordinator. Chooses between full-width list or list plus sidebar, then renders overlays in strict order.

Suggested render order:

1. Main list area
2. Filter sidebar if open
3. Inline editor or add editor if active
4. Autocomplete popup if active
5. Delete confirmation modal last
6. Status bar always last on the base layer, but modal overlays visually sit above content area

Complexity estimate per component:

| Component | Complexity | Notes |
|----------|------------|-------|
| Terminal wrapper and event loop | Medium | Straightforward once watcher channel is decided |
| App state and selection reconciliation | Medium | Most important correctness logic in the crate |
| Task list renderer | Low | Mostly formatting and scroll behavior |
| Add and edit editor wrapper | Medium | `tui-textarea` integration plus single-line constraints |
| Filter panel | Medium | More state than rendering difficulty |
| Delete confirmation modal | Low | Small isolated overlay |
| Autocomplete popup | Medium | Needs token parsing and focus handoff but stays local |
| File-watch reload integration | Medium | Mainly mode-sensitive behavior and selection preservation |
| Status bar and flash messages | Low | Small but high-value polish |

## Build Order

Suggested phased implementation order:

1. Terminal shell and crate wiring
   - Create `crates/todotxt-tui`.
   - Add workspace member and dependencies (`ratatui`, `crossterm` with `event-stream`, `tokio`, `futures`, `tui-textarea`, `color-eyre`).
   - Implement `tui.rs` setup/teardown and minimal `main.rs`.
   - Rationale: nothing else matters until raw mode, alternate screen, and controlled exit are reliable.

2. Read-only task list view
   - Load config and `TaskList`.
   - Implement `AppState`, `visible_tasks`, selection, and the task list renderer.
   - Support navigation keys, resize handling, and quit.
   - Rationale: establishes the core data flow from `todotxt-core` into a ratatui view without mutation complexity.

3. Filter and sort foundation
   - Add `FilterState`, conversion into `todotxt_core::Filter`, and sort cycling against `SortOrder`.
   - Rebuild visible rows after state changes.
   - Add status bar summaries.
   - Rationale: filtering and sorting define the list identity and selection behavior that all later edits depend on.

4. Add mode
   - Add `EditorState`, single-line textarea wrapper, validation, and `TaskList::add()`.
   - Re-anchor selection on the newly inserted task.
   - Rationale: simplest mutation path and good proof that editing state plus redraw works.

5. Edit mode
   - Inline selected-row editing using raw task text.
   - Save with `TaskList::update(source_index, Task::parse(new_raw))`.
   - Handle cancel and reload deferral.
   - Rationale: same editor machinery as add mode, but with higher correctness pressure around index stability.

6. Delete confirmation
   - Modal overlay and `TaskList::delete(source_index)`.
   - Reconcile selection after deletion.
   - Rationale: small mutation feature that exercises confirmation flow and modal precedence.

7. File-watch auto-reload
   - Bridge watcher notifications into the async loop.
   - Add deferred reload behavior for editor modes.
   - Rationale: adds concurrency after the basic state machine is already stable.

8. Autocomplete popup
   - Extract tags from current tasks and offer project/context completion while editing.
   - Accept suggestion insertion and popup navigation.
   - Rationale: purely ergonomic layer that depends on the editor modes already existing.

9. Polish and resilience
   - Flash messages, better empty states, key help, terminal restore tests where possible, and panic-safe teardown.
   - Rationale: final hardening after the interaction model is stable.

Dependency rationale:

- Selection reconciliation must exist before sort, edit, delete, and reload are trustworthy.
- Add/edit share editor infrastructure, so add should come first to reduce risk.
- Watcher integration should come after edit mode so deferred reload semantics have something real to protect.
- Autocomplete should come after add/edit because it is an enhancement, not a foundation.

## Codebase Integration Points

Direct `todotxt-core` touchpoints:

- `TaskList::load(path)`: initial file load.
- `TaskList::reload()`: external-change refresh.
- `TaskList::tasks()`: read-only access for rendering, autocomplete extraction, counts, and sort comparisons.
- `TaskList::filter(&Filter)`: base filtered row set with stable source indices.
- `TaskList::add(Task)`: add flow.
- `TaskList::update(index, Task)`: edit flow and completion toggle if implemented as update-in-place.
- `TaskList::delete(index)`: delete flow.
- `Task::parse(raw)`: add/edit submit path from textarea text.
- `Task::with_completed(bool)`: toggle done/undo from the list.
- `Filter::from_query(...)` or a TUI-built `Filter`: map panel state into core filtering rules.
- `SortOrder`: reuse existing sort semantics instead of inventing TUI-local ordering.
- `FileWatcher` behind the `watching` feature: preferred watch implementation if it can be bridged cleanly into the tokio loop.

Recommended watcher integration approach:

- Reuse `todotxt_core::FileWatcher` instead of reimplementing `notify` in the TUI.
- In `tui.rs`, create an unbounded tokio sender and pass an `Arc<dyn Fn()>` callback that calls `watch_tx.send(AppEvent::FileChanged)`.
- Keep the watcher instance alive in a small wrapper owned by `App` or `TuiRuntime`.

Why reuse core watcher:

- The core watcher already handles the important atomic-write detail by watching the parent directory instead of the file.
- Reimplementing this in the TUI duplicates the exact subtlety that core already solved.

Likely core changes worth considering:

1. Move config schema out of CLI if both binaries should share it.
   - Today `Config` lives in `crates/todotxt-cli/src/config.rs`, which the TUI cannot reuse without duplicating code or creating a library target in CLI.
   - Best long-term fix: extract config types and path resolution into either `todotxt-core` or a new tiny shared crate such as `crates/todotxt-app` or `crates/todotxt-config`.
   - Short-term acceptable fallback: duplicate the deserialization struct in TUI and keep the schema identical.

2. Add a core helper for metadata extraction if autocomplete/filter lists need it often.
   - The TUI can derive projects and contexts from `task_list.tasks()` initially.
   - No core change is required for v1.1, but a helper like `TaskList::projects()` and `TaskList::contexts()` would reduce repeated iteration in multiple frontends later.

3. Consider a non-callback watcher API only if the callback bridge becomes awkward.
   - Not required now. The existing callback-based watcher is sufficient for the TUI.

New files expected:

- `crates/todotxt-tui/Cargo.toml`
- `crates/todotxt-tui/src/main.rs`
- `crates/todotxt-tui/src/app.rs`
- `crates/todotxt-tui/src/action.rs`
- `crates/todotxt-tui/src/mode.rs`
- `crates/todotxt-tui/src/state.rs`
- `crates/todotxt-tui/src/event.rs`
- `crates/todotxt-tui/src/tui.rs`
- `crates/todotxt-tui/src/ui.rs`
- `crates/todotxt-tui/src/config.rs`
- `crates/todotxt-tui/src/autocomplete.rs`
- `crates/todotxt-tui/src/components/mod.rs`
- `crates/todotxt-tui/src/components/task_list.rs`
- `crates/todotxt-tui/src/components/editor.rs`
- `crates/todotxt-tui/src/components/filter_panel.rs`
- `crates/todotxt-tui/src/components/confirm.rs`
- `crates/todotxt-tui/src/components/status_bar.rs`

Modified existing files expected:

- `Cargo.toml` at repo root to add the new workspace member and shared workspace dependencies.
- Potentially `crates/todotxt-core/Cargo.toml` only if the watcher feature wiring needs adjustment for the TUI build.
- Potentially `crates/todotxt-core/src/lib.rs` only if new shared config or metadata helpers are extracted later.
- Potentially CLI config code only if config is moved into shared code.

Recommendation on config reuse:

- Reuse the same TOML file and schema.
- Do not create a separate TUI config file for v1.1 unless the TUI genuinely needs settings that would confuse CLI users.
- If TUI-only settings appear, nest them under a dedicated table such as `[tui]` in the same config file rather than splitting config ownership.

Recommendation on error handling:

- Use `color-eyre` in `main.rs` and install it before terminal setup.
- Wrap the terminal lifecycle in a guard type from `tui.rs` whose `Drop` implementation restores raw mode, cursor, and alternate screen.
- Do not rely only on normal-path teardown. Panic-safe restoration matters more for a TUI than for the CLI.
- Return rich `Result<()>` from app methods and convert operational errors into status-bar flashes when recovery is possible.
- Reserve process-level error returns for startup failures, terminal initialization failures, and unrecoverable runtime failures.

Practical recommendation:

- Startup errors: bubble up and let `color-eyre` print them after terminal restore.
- In-app mutation errors: catch in `app.rs`, flash in status bar, keep running.
- Watcher errors: treat as non-fatal if the watcher drops or cannot notify, but show a one-time warning.
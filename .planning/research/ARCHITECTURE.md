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
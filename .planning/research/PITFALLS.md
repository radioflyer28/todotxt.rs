# Pitfalls Research - v1.1 TUI Interface

## Critical (data loss / corrupted terminal)

### Reloading the file while an editor buffer is open
**Warning:** `todotxt-core::FileWatcher` calls back from a background debouncer thread, and `TaskList::reload()` replaces the in-memory task vector. If the TUI reloads immediately while `Add`, `Edit`, or delete confirmation is open, the user can lose unsaved textarea edits or save against a task that has moved underneath them.
**Prevention:** Treat watcher events as notifications only. In the callback, send a `FileChanged` event into the async app loop and do not reload there. In `Normal` mode, reload immediately. In `Add`, `Edit`, and `DeleteConfirm`, set `pending_reload = true`, show a status warning, and only apply the reload after save or cancel. On save, revalidate the target by stored `source_index` and optionally compare the original raw text before overwriting.
**Phase:** Edit Mode

### Using visible row or displayed ID as task identity
**Warning:** This codebase already uses index-based identity in `TaskList::update`, `TaskList::delete`, and `TaskList::filter`, while the planned TUI display ID stays aligned with CLI semantics as `source_index + 1`. If the TUI mutates by selected row number after filtering or sorting, it will edit or delete the wrong task.
**Prevention:** Store selection as an index into `visible_tasks`, and store `source_index` inside each visible row. Every mutation must resolve back to `source_index`; never mutate by row position or by the displayed ID shown in the UI. After every reload, filter change, or sort change, rebuild visible rows and re-anchor selection by prior `source_index`, not by list position.
**Phase:** Core TUI

### Terminal restore gaps after panic or early return
**Warning:** A new TUI crate will enable raw mode, switch to the alternate screen, and hide the cursor. If any startup error, panic, or `?` return escapes before cleanup, the user's terminal stays corrupted. This is much easier to trigger in a mixed CLI plus TUI workspace because config loading, file loading, and watcher startup can all fail before the main loop settles.
**Prevention:** Isolate terminal setup and teardown in `tui.rs` behind a guard type with `Drop` restoration. Install `color-eyre` before terminal setup, but print rich errors only after terminal restoration. Register a panic hook that restores the terminal first, and avoid `println!` or `eprintln!` while the alternate screen is active.
**Phase:** Foundation

### Treating self-originated saves as external reloads
**Warning:** `TaskList::save()` uses atomic replace semantics and the existing watcher intentionally watches the parent directory because renames are how saves land on disk. That means the TUI will receive watcher events for its own add, edit, and delete operations. If every watcher event is treated as a true external change, the UI can immediately reload after a successful local save, flash misleading warnings, reset selection, or stomp pending editor state.
**Prevention:** Add a small self-write suppression policy. After a successful local mutation, mark one watcher notification as locally originated, or compare file metadata before reloading. Coalesce repeated file events into a single pending reload flag rather than reloading for every debounced callback.
**Phase:** Core TUI

## Significant (bugs / poor UX)

### Workspace dependency skew between ratatui, crossterm, and tui-textarea
**Warning:** The workspace currently centralizes shared versions in `[workspace.dependencies]`, but it does not yet contain `ratatui`, `crossterm`, `tokio`, `futures`, `tui-textarea`, or `color-eyre`. Adding them ad hoc inside only `todotxt-tui` makes it easy to drift into duplicate `crossterm` versions or incompatible backend feature selection. Ratatui 0.30 explicitly supports multiple crossterm versions via features, and tui-textarea expects backend versions to match the stack you are actually using.
**Prevention:** Promote the new TUI stack to `[workspace.dependencies]` and pin the intended versions once for the workspace. Keep Cargo resolver 2. Run `cargo tree -d` before closing Foundation work and make sure there is only one intended crossterm line in the final graph. If you disable ratatui default features, re-enable the exact backend and any needed layout-cache features explicitly instead of assuming older blog-post snippets still apply.
**Phase:** Foundation

### Bridging the synchronous watcher callback into tokio the wrong way
**Warning:** The existing core watcher is callback-based and not async. A common mistake is to call async runtime APIs incorrectly from that callback, or worse, to touch `AppState` or `TaskList` directly from the watcher thread. That introduces race-prone cross-thread mutation into what should remain a single-threaded state machine.
**Prevention:** Keep the callback dumb. Give it an `Arc<dyn Fn()>` that only sends `AppEvent::FileChanged` into a `tokio::sync::mpsc::UnboundedSender`. Keep all reload logic, state mutation, and redraw decisions inside the main app loop. If the watcher fails, degrade gracefully and show a one-time warning instead of crashing the TUI.
**Phase:** Foundation

### Key event duplication from press, repeat, and release handling
**Warning:** Crossterm 0.29 exposes `KeyEventKind::{Press, Repeat, Release}` and convenience helpers like `Event::as_key_press_event()`. If the TUI processes all key events, commands can fire twice on Windows-like terminals or keep firing unexpectedly on repeat. This becomes worse when both the TUI command layer and `tui-textarea` see the same event stream.
**Prevention:** Filter command dispatch to key press events only unless repeat is intentionally supported for navigation. Prefer `event.as_key_press_event()` or `key.kind.is_press()` semantics instead of broad `Event::Key(_)` matching. Make one component own each key event: list navigation, active editor, popup, or modal, but never multiple layers at once.
**Phase:** Core TUI

### Single-line editors inheriting multiline and shortcut behavior from tui-textarea
**Warning:** `tui-textarea` is a full editor widget. By default it accepts Enter, multiline editing, undo or redo history, and Emacs-like shortcuts. That is useful for a real text editor but wrong for a one-line add or edit field in this TUI. Copying the default example code will let newline insertion or default shortcuts leak into what should be a constrained input control.
**Prevention:** Build add and edit on top of the crate's documented single-line pattern. Strip CR and LF from initial text, ignore Enter and Ctrl+M insertions, and prefer `input_without_shortcuts()` if default shortcuts conflict with app-level keybindings. Treat completion acceptance, save, and cancel as app commands, not textarea defaults.
**Phase:** Edit Mode

### Popup and modal ordering bugs
**Warning:** Inline editing plus autocomplete plus delete confirmation introduces three competing overlays. Rendering them in the wrong order or routing input without a strict ownership rule produces clipped popups, invisible suggestions, or confirmation dialogs that still let the editor consume keys underneath.
**Prevention:** Define a hard interaction stack: delete confirmation, then autocomplete popup, then editor, then base list. Render overlays in that order every frame. Route input only to the top active layer. Compute popup placement from the editor rect and cursor anchor, then clamp it to the current frame area so it never renders off-screen.
**Phase:** Edit Mode

### Resize handling based on cached dimensions
**Warning:** Crossterm documents `Event::Resize(columns, rows)` and notes that resize events can occur in batches. If the TUI caches layout measurements, popup placement, or sidebar widths across draws, narrow terminal resizes will produce negative widths, clipped status text, or stale overlay positions.
**Prevention:** Recompute layout from `frame.area()` on every draw. Treat resize as a redraw signal, not as a one-time recomputation of persistent geometry. Clamp all split widths and popup rectangles, and explicitly test very narrow widths where the filter sidebar must collapse or become an overlay.
**Phase:** Core TUI

### Width bugs from Unicode assumptions
**Warning:** todo.txt lines can contain Unicode, but terminal width is measured in display cells, not bytes or chars. Status summaries, autocomplete previews, and task previews that slice strings naively can misalign borders or place the cursor and popup incorrectly, especially after resize.
**Prevention:** Let ratatui handle rendering widths where possible instead of pre-padding strings manually. Avoid storing cursor X in raw byte offsets for popup placement. Recompute popup and preview layout from widget areas, and truncate through widget APIs rather than homegrown substring math.
**Phase:** Polish

### Config duplication between CLI and TUI drifting over time
**Warning:** Config loading currently lives in `crates/todotxt-cli/src/config.rs`, while only `resolve_config_path()` lives in core. If the TUI duplicates config structs and defaulting rules carelessly, the CLI and TUI can resolve the same `config.toml` but interpret it differently, or the TUI can auto-create defaults that surprise the existing CLI contract.
**Prevention:** Share the same config file and path resolution logic. Long term, move config schema types into shared code. Short term, duplicate only the serde shape and keep it byte-for-byte compatible with the CLI schema. Put TUI-only settings under a dedicated `[tui]` table instead of redefining top-level defaults like `todo_file` or `auto_creation_date`.
**Phase:** Foundation

### Over-testing terminal plumbing and under-testing state logic
**Warning:** Teams adding a TUI often try to integration-test live crossterm sessions and then give up because the tests are brittle. The real regressions in this workspace will not be terminal paint differences first; they will be bad selection reconciliation, bad reload deferral, and wrong index mapping after filtering and sorting.
**Prevention:** Unit-test the reducer and state transitions. Feed synthetic app events into `App::handle_event` and assert selection, mode, pending reload, and visible-row mapping. Use `ratatui::backend::TestBackend` only for render smoke tests and snapshots of stable widgets. Leave panic cleanup and real raw-mode behavior to a small amount of manual verification or very targeted integration tests.
**Phase:** Polish

### Ratatui 0.30 migration traps from older examples
**Warning:** Much community sample code still targets older ratatui APIs. In 0.30, imports and behavior changed in ways that break copy-paste implementations: the `terminal` module is private, `Frame::size()` became `Frame::area()`, some backend conversions moved to explicit traits, and `render_widget_ref` requires `FrameExt` plus the `unstable-widget-ref` feature. There are also renamed or changed layout and widget APIs compared with older examples.
**Prevention:** Start from 0.30 docs and examples only. Import `Terminal` and `Frame` from the crate root, use `frame.area()`, and avoid `unstable-widget-ref` unless it solves a real problem. If you see code importing from `ratatui::terminal`, matching old `WidgetRef` behavior, or using outdated layout names, treat it as migration debt, not as a template.
**Phase:** Foundation

## Minor (polish issues)

### Disabling default features without replacing what 0.30 now expects
**Warning:** Ratatui 0.30 modularized the crate and changed backend feature handling. A common workspace cleanup step is to set `default-features = false` everywhere, but doing that blindly can drop the selected crossterm backend or the layout cache and leave the TUI slower or unexpectedly misconfigured.
**Prevention:** Do not disable default features unless there is a concrete reason. If you do, re-enable the exact backend for crossterm 0.29 and any needed performance-related features intentionally. Verify the final manifest with a real build instead of assuming older ratatui recipes still map to 0.30.
**Phase:** Foundation

### Ignoring non-key events until they become user-visible bugs
**Warning:** The TUI can safely ignore many events early, but focus change, paste, and batched resize events are all real crossterm events. If they are silently treated like normal key input or not accounted for in the event layer, the app will behave oddly during paste or window switching and the bug will look random.
**Prevention:** Normalize event handling in one place. Explicitly match key, resize, paste, focus, and watcher events and decide what the app does for each. Even if the initial answer is "ignore focus" or "ignore paste," make that an intentional branch so later behavior is easy to add and test.
**Phase:** Polish

## Sources informing this note

- Workspace manifests and current code: root `Cargo.toml`, `todotxt-core` watcher and task list APIs, and CLI config loading
- Crossterm 0.29 docs: `Event`, `KeyEventKind`, and resize semantics on docs.rs
- Ratatui 0.30 highlights and breaking changes guidance on ratatui.rs
- tui-textarea 0.7 docs: installation, single-line usage, default shortcuts, and backend compatibility on docs.rs
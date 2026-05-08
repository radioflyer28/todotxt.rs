# Pitfalls Research — v1.6

**Researched:** 2026-05-04
**Scope:** Integration risks for SEED-005 through SEED-015 in the todotxt.net Rust TUI

---

## Summary

The three highest-risk areas in v1.6 are: (1) `$EDITOR` integration, where terminal raw-mode can be left in an unusable state if the editor crashes or the suspend/resume sequence is wrong; (2) view state persistence, where TOML forward-compatibility requires explicit `#[serde(default)]` on every field to avoid panics on older state files; and (3) the autocomplete extension to the filter bar, where a single shared `AutocompleteState` can silently write completions to the wrong editor if mode guards are missing. All three can produce data loss or terminal corruption — they need to be addressed explicitly in plans, not left to happy-path implementation.

---

## Pitfalls by Feature

### TUI Archive Hotkey (SEED-006)

| Pitfall | Risk Level | Prevention Strategy | Which Phase Should Address It |
|---------|-----------|---------------------|-------------------------------|
| `archive_path` not configured or not writable — the app opens done.txt in append mode without checking existence first; errors surface only after the task is already removed from todo.txt | **HIGH** | Check `done_file` is `Some(_)` before starting the operation; open the archive file first (before touching `task_list`), and surface the error with a status message if it fails. Never remove the task unless the archive write succeeds. | Archive Hotkey implementation phase |
| Archiving with an active filter — the hotkey is pressed when only a subset of completed tasks is visible; user intent is ambiguous (archive all-done vs only visible-done) | **MEDIUM** | Define the scope explicitly in the phase spec: "archive only tasks visible in the current filtered view." Document the choice in help text. Add a count to the status message ("Archived 3 tasks"). | Archive Hotkey implementation phase |
| Cursor/selection state invalid after archive — `selected` and `display_rows` shrink when tasks are removed, but `selected` is not re-clamped until `rebuild_display` runs; if the event loop renders before rebuild, `display_rows[selected]` panics on out-of-bounds | **HIGH** | Call `rebuild_display` immediately after the archive operation completes, before returning to the render pass. Then clamp `selected` to `display_rows.len().saturating_sub(1)`. Clear `selected_tasks` after archiving. | Archive Hotkey implementation phase |
| Undo of archive — `undo_entry` holds a single snapshot of `task_list`; restoring it after archive brings tasks back to todo.txt but does NOT remove them from done.txt, leaving duplicates | **MEDIUM** | Either exclude archive from undo entirely (document it as non-undoable with a confirmation prompt), or implement a two-phase undo that removes the appended lines from done.txt. The simpler safe approach: no undo for archive + confirmation dialog. | Archive Hotkey implementation phase |

---

### Bulk Mark-Done (SEED-009)

| Pitfall | Risk Level | Prevention Strategy | Which Phase Should Address It |
|---------|-----------|---------------------|-------------------------------|
| Mixed selection (some tasks already done, some not) — pressing `x` silently no-ops already-done tasks, so the user sees fewer changes than expected with no feedback | **MEDIUM** | Skip already-completed tasks silently but include them in a count: "Marked 4 done (2 already complete)." Never toggle — bulk `x` is always force-complete. | Bulk Mark-Done phase |
| Undo pushes one entry but restores the entire pre-bulk snapshot — this is correct for `undo_entry: Option<UndoEntry>` design, but the user must be informed it is an all-or-nothing undo | **MEDIUM** | Take the snapshot before any mutations. Log "Undo will restore all N tasks" in the status bar after the operation. Consistent with the existing `undo_entry` single-entry model. | Bulk Mark-Done phase |
| Index invalidation timing — `selected_tasks: HashSet<usize>` stores canonical task indices; after `task_list.complete()` runs for multiple indices the display is stale until `rebuild_display` runs; any intermediate render will use stale `display_rows` | **HIGH** | Batch all mutations (loop over sorted indices), then call `rebuild_display` once, then clear `selected_tasks`, then render. Never call render between individual completions. | Bulk Mark-Done phase |

---

### Open Task in $EDITOR (SEED-012)

| Pitfall | Risk Level | Prevention Strategy | Which Phase Should Address It |
|---------|-----------|---------------------|-------------------------------|
| Terminal left in raw mode if editor crashes or process exits non-zero — crossterm raw mode and the alternate screen are not automatically restored when the child process dies; the user's shell becomes unusable | **CRITICAL** | Before `Command::spawn()`, call `disable_raw_mode()` and `execute!(stdout, LeaveAlternateScreen)`. Wrap the entire spawn-wait-restore block in a `defer!`-style guard or explicit `finally` pattern using Rust's `Drop`. Restore raw mode and alternate screen even if the child exits with error. | Editor Integration phase |
| Editor does not fully restore terminal state on exit (e.g., `nvim` with certain plugins leaves `DECCKM` cursor key mode set) — after return, arrow keys send `[A`/`[B` literals instead of being handled by crossterm | **HIGH** | After the child exits, call `crossterm::terminal::enable_raw_mode()` AND re-execute `EnterAlternateScreen` AND reset cursor visibility. Additionally flush the terminal with `execute!(stdout, crossterm::terminal::Clear(ClearType::All))` to clear any residue from the editor. | Editor Integration phase |
| Windows `EDITOR` env var behavior — on Windows `EDITOR` is rarely set; `VISUAL` is also rarely set; `notepad.exe` does not block (`CreateProcess` returns immediately), so the "wait for editor exit" pattern breaks silently | **HIGH** | On Windows: check `VISUAL` then `EDITOR` then fall back to `notepad`. For `notepad`, detect that the process exits before file modification by comparing file mtime before and after; warn the user if no change is detected. Document that GUI editors on Windows require a blocking invocation (`notepad.exe /W` does not exist; use `start /WAIT notepad.exe` via `cmd /C`). | Editor Integration phase |
| File locking on Windows — writing the task to a temp file, opening it in an editor, then reading it back can fail if the editor holds an exclusive lock on close | **MEDIUM** | Use a named temp file in the same directory as todo.txt (not `%TEMP%`) to avoid cross-drive rename issues. Retry the read up to 3 times with a short delay before surfacing an error. | Editor Integration phase |
| Multi-line edits silently truncated — the editor may save multiple lines; `Task::new()` from `todotxt-core` expects a single line; reading back the file and picking only the first line is silently lossy | **MEDIUM** | After the editor exits, read the file, filter empty lines, take only the first non-empty line, and display a warning if more than one non-empty line was found. | Editor Integration phase |

---

### Fix `+` Project Autocomplete (SEED-013)

| Pitfall | Risk Level | Prevention Strategy | Which Phase Should Address It |
|---------|-----------|---------------------|-------------------------------|
| `t.projects` returns an empty `Vec` (or `Option`) for tasks without `+` tokens — iterating this in `get_existing_projects()` is safe, but if the parser representation changed from `Vec<String>` to `HashSet<String>` or introduced an `Option` wrapper, all call sites fail to compile without review | **MEDIUM** | Read the `todotxt_core::Task` struct definition before implementing. Verify `t.projects` type against actual source. Do not assume it matches the CLI's representation. | Autocomplete Fix phase |
| Trigger character collision — if `+` prefix scanning in the editor scans backward from cursor and the cursor is mid-word (e.g., `+proj|ect`), the prefix includes the full typed word; then `accept_completion()` inserts the full candidate, producing `+projectproject` | **HIGH** | Scan backward to find the trigger `+`, extract everything after it as the prefix, and when inserting the completion, delete exactly `prefix.len()` characters before inserting the candidate — not just append. Existing `@` autocomplete should be used as the canonical reference. | Autocomplete Fix phase |

---

### Autocomplete in Filter Input (SEED-014)

| Pitfall | Risk Level | Prevention Strategy | Which Phase Should Address It |
|---------|-----------|---------------------|-------------------------------|
| `accept_completion()` writes to the task editor when filter mode is also active — the current implementation dispatches to `self.editor` unconditionally; if `AppMode::Filtering` is active and uses a separate `TextArea`, the accepted token lands in the wrong buffer | **CRITICAL** | Add a mode guard at the top of `accept_completion()`: if `self.mode == AppMode::Filtering`, write to the filter input widget, not `self.editor`. Or refactor to pass the target `TextArea` as a parameter. Never share a single `accept_completion` path that dispatches to different widgets based on side-channel state. | Filter Autocomplete phase |
| Autocomplete popup consuming `Esc` that should close the filter panel — `Esc` in filter mode should close the filter and return to Normal; if the autocomplete popup intercepts `Esc` first to dismiss itself, the filter panel stays open and the user has to press `Esc` again | **MEDIUM** | Define key priority explicitly: `Esc` when autocomplete is open → close autocomplete only, do NOT close filter. `Esc` when autocomplete is closed and filter is open → close filter. Use a precedence chain in the event handler, not nested `if` blocks that both consume the event. | Filter Autocomplete phase |
| Popup staying open when filter mode exits — if the user presses `Enter` to apply the filter while `autocomplete` is `Some(_)`, the autocomplete `Option` is not cleared; on the next `Filtering` entry the stale popup reappears with old candidates | **MEDIUM** | Clear `self.autocomplete = None` in every code path that exits `AppMode::Filtering` (Enter, Esc, Ctrl+F toggle). Add an assertion in debug builds: `debug_assert!(self.autocomplete.is_none() || self.mode == AppMode::Filtering)`. | Filter Autocomplete phase |

---

### Filter Input History (SEED-011)

| Pitfall | Risk Level | Prevention Strategy | Which Phase Should Address It |
|---------|-----------|---------------------|-------------------------------|
| History ring buffer index wrapping — if the user presses Up past the oldest entry, the index wraps to the newest, giving the illusion of infinite history; this surprises users who expect the ring to stop at the oldest entry | **LOW** | Clamp at boundaries (oldest = stop, newest = stop at current input), not wrapping. Store the "current input" as a temporary entry at index 0 so Up/Down can navigate back to the typed-but-not-submitted query. | Filter History phase |
| Duplicate consecutive entries — submitting the same filter twice pushes duplicate entries, cluttering the history | **LOW** | Before push, check if the new entry equals the most recent entry; if so, skip. | Filter History phase |
| History not cleared with the filter — when the user clears the filter field with `Ctrl+U` or `Backspace`-to-empty, the history ring still contains the previous queries; this is correct behavior but should be tested explicitly | **LOW** | Explicitly test that history is navigable after the filter field is cleared, and that clearing the filter does not reset the history ring. | Filter History phase |

---

### Expand Numeric Presets (SEED-015)

| Pitfall | Risk Level | Prevention Strategy | Which Phase Should Address It |
|---------|-----------|---------------------|-------------------------------|
| `TuiPreset` struct gains new fields (`sort`, `group`) but existing config files only have `filter` — serde's `#[serde(default)]` is required on every new field, or existing configs fail to deserialize at startup | **HIGH** | Add `#[serde(default)]` to both new fields. Add a smoke test that deserializes an old-format preset string and confirms the new fields get their defaults. | Preset Expansion phase |
| Preset activation silently overrides user's current sort/group state — pressing F3 applies the preset's sort and group, discarding any runtime changes the user made in the session; this is surprising if the preset has `sort: None` (which should mean "don't change") vs `sort: FileOrder` (which should mean "reset to file order") | **MEDIUM** | Use `Option<PaneSort>` for the sort and group fields in `TuiPreset`: `None` = "preserve current", `Some(x)` = "set to x". Serialize `None` as absent/null in TOML. | Preset Expansion phase |

---

### View State Persistence (SEED-007)

| Pitfall | Risk Level | Prevention Strategy | Which Phase Should Address It |
|---------|-----------|---------------------|-------------------------------|
| New version writes fields that old version doesn't know — when the user downgrades (or runs a different build), serde's default behavior for unknown fields is `deny_unknown_fields`; if that derive is used, startup panics or refuses to load | **HIGH** | Never use `#[serde(deny_unknown_fields)]` on `TuiState`. Always use `#[serde(default)]` on every field. On any deserialization error, log a warning and start fresh — never hard-fail startup. | View State Persistence phase |
| Partial/corrupted state file — TOML parser returns an error on any syntax violation; if the app treats this as fatal, the user must manually delete the file to launch | **HIGH** | Wrap `toml::from_str` in a `match`; on error emit a `runtime_warnings` entry and continue with default state. Optionally rename the corrupted file to `tui-state.toml.bak` to aid debugging. | View State Persistence phase |
| Race condition: state written on clean exit while file watcher is active — `tui-state.toml` sits alongside `todo.txt`; if the watcher monitors the entire directory, the state file write triggers a `FileChanged` event just as the app is shutting down | **MEDIUM** | Either watch only `todo.txt` by filename (not the parent directory) when possible, or filter incoming `FileChanged` events by path, ignoring events for `tui-state.toml`. | View State Persistence phase |
| User manually edits state file — typos in enum variant names (e.g., `sort = "prioritty"`) produce serde errors; since the state file is user-visible, this is a real scenario | **LOW** | On serde error for any individual field, fall back to the field default rather than failing the entire file. Consider `serde_ignored` or manual deserialization for robustness, but at minimum the `#[serde(default)]` + whole-file fallback gives acceptable behavior. | View State Persistence phase |

---

### Decouple Group-By from Sort Order (SEED-008)

| Pitfall | Risk Level | Prevention Strategy | Which Phase Should Address It |
|---------|-----------|---------------------|-------------------------------|
| Breaking config change — if existing `PaneConfig` has `group: bool` and the new design adds a `group_by: GroupBy` enum, configs that have `group = true` would need to map to `GroupBy::ByPriority` (or whatever the default was). Serde can't do this mapping automatically | **HIGH** | Keep the existing `group: bool` field as a deprecated fallback (`#[serde(default)]`), add `group_by: Option<GroupBy>` as the new field. If `group_by` is absent but `group: true`, infer `GroupBy::ByPriority` as the legacy default. Document the migration path. | GroupBy Decoupling phase |
| `group_key_for` signature change — any rename or parameter change to `group_key_for` breaks all call sites including `rebuild_display`; the compiler will catch these, but the risk is silent logic change if the new variant arms have subtly wrong grouping logic | **MEDIUM** | Add exhaustiveness tests: a unit test that constructs one task of each priority/context/project/due-date combination and asserts the expected group key for each `GroupBy` variant. | GroupBy Decoupling phase |
| `G` key binding conflict — `G` (Shift+g) is conventionally "go to bottom" in vi-style lists; if it is used for group-by cycling it conflicts with expected navigation behavior in this codebase's existing vi-style keymap | **MEDIUM** | Check `effective_keymap` for `G` before assigning it. If already bound to `go_to_bottom`, use a different default (e.g., `Ctrl+G`). Make group-by cycling a configurable action in the keymap system rather than a hardcoded binding. | GroupBy Decoupling phase |

---

### Phase 22 Tests (SEED-005)

| Pitfall | Risk Level | Prevention Strategy | Which Phase Should Address It |
|---------|-----------|---------------------|-------------------------------|
| `App::new()` requires a real file path and performs file I/O — using a path that doesn't exist produces a `TaskList` error at construction; test isolation requires either a temp file or a test constructor that accepts a pre-loaded `TaskList` | **HIGH** | Add `App::new_for_test(task_list: TaskList, config: TuiConfig) -> App` that bypasses file-path resolution, or use `tempfile::NamedTempFile` to create a real empty file for each test. The latter is simpler and more realistic. | Test Infrastructure phase |
| Event dispatch tests depend on internal order of `effective_keymap` resolution — if two tests assume the same key triggers the same action but the config differs between test cases, tests can pass individually and fail when run in parallel with a shared config | **MEDIUM** | Build `effective_keymap` from an explicit `TuiConfig` in each test, not a global fixture. Use `App::new_for_test` with a test-specific config. Never share mutable `App` state between tests. | Test Infrastructure phase |
| `HashSet<usize>` non-determinism in `selected_tasks` — tests that iterate over `selected_tasks` and compare results may be order-sensitive; `HashSet` iteration order is unspecified | **LOW** | Collect to `Vec`, sort, then compare. Never assert on iteration order of `selected_tasks` directly. | Test Infrastructure phase |

---

## Cross-Cutting Risks

| Risk | Affects | Mitigation |
|------|---------|------------|
| Multiple features touch `rebuild_display` — concurrent changes to grouping (SEED-008), filter autocomplete (SEED-014), and archive (SEED-006) all trigger display rebuilds; merge conflicts in `app.rs` are likely | SEED-006, SEED-008, SEED-014 | Sequence these phases to avoid parallel edits to `rebuild_display`; or extract `rebuild_display` into a standalone method early and have all features call it |
| `undo_entry` is a single-level snapshot — SEED-009 (bulk mark-done) and SEED-006 (archive) both mutate many tasks in one operation; the single-entry design means undo of one replaces undo of the other | SEED-006, SEED-009 | Decide before implementation: is archive undoable? If yes, design a composite undo that covers done.txt. If no, document it clearly and add a confirmation step |
| `AppMode` enum has 13+ variants — every new `match` arm on `AppMode` that omits new variants compiles with `_` wildcards but silently skips the new mode; Filtering autocomplete (SEED-014) adds behavior conditional on `AppMode::Filtering` | SEED-014 | Remove `_ =>` wildcards from `AppMode` matches where behavior should be mode-specific; the compiler then forces exhaustive handling of every new variant |

---

## Sources

- Codebase analysis: `crates/todotxt-tui/src/app.rs`, `state.rs`, `config.rs`, `event.rs` — [VERIFIED: grep/read]
- Existing pitfalls: `.planning/research/PITFALLS.md` (v1.1–v1.3 research) — [CITED: local]
- Rust TUI raw-mode restore patterns: crossterm docs, ratatui template examples — [ASSUMED: training knowledge, crossterm 0.28+ API shape]
- Windows `EDITOR` env var behavior: known Win32 console limitation — [ASSUMED: training knowledge]
- Serde `deny_unknown_fields` gotcha: serde.rs documentation — [ASSUMED: training knowledge, well-established]

### 1. Row-based selection bugs

Current TUI navigation includes group headers and filtered display rows. If v1.3 stores selection by visible row offset, selections will drift after regrouping, re-sorting, filtering, or reload.

**Prevention:** store canonical task identity, derive visible highlights second.

### 2. Unsafe bulk deletes

Deleting selected tasks in ascending source-index order can invalidate later indices.

**Prevention:** collect canonical indices, sort descending for destructive operations.

### 3. Over-aggressive text rewriting

Smart normalization is valuable only if users trust it. If append/edit rewrites unrelated text or unknown metadata, the feature will feel destructive.

**Prevention:** normalize only recognized tokens (`(A)`, `+proj`, `@ctx`, `due:`, `t:` and any explicitly supported metadata). Preserve unknown text verbatim.

### 4. Diverging from todotxt.net without documenting it

The goal is familiarity. Intentional deviations are fine, but silent ones will look like regressions to switching users.

**Prevention:** explicitly document parity choices in requirements and help text.

### 5. Multi-select UX without clear affordances

If the user cannot tell which rows are selected versus merely focused, bulk actions become error-prone.

**Prevention:** distinct row styling, selection counts in status/help, and confirmation dialogs for destructive actions.

---
*Archived v1.1–v1.3 pitfalls retained below for historical reference.*

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
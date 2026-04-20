# Phase 11: Edit Mode — Context

**Gathered:** 2026-04-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 11 delivers full task editing from within the TUI: add new tasks, inline-edit existing tasks, delete with confirmation, `@context`/`+project` autocomplete, and deferred reload while editing.

The user can:
- Press `n` to open a blank input field and add a new task (`Enter` saves, `Esc` cancels)
- Press `u` or `e` to open the selected task for inline editing (pre-populated with its raw text)
- Press `d` to delete the selected task (confirmation prompt shown, `y` confirms)
- See `@context` and `+project` autocomplete popups while typing in add/edit mode
- Trust that a file-change arriving during editing is quietly queued and applied after save/cancel

Phase 10 owns navigation and done/undo (complete).
Phase 12 owns filter and sort UI.
Phase 13 owns theming and colors.

</domain>

<decisions>
## Implementation Decisions

### Mode State

- **D-01: AppMode enum** — Extend `App` with a `mode: AppMode` field:
  ```rust
  enum AppMode {
      Normal,
      Adding,
      Editing { original_idx: usize },
      DeleteConfirm,
  }
  ```
  All key event handling dispatches on `self.mode` first.

### Input Surface

- **D-02: Footer swap — status bar becomes input field in Add/Edit mode** — The layout retains `Layout::vertical([Constraint::Min(0), Constraint::Length(1)])` in both Normal and Add/Edit modes. In Normal/DeleteConfirm mode, the bottom row renders the status bar. In Adding/Editing mode, the bottom row renders a `tui-textarea` single-line editor. The status bar is NOT visible during text input.

- **D-03: tui-textarea as input widget** — As noted in PITFALLS.md, use `tui-textarea` for the input field. Apply single-line pattern, strip CR/LF from output, and call `input_without_shortcuts()` to avoid conflicting default bindings. `tui-textarea` must be added to `crates/todotxt-tui/Cargo.toml`.

- **D-04: Pre-populate editor for edit mode** — When entering Editing mode, set the textarea content to the selected task's raw text. When entering Adding mode, the textarea starts empty.

### Keybindings (user-selected: legacy-first)

- **D-05: Keybindings (deviation from requirements)** — User chose legacy-first keybinding over the `a`/`e` scheme in REQUIREMENTS.md (TUI-ACT-03, TUI-ACT-04). Locked decisions:
  - `n` = add new task (primary) — **note:** TUI-ACT-03 specifies `a`; user overrides to `n`
  - `u` = edit selected task (primary) — matches Phase 10 D-12 override (commit `2970e0c`)
  - `e` = edit selected task (alias) — TUI-ACT-04 specifies `e`; satisfied as alias
  - `d` = delete selected task with confirmation (TUI-ACT-05)
  - `Enter` = save (in Add/Edit mode) — per all action requirements
  - `Esc` = cancel (in Add/Edit/DeleteConfirm mode)
  - `y` = confirm delete (in DeleteConfirm mode)
  - Any other key in DeleteConfirm = cancel (no-op, return to Normal)

### Delete Confirmation Layout

- **D-06: Extra row above status bar in DeleteConfirm mode** — During `DeleteConfirm`, layout expands to `Layout::vertical([Constraint::Min(0), Constraint::Length(1), Constraint::Length(1)])`: task list + confirm panel + status bar. The confirm panel shows: `Delete: "<task raw text>"  y=confirm  any=cancel`. After confirmation or cancellation, layout returns to `[Min(0), Length(1)]`.

- **D-07: Delete confirmation keys** — `y` (lowercase only) confirms. Any other key cancels and returns to Normal. `Esc` also cancels. No `Y` uppercase matching (keep it simple; Esc/any-key is the cancel).

### Autocomplete

- **D-08: Full legacy popup behavior** — Typing `@` or `+` in add/edit mode triggers an autocomplete popup list:
  - Source data: all unique `@context` tokens and `+project` tokens already present in `task_list.tasks()`
  - Filtered: popup shows only tokens matching the current prefix (case-insensitive)
  - Popup position: floating widget above/beside the input row (agent's discretion on exact placement)
  - `Down` arrow moves focus into the popup list
  - `Tab`, `Enter`, or `Space` while popup is focused → insert the selected completion into the input at cursor, close popup
  - `Esc` closes popup without inserting (returns focus to input)
  - If no matching tokens exist, popup does not appear

- **D-09: Popup state on App** — Add `autocomplete: Option<AutocompleteState>` to `App`. `AutocompleteState` holds the trigger character (`@` or `+`), current prefix, filtered list, and selected index. When `autocomplete` is `Some`, the popup is rendered. When `None`, no popup.

### Reload Guard (TUI-UX-03)

- **D-10: pending_reload flag — silent queue** — Add `pending_reload: bool` to `App`. When `AppEvent::FileChanged` arrives while `mode != AppMode::Normal`, set `pending_reload = true` instead of reloading. On any Normal-mode entry (after save or cancel), check `pending_reload`; if true, call `task_list.reload()` and reset the flag. No user-visible indicator is shown.

- **D-11: Own-write suppression** — Phase 10 D-13 allowed own-write `FileChanged` to trigger a harmless reload. In Phase 11, the `pending_reload` guard also covers own-write in Add/Edit mode — this is safe: the task list reloads after save, which re-reads the file the save just wrote. No data loss.

### Saving and Cancelling

- **D-12: Save path — append for add, update for edit** — On Enter in Adding mode: call `task_list.append(new_task)` + `task_list.save()`. On Enter in Editing mode: call `task_list.update(original_idx, edited_task)` + `task_list.save()`. Parse raw text into a `Task` using the existing core parser.

- **D-13: Selection after add** — After a successful add, move `selected` to the index of the newly added task (last in list). After edit, keep `selected` at the same `original_idx`. After delete, clamp `selected` to `[0, task_count - 1]`.

### Agent's Discretion

- Whether `AutocompleteState` is a separate struct or inline on `App`
- Exact popup rendering geometry (above the footer, beside cursor, etc.)
- Whether autocomplete token collection is a method on `TaskList` or computed inline in `App`
- How to split Phase 11 into plans (e.g., Plan 1: modes + add/edit/delete; Plan 2: autocomplete + reload guard)
- Whether `tui-textarea` version is pinned or inherited from workspace — agent checks compatibility with ratatui 0.30

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase 10 Foundation (existing code to extend)

- `crates/todotxt-tui/src/app.rs` — Current `App` struct, event loop, and draw() to extend
- `crates/todotxt-tui/src/event.rs` — `AppEvent` enum
- `crates/todotxt-tui/src/tui.rs` — `Tui` type alias
- `crates/todotxt-tui/Cargo.toml` — Add `tui-textarea` dependency here

### Core Library APIs

- `crates/todotxt-core/src/task_list.rs` — `append()`, `update()`, `delete()`, `save()`, `tasks()` signatures — verify `append` and `delete` exist; add if missing
- `crates/todotxt-core/src/task.rs` — `Task` struct fields, parse-from-string (verify parser API)
- `crates/todotxt-core/src/lib.rs` — Public exports

### Prior Decisions In Force

- `.planning/phases/09-tui-foundation/09-CONTEXT.md` — D-01..D-11 from Phase 9 (sync only, no tokio, `#![deny(warnings)]`, etc.)
- `.planning/phases/10-core-tui/10-CONTEXT.md` — D-01..D-18 from Phase 10 (layout, ListState, selection clamping, status bar, `frame.area()` API)

### Requirements

- `.planning/REQUIREMENTS.md` — TUI-ACT-03, TUI-ACT-04, TUI-ACT-05, TUI-ACT-06, TUI-UX-03

### Research

- `.planning/research/PITFALLS.md` — tui-textarea pitfalls: single-line pattern, strip CR/LF, `input_without_shortcuts()`

</canonical_refs>

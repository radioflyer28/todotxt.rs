# Phase 36: Recovery Path (Short-Horizon Undo) — Context

**Gathered:** 2026-04-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 36 delivers a single-level undo for all CRUD operations on the task list:

- Undo triggered by Ctrl+Z in Normal mode
- All task-mutating actions push an undo entry: create, edit, delete, property overwrite, paste, bulk append, complete/toggle, quick tag setters
- Undo restores the full task list to its state before the last action, plus best-effort cursor position restore
- Silent no-op when history is empty (no message)
- No feedback on successful undo — the visual change in the task list is sufficient

In scope (UNDO-01/02/03):
- UNDO-01: Short-horizon undo for all destructive/high-impact actions
- UNDO-02: Undo restores task content + best-effort cursor position
- UNDO-03: No-op behavior when history is empty (silent); no feedback message on successful undo

Out of scope for this phase:
- Multi-level undo (only the most recent action is undoable)
- Redo support
- Clipboard undo (arboard state is not tracked)
- Undo across file-watcher-driven external changes
- Feedback messages for undo events (intentionally excluded — visual change is sufficient)

</domain>

<decisions>
## Implementation Decisions

### Undo Trigger

- **D-01:** Ctrl+Z triggers undo in Normal mode. `u` (plain) is already bound to edit; Ctrl+U is half-page scroll. Ctrl+Z is the universal undo convention, has zero conflicts in the existing keymap, and is immediately discoverable.

### Undo Depth

- **D-02:** Single-level undo only — depth 1. Stores the previous state as `Option<UndoEntry>` on the `App` struct. Each new undoable action overwrites the prior entry. Multi-step undo is out of scope for this phase.

### Which Actions Are Undoable (UNDO-01)

- **D-03:** Every action that creates, edits, or deletes task data pushes an undo entry before executing:
  - **Create:** new task via `n` + Enter/save
  - **Edit:** save existing task via `u` + Enter/save
  - **Delete:** single delete (`d`) and bulk delete (`D`)
  - **Property overwrite:** due-date setter (`s`) and priority setter (`i`)
  - **Create:** paste from clipboard (`p` in Normal mode)
  - **Modify:** bulk append text (`T`)
  - **Complete/toggle:** mark complete or toggle completion (`x`/`c` or equivalent)
  - **Tag setters:** quick `@` context setter and `+` project setter

### Storage Model

- **D-04:** Before each undoable action, snapshot the full task list content and the current cursor position (`selected` index) as an `UndoEntry` struct. Store as `Option<UndoEntry>` on `App`. Restore by replacing `task_list` content from the snapshot, restoring the `selected` cursor, then calling `rebuild_all_panes()` and `rebuild_and_reanchor()`.
- **D-05:** `UndoEntry` fields: `tasks: Vec<RawTask>` (or the serialized form of all tasks), `selected: usize`. The snapshot must be independent of the live task list — a deep clone of the task data at the moment the action is initiated.
- **D-06:** Multi-pane selection state (the `selected_tasks` HashSet) is **not** restored — only the primary cursor position (`selected`) is restored, consistent with "best-effort" semantics of UNDO-02.

### Ctrl+Z Binding

- **D-07:** Ctrl+Z is processed in the main `handle_key` / `press_key` dispatch for Normal mode. It must be placed before the `KeyCode::Char('z')` plain arm (if any exists) and checked with `key.modifiers.contains(KeyModifiers::CONTROL)`.
- **D-08:** Ctrl+Z is a no-op when `self.undo_entry.is_none()`. No message, no visual change.

### Feedback (UNDO-03)

- **D-09:** No status message on successful undo. The visual result (task list restoring to prior state) is the feedback. This avoids polluting the runtime warning / error channel with routine operational events.
- **D-10:** No message when history is empty. Silent no-op only.

### Agent's Discretion

- Exact field type for stored task snapshots (clone of `TaskList`, or `Vec<String>` of raw lines, or `Vec<Task>`)
- Whether `UndoEntry` is a named struct or an inline tuple
- Exact placement of the `undo_entry` field in the `App` struct
- Whether to call `task_list.save()` after restoring (required to persist undo to disk, consistent with how all other mutations are handled)
- Whether to push a new undo entry when the user undoes (i.e., enable redo) — this is out of scope but the agent may document the decision not to implement it

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase and Scope Authority

- `.planning/ROADMAP.md` — Phase 36 scope, requirement mapping (UNDO-01, UNDO-02, UNDO-03)
- `.planning/REQUIREMENTS.md` — UNDO-01 through UNDO-03 full text
- `.planning/phases/36-recovery-path-short-horizon-undo/36-CONTEXT.md` — this file

### Primary Implementation Target

- `crates/todotxt-tui/src/app.rs` — `App` struct (add `undo_entry` field), `handle_key` / Normal mode dispatch (add Ctrl+Z arm), all undoable action sites (push undo entry before each)
- `crates/todotxt-tui/src/state.rs` — define `UndoEntry` struct

### Existing Delete and Mutation Patterns (reference before implementing)

- `app.rs` — `handle_delete_confirm_key()` (bulk delete path, D), single delete dispatch (`d`), `handle_date_picker_key()` (s overwrite path), `handle_priority_picker_key()` (i overwrite path)
- `app.rs` — `handle_append_text_key()` (T bulk append), `copy_selected_to_clipboard()` / `paste_from_clipboard()` (Phase 35 clipboard)
- `app.rs` — `handle_editor_key()` / `AppMode::Adding` / `AppMode::Editing` (task create and edit save paths)

### Prior Phase Contracts (must not regress)

- `.planning/phases/34-bulk-action-safety-metadata-preservation/34-CONTEXT.md` — D-13/D-14: structured Task mutation for s/i setters
- `.planning/phases/35-basic-clipboard-workflows/35-CONTEXT.md` — D-11/D-14: paste creates tasks via same code path as add; rebuild_all_panes call pattern
- `.planning/phases/20-bulk-actions-selection-ux/20-CONTEXT.md` — D-01 through D-14: bulk delete/append patterns, descending-index rule
</canonical_refs>

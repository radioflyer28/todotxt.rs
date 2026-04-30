# Phase 35: Basic Clipboard Workflows — Context

**Gathered:** 2026-04-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 35 delivers clipboard cut/copy/paste workflows:

- Copy selected task text to system clipboard (`y` in Normal mode)
- Cut = copy + delete: press `y` to copy, then use existing delete (`d`/`D`) to remove tasks
- Paste clipboard content as new task entries (`p` in Normal mode — appended to end of file)
- Paste clipboard content into the new-task editor (Ctrl+V in Adding mode via `n`)

In scope:
- CLIP-01: Copy active/selected task(s) raw text to system clipboard
- CLIP-02: Cut = y to copy, then d/D to delete (two-step, no new cut mode needed)
- CLIP-03: p in Normal mode creates new task entries from clipboard lines (all lines at once, appended to end)
- CLIP-04: Ctrl+V in Adding mode pre-fills the editor from system clipboard
- Adding `arboard` crate as clipboard backend

Out of scope:
- Dedicated cut AppMode or atomic cut-with-confirm state
- Insert-below-cursor paste positioning
- Undo integration (Phase 36)
- Workspace-specific clipboard behavior

</domain>

<decisions>
## Implementation Decisions

### Clipboard Backend

- **D-01:** Use the `arboard` crate for system clipboard access. Tasks copied/cut are accessible system-wide (cross-app paste works).
- **D-02:** No in-process fallback buffer — arboard is the sole clipboard store. If arboard fails to initialize (e.g., headless environment), surface a runtime warning and no-op gracefully.

### Keybindings

- **D-03:** `y` in Normal mode = copy selected/active task(s) to system clipboard. Targets `selected_tasks` when non-empty, otherwise active cursor task — same semantics as `s` and `i`.
- **D-04:** Cut is a user workflow: `y` (copy) followed by `d`/`D` (existing delete). No new cut AppMode. No special coupling between y and d — they are independent actions. Users achieve cut by composing copy + delete.
- **D-05:** `p` in Normal mode = paste clipboard content as new tasks (appended to end of file). All clipboard lines pasted in a single operation (not one-at-a-time).
- **D-06:** Ctrl+V in Adding mode (AppMode::Adding via `n`) pre-fills the editor from system clipboard. This integrates into `handle_editor_key` — intercept Ctrl+V before passing to tui-textarea, fetch clipboard text, and insert into editor. Only first clipboard line used if clipboard has multiple lines (the editor is single-line).

### Copy Behavior (CLIP-01)

- **D-07:** Copy captures raw todo.txt text for each targeted task via `task.to_raw()`.
- **D-08:** For multi-task copy, lines are joined with newlines (`\n`) in descending-visual-order (consistent with Phase 33/34 bulk mutation ordering convention, descending canonical index).
- **D-09:** After a successful copy, show a brief status message via `push_runtime_warning` indicating how many tasks were copied (e.g., "copied 3 tasks").
- **D-10:** Copy is available on header rows too if somehow targeted — but guard: only `DisplayRow::Task` entries contribute text. Group headers are skipped silently.

### Paste Behavior (CLIP-03 / CLIP-04)

- **D-11:** `p` in Normal mode reads clipboard text, splits on newlines, filters empty lines, and adds each non-empty line as a new task appended to the end of todo.txt. Same code path as `n`-then-Enter for each line.
- **D-12:** Pasted task lines are treated as raw todo.txt text. No transformation applied — user is responsible for valid format.
- **D-13:** If clipboard is empty or arboard fails to read, `p` is a no-op with a brief status hint: "clipboard is empty".
- **D-14:** After paste, `rebuild_all_panes()` and `rebuild_and_reanchor()` run as with any task addition.
- **D-15:** Ctrl+V in Adding mode: fetch clipboard text, strip to first line only (since editor is single-line), insert via `editor.insert_str()`. If clipboard is empty, no-op silently.

### Cut Confirmation (CLIP-02)

- **D-16:** No dedicated cut confirmation step. The `d`/`D` delete after `y` follows existing delete/bulk-delete confirmation rules (DeleteConfirm for multi-select, single-task immediate delete). No new behavior needed.
- **D-17:** The clipboard state persists independently of what the user does after copying — copying does not lock or track the tasks. Delete is a separate unrelated action.

### Agent's Discretion

- Exact arboard initialization strategy (lazy vs. at App::new) — lazy init on first use avoids startup errors in headless test environments
- Whether to add a `clipboard: Option<arboard::Clipboard>` field to `App` or keep it as a local in each clipboard handler
- Status message wording for copy/paste feedback
- Whether Ctrl+V in Adding mode replaces the full editor contents or inserts at cursor position

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase and Scope Authority
- `.planning/ROADMAP.md` — Phase 35 scope and requirement mapping
- `.planning/REQUIREMENTS.md` — CLIP-01, CLIP-02, CLIP-03, CLIP-04

### Existing App Architecture
- `crates/todotxt-tui/src/app.rs` — `AppMode` enum, `handle_normal_key`, `handle_editor_key`, `App` struct, `push_runtime_warning`, `selected_tasks`, `quick_setter_targets`, `rebuild_all_panes`, `rebuild_and_reanchor`
- `crates/todotxt-tui/Cargo.toml` — add `arboard` dependency here

### Existing Bulk Action Patterns (Phase 33/34 precedents)
- `.planning/phases/34-bulk-action-safety-metadata-preservation/34-CONTEXT.md` — D-10/D-11 (count preview), D-17 (descending-index ordering)
- `.planning/phases/33-fast-capture-property-pickers/33-CONTEXT.md` — D-01 (targeting semantics: selected_tasks vs. active cursor)

### tui-textarea Reference
- `crates/todotxt-tui/src/app.rs` — `handle_editor_key` (how editor input is processed; Ctrl+V must be intercepted here before the `.input(key)` passthrough)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `App::push_runtime_warning()` — status/error message display used by all bulk actions; reuse for copy/paste feedback
- `App::quick_setter_targets()` — returns canonical indices for selected_tasks or active cursor; same targeting logic needed for copy
- `App::rebuild_all_panes()` + `App::rebuild_and_reanchor()` — call after paste adds tasks, same as other task mutations
- `task.to_raw()` — returns raw todo.txt line; use for clipboard text extraction

### Established Patterns
- Descending-index ordering for bulk mutations (Phase 33/34 invariant) — apply when collecting multi-task copy text
- `AppMode::Adding` + `self.editor = TextArea::default()` then `self.mode = AppMode::Adding` — how `n` is entered; paste in Adding mode hooks into this same editor
- `handle_editor_key` routes all editor keypresses — Ctrl+V intercept goes here before `self.editor.input(key)` fallthrough

### Integration Points
- `handle_normal_key`: add `y` arm for copy, `p` arm for paste (alongside existing `s`, `i`, `@`, `+` arms)
- `handle_editor_key`: add `Ctrl+V` arm before the `_` fallthrough to intercept paste into editor
- `App` struct: add `clipboard: Option<arboard::Clipboard>` field (or lazy-init per call)
- `Cargo.toml` for `todotxt-tui`: add `arboard = "..."` dependency

</code_context>

<specifics>
## Specific Ideas

- "Cut" is intentionally decomposed into two separate actions (y + d) rather than a new mode — keeps the state machine simple and reuses the existing delete confirmation rules.
- Ctrl+V in Adding mode for CLIP-04 — `tui-textarea` may or may not handle this natively; implementation should explicitly intercept to ensure arboard is the source (not terminal bracketed paste).
- Copy feedback should be visible but non-intrusive — the existing `push_runtime_warning` pattern fits (shows in status bar, disappears on next action).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 35-basic-clipboard-workflows*
*Context gathered: 2026-04-30*

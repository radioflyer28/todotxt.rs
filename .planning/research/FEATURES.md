# Features Research — v1.1 TUI Interface

**Project:** todotxt.net Rust TUI (ratatui-based)
**Researched:** 2026-04-18
**Confidence:** HIGH (pattern analysis from taskwarrior-tui, gitui, lazygit, ratatui examples, and ratatui docs)

---

## Table Stakes

Must-have for a usable TUI. Missing any of these = product feels broken.

---

### 1. Task List Navigation and Selection

**Expected behavior:**
- Full-screen list occupies the main area (below header/title, above status bar).
- Exactly one task is highlighted at all times (cursor never disappears).
- The cursor row uses a visually distinct highlight (reverse video, bold, or accent color) so it is unambiguous.
- The list scrolls to keep the cursor visible. Use ratatui `List::scroll_padding(2)` so 2 items above/below the cursor remain visible when scrolling — prevents the "jump to edge and lose context" effect.
- Items are displayed with their task ID (line number in todo.txt), priority badge, due date, and full text. Priority `(A)`–`(Z)` should be colored by priority level.
- Completed tasks are rendered with strikethrough or dim style to visually differentiate.

**Keyboard shortcuts (vim-like as default, arrows as fallback):**

| Key | Action |
|-----|--------|
| `j` / `↓` | Move cursor down one item |
| `k` / `↑` | Move cursor up one item |
| `g` / `Home` | Jump to first item |
| `G` / `End` | Jump to last item |
| `Ctrl+d` / `PgDn` | Scroll down half-page |
| `Ctrl+u` / `PgUp` | Scroll up half-page |

**Edge cases:**
- **Empty list:** Show a centered placeholder message ("No tasks. Press `a` to add one."). No cursor. `j`/`k` are no-ops.
- **Single item:** Cursor stays on the single item. Navigation is no-op.
- **Long task text:** Truncate with `…` at terminal width − 2 columns. Do NOT wrap task text in the list view (wrapping breaks alignment; save wrapping for a detail pane).
- **Terminal resize:** On `SIGWINCH` / `crossterm::event::Event::Resize`, re-render immediately. `ListState` offset auto-adjusts via ratatui. Test: if cursor was at item 40 and terminal shrinks, cursor must remain visible.
- **Task IDs after filtering:** Show the *original* todo.txt line number (task ID), not the filtered list index. This matches CLI behavior and prevents confusion ("task 7" in TUI must match `todotxt do 7`).

---

### 2. Mark Done / Undo

**Expected behavior:**
- Press `x` (or `Space`) on a selected task to toggle completion.
- If task is pending: mark done immediately (no confirmation). The task's display updates in-place with strikethrough/dim styling. Task remains in the visible list (does not vanish unless the current filter hides done tasks).
- If task is already done: undo — strip `x DATE` prefix, restore to pending.
- The cursor stays on the same item after toggling (does not jump to next item).
- The file is written atomically via `todotxt-core::TaskList`. The in-memory state updates before the write — instant visual feedback even on slow filesystems.

**Visual feedback:**
- A brief status bar flash: "Task 7 marked done" or "Task 7 restored" for ~1.5 seconds, then returns to normal status bar content.
- The task row immediately reflects the new state (strikethrough for done, normal for restored). Do NOT wait for the file-watcher reload to update the display — update in-memory state first, then write.
- If the active filter hides completed tasks (e.g., `-DONE` is the default filter), the task disappears from the list after marking done. The cursor moves to the next item (or previous if it was the last).

**Keyboard shortcuts:**

| Key | Action |
|-----|--------|
| `x` | Toggle done/pending on selected task |
| `Space` | Alias for `x` (optional, ergonomic) |

**Edge cases:**
- **Marking done on an already-done task:** Undo it — this is the expected toggle behavior, not an error.
- **File write fails:** Show error in status bar. Keep the in-memory state at the pre-toggle value. Do NOT show the updated state if the write failed.
- **Multiple tasks selected (future):** Bulk toggle — out of scope for v1.1, but the key binding must not conflict.

---

### 3. Add New Task

**Expected behavior:**
- Press `a` to enter **Add mode**: a single-line input bar appears at the bottom of the screen (above the status bar), replacing it temporarily. The label "Add: " prefixes the input area.
- The user types the full todo.txt task text (e.g., `(A) Call dentist +Health @phone due:2026-04-25`).
- `tui-textarea` in single-line mode handles the input widget. Provides cursor movement (`←`/`→`), `Ctrl+A`/`Ctrl+E` (line start/end), `Ctrl+W` (delete word), backspace.
- On `Enter`: validate (non-empty), prepend today's creation date if the config option `add_creation_date = true`, write via `todotxt-core::TaskList::add()`, exit add mode. Cursor moves to the newly added task in the list.
- On `Esc`: cancel without saving. Restore previous status bar.

**Autocomplete for `@` and `+`:**
- When the user types `@` or `+`, a small popup `List` widget appears above the input bar (centered or aligned to the cursor position).
- The popup shows existing `@context` or `+project` values from `todotxt-core`'s tag index, filtered by what follows the trigger character.
- Navigation within the popup: `↑`/`↓` or `Tab`/`Shift+Tab`. `Enter` or `Tab` to accept, `Esc` to dismiss without accepting.
- Popup disappears if the user types a space or backspaces past the trigger character.
- **Implementation note:** This is a ~50-line custom popup — no external autocomplete crate needed. Render as a `ratatui::widgets::List` floated over the main layout using `ratatui::layout::Rect` offsets from the cursor position.

**Keyboard shortcuts:**

| Key | Context | Action |
|-----|---------|--------|
| `a` | Normal mode | Enter add mode |
| `Enter` | Add mode | Confirm and save |
| `Esc` | Add mode | Cancel |
| `@` | Add mode, mid-input | Trigger context autocomplete |
| `+` | Add mode, mid-input | Trigger project autocomplete |
| `Tab` / `↑↓` | Autocomplete popup | Navigate suggestions |
| `Tab` / `Enter` | Autocomplete popup | Accept suggestion |
| `Esc` | Autocomplete popup | Dismiss popup, return to input |

**Edge cases:**
- **Empty input on Enter:** Ignore — do not add an empty task. Flash status bar: "Task text cannot be empty."
- **Autocomplete with no existing tags:** Do not show the popup if the tag index is empty for that trigger character.
- **Autocomplete filtering:** If user types `@ph`, show only contexts starting with `ph` (case-insensitive prefix match). If user types `@` followed by a space, dismiss popup.
- **Pasting multi-line text:** `tui-textarea` in single-line mode strips newlines. The entire pasted text up to the first newline is inserted.
- **Long input exceeding terminal width:** `tui-textarea` handles horizontal scrolling within the input field automatically.

---

### 4. Inline Edit

**Expected behavior:**
- Press `e` on a selected task to enter **Edit mode**.
- The selected list row is replaced in-place by a `tui-textarea` input widget, pre-populated with the task's raw text (the full `raw` string, not a pretty-printed version — preserving all todo.txt fields exactly).
- The user can edit the text. All `tui-textarea` shortcuts apply: `←`/`→`, `Ctrl+A`/`Ctrl+E`, `Ctrl+W`, etc.
- On `Enter`: validate (non-empty), write the updated task via `todotxt-core::TaskList::update()`, exit edit mode. The list row refreshes with the updated content.
- On `Esc`: cancel, restore the original task text. No write.
- Autocomplete for `@` and `+` works identically to Add mode.

**Keyboard shortcuts:**

| Key | Context | Action |
|-----|---------|--------|
| `e` | Normal mode, task selected | Enter edit mode for selected task |
| `Enter` | Edit mode | Confirm edit and save |
| `Esc` | Edit mode | Cancel, restore original text |

**Edge cases:**
- **Edit during file-watch reload:** If a file change arrives while in edit mode, do NOT apply the reload (it would clobber the edit in progress). Queue the reload; apply after the user exits edit mode (confirm or cancel).
- **Priority in edited text:** If the user removes `(A)` from the text and saves, the task loses priority. This is correct — edit is a raw-text replace.
- **Empty text on Enter:** Do not save. Flash: "Task text cannot be empty."
- **Editing a completed task:** Allowed. The user can edit the raw text including the `x DATE` prefix if they want.

---

### 5. Delete with Confirmation

**Expected behavior:**
- Press `d` to initiate delete on the selected task.
- A centered modal overlay appears: `"Delete task N? [y/N]"` with the task text shown for context (truncated to one line if needed).
- The modal is a `ratatui::widgets::Paragraph` inside a `ratatui::widgets::Block` with a border, rendered over the main list using `ratatui::widgets::Clear` to erase the background area first.
- Press `y` or `Y`: delete the task via `todotxt-core::TaskList::remove()`, dismiss modal. Cursor moves to the next item (or previous if it was the last).
- Press `n`, `N`, `Esc`, or any other key: dismiss modal without deleting. Return to normal mode.
- **No timeout** — the modal stays until the user responds.

**Keyboard shortcuts:**

| Key | Context | Action |
|-----|---------|--------|
| `d` | Normal mode, task selected | Show delete confirmation modal |
| `y` / `Y` | Confirmation modal | Confirm deletion |
| `n` / `N` / `Esc` | Confirmation modal | Cancel deletion |

**Edge cases:**
- **Pressing `d` on empty list:** No-op.
- **File write fails on delete:** Show error in status bar; task remains in list.
- **Deleting the only task:** After deletion, list is empty; show the empty placeholder.
- **Confirmation modal rendering:** Must render above ANY other overlay (filter panel, autocomplete). Render last in the frame.

---

### 6. Filter Panel

**Expected behavior:**
- Press `f` to toggle the filter panel open/closed.
- Appears as a fixed-width sidebar on the right side (recommended: 28–32 columns, ~30% of terminal width). The task list area shrinks proportionally when the panel is open.
- The filter panel contains:
  1. **Text search** — a single-line `tui-textarea` input. Live-filters as the user types.
  2. **Contexts** — scrollable checklist of all `@context` values. Toggle with `Space`.
  3. **Projects** — scrollable checklist of all `+project` values. Toggle with `Space`.
  4. **Due date** — radio-style selector: `All` / `Today` / `Overdue` / `Active` / `Future`.
  5. **Show done** — checkbox toggle.
- All filters are ANDed. Task list updates live — no "apply" button.
- `Tab` / `Shift+Tab` cycles focus between the five sections.
- `j`/`k` navigates within the focused context or project list.

**Active filter indicator:**
- When any non-default filter is active, the status bar shows a compact indicator: `[filtered: @work +Project due:today]` so the user knows not all tasks are visible.

**Keyboard shortcuts:**

| Key | Context | Action |
|-----|---------|--------|
| `f` | Normal mode | Toggle filter panel open/closed |
| `Esc` | Filter panel focused | Close filter panel, return focus to task list |
| `Tab` / `Shift+Tab` | Filter panel | Cycle focus between filter sections |
| `Space` | Context/project list in panel | Toggle selected filter checkbox |
| `j`/`k` | Context/project list in panel | Navigate items |
| `Ctrl+R` | Anywhere | Clear all filters, reset to defaults |

**Edge cases:**
- **No contexts or projects exist:** Show section with placeholder ("No contexts found").
- **Filter results in zero tasks:** Show empty list with message "No tasks match the current filter." Panel remains operable.
- **Terminal width < 60 columns:** Render the filter panel as a full-screen overlay instead of a sidebar to remain usable.
- **Task IDs with active filter:** List always shows original todo.txt line numbers, not filtered sequence numbers.

---

### 7. Sort Toggle

**Expected behavior:**
- Press `s` to cycle forward through sort modes. The active sort name is always visible in the status bar.
- Sort cycle (press `s` to advance):
  1. `Priority` — `(A)` before `(B)` before no-priority (most common default)
  2. `Due date` — earliest first; no-due-date last
  3. `File order` — original insertion order (line number)
  4. `Alphabetical` — task text, case-insensitive
  5. `Project` — alphabetical by first `+tag`
  6. `Context` — alphabetical by first `@tag`
- Sort does not clear active filters.
- After a sort change, the cursor follows the previously selected task by task ID to prevent disorienting jumps.
- Sort mode is session-only (not persisted to config in v1.1).

**Keyboard shortcuts:**

| Key | Action |
|-----|--------|
| `s` | Cycle sort mode forward |
| `S` | Cycle sort mode backward |

**Edge cases:**
- **Tasks with equal sort keys:** Stable sort — preserve relative file order.
- **Cursor follow after sort:** If the previously selected task ID is no longer present (e.g., was deleted while the panel was open), move cursor to position 0.

---

### 8. Status Bar

**Expected behavior:**
- Permanent single line at the very bottom of the screen. Never hidden.
- Default content layout:

```
 N tasks [F visible] | T due today | O overdue | Sort: Priority | [filter summary]    [MODE]
```

Where `[F visible]` is omitted if no filter is active (visible == total).

- Right-aligned **mode indicator**: `NORMAL` / `ADD` / `EDIT` / `FILTER` / `DELETE?`
- Transient messages replace the left portion for ~1.5 seconds, then revert: "Task 7 marked done ✓" / "Error: ..."

**Counts to show:**

| Field | Description |
|-------|-------------|
| Total | ALL tasks in the file (unfiltered) |
| Visible | Tasks currently shown (omit if equal to total) |
| Due today | Pending tasks with `due:` = today |
| Overdue | Pending tasks with `due:` < today |
| Sort | Active sort mode name (abbreviated if needed: `Pri`, `Due`, `File`, `Alpha`, `Proj`, `Ctx`) |
| Filter summary | Active filter tokens, e.g., `@work +Project` |
| Mode | Current input mode, right-aligned |

**Edge cases:**
- **Narrow terminal (< 60 cols):** Truncate in priority order: mode > counts > filter summary. Drop filter summary first, then abbreviate counts to "N/F".
- **Transient message arrives while another is showing:** Replace it; reset the 1.5s timer.
- **Zero tasks:** "0 tasks | 0 due today | 0 overdue"

---

### 9. Themeable Colors

**Expected behavior:**
- Ship 2 built-in named themes: `default` (optimized for dark terminals) and `light` (optimized for light terminals).
- Configure via the existing TOML config file under a `[tui]` section:

```toml
[tui]
theme = "default"          # or "light", or an absolute path to a custom .toml theme file
```

- Custom theme TOML file structure (all fields optional; unspecified slots fall back to built-in defaults):

```toml
[task.priority_a]
fg = "red"
modifiers = ["bold"]

[task.done]
fg = "dark_gray"
modifiers = ["crossed_out"]

[task.overdue]
fg = "red"
modifiers = ["bold"]

[task.selected]
modifiers = ["reversed"]

[status_bar]
bg = "blue"
fg = "white"
```

- Color values accept: named ANSI colors (`"red"`, `"green"`, `"dark_gray"`, etc.), indexed (`"Color(196)"`), or RGB hex (`"#FF5733"`).
- Honor `NO_COLOR` env var: if set, disable all color. Render in terminal defaults only.
- `TODOTXT_THEME` env var overrides the config file theme name.

**Named style slots (minimum set for v1.1):**

| Slot | Default (dark) | Purpose |
|------|---------------|---------|
| `task.priority_a` | bold red | Priority A |
| `task.priority_b` | bold yellow | Priority B |
| `task.priority_c` | bold green | Priority C |
| `task.priority_other` | default | Priority D–Z |
| `task.done` | dim + strikethrough | Completed tasks |
| `task.overdue` | red | Due date in the past |
| `task.due_today` | yellow | Due date = today |
| `task.selected` | reversed | Highlighted cursor row |
| `task.context` | cyan | `@context` tokens in text |
| `task.project` | magenta | `+project` tokens in text |
| `status_bar` | blue bg, white fg | Status bar |
| `filter_panel` | dark_gray bg | Filter panel background |
| `modal_border` | white | Confirmation modal border |

**Edge cases:**
- **Invalid color name in config:** Warn in status bar at startup ("Unknown color 'purpl' in theme, using default"). Fall back to built-in default for that slot only.
- **Terminal lacks 256-color / RGB support:** ratatui/crossterm auto-downgrades to nearest ANSI 16-color. No special handling needed in app code.
- **Theme file not found:** Fall back to built-in `default` theme; brief startup warning in status bar.

---

### 10. File-Watch Auto-Reload

**Expected behavior:**
- `todotxt-core`'s `notify` + `notify-debouncer-mini` file watcher runs continuously. Events are debounced at 500ms to absorb burst edits (editor saves, scripts).
- On a file-change event: reload task list in-memory, re-apply active filter and sort, re-render.
- **If not in add/edit mode:** reload immediately and silently (just re-render).
- **If in add/edit mode:** queue the reload; apply it when the user exits the mode (Enter or Esc). Do NOT interrupt active input.
- Cursor preservation: after reload, attempt to keep the cursor on the same task by task ID. If that task was deleted externally, move cursor to the nearest surviving position.

**Visual indicator:**
- Brief status bar flash on reload: `↺ Reloaded` for ~1 second, then revert to normal.
- No persistent "watching" icon in the status bar (always-on watchers should be silent).

**Keyboard shortcuts:**

| Key | Action |
|-----|--------|
| `r` | Force manual reload |

**Edge cases:**
- **File deleted externally:** Status bar: "todo.txt not found — waiting…". Poll every 5 seconds; auto-reload when file reappears.
- **File unreadable (permissions):** Show error in status bar. Do not crash.
- **Rapid external edits:** 500ms debounce absorbs bursts; one reload fires per quiet period.
- **`notify` not supported on platform:** Disable silently; emit a startup note to stderr. Manual `r` key still works.
- **Large file reload performance:** Target < 16ms (one frame). If profiling shows this is exceeded for large files, move reload to a background tokio task and swap the task list via `Arc<RwLock<Vec<Task>>>`.

---

## Differentiators

Nice-to-have. Do not block v1.1 ship on any of these.

---

### D1. Task Detail Pane

Toggle with `i` or `Enter`. A right-side split pane showing full task text (word-wrapped), all parsed fields (priority, creation date, completion date, due date, all `key:value` extensions), and the raw todo.txt line. Rare in todo.txt TUIs — most just truncate in the list view.

---

### D2. Scrollbar in Task List

Ratatui's `Scrollbar` widget on the right edge gives a visual position indicator for long lists (> 50 tasks). ~5 lines to implement: bind `ScrollbarState` to `ListState.offset` and `items.len()`.

---

### D3. Sort Persistence in Config

Persist the active sort mode to `[tui] sort = "priority"` in the TOML config so the user's preferred sort survives restarts.

---

### D4. Session Undo Stack

Press `u` to undo the last mutating action (mark done, delete, edit) within the current TUI session. Session-only (no cross-session undo). Each action pushes a `Vec<Task>` snapshot. Memory cost is stack depth × list size.

---

### D5. Quick Search via `/`

Press `/` to open a bottom-bar text input. Task list live-filters by substring. `Esc` clears and closes. This is the htop/vim `/` search convention — very discoverable. Complements the full filter panel (`f`).

---

### D6. Help Overlay

Press `?` to display a full-screen key binding reference. `Esc` or `q` to dismiss. Rendered as a `Paragraph` in a centered `Block::bordered()` popup.

---

### D7. Bulk Multi-Select

`v` enters visual mode. `j`/`k` extends the selection. `x` bulk-marks done; `d` bulk-deletes with a single "Delete N tasks? [y/N]" confirmation.

---

## Anti-Features

Skip for v1.1. Adding these would be scope creep or introduce maintenance burden disproportionate to value.

| Anti-Feature | Why Skip | What to Do Instead |
|--------------|---------|-------------------|
| Mouse support | Adds significant event-handling complexity for minimal gain in a keyboard-driven app; terminal mouse events are inconsistent across platforms and terminal emulators. | Ship keyboard-only for v1.1; add mouse in v1.2 if users request it. |
| Task dependency visualization | `dep:`/`parent:` are not in the core todo.txt spec; requires a DAG renderer. | Out of scope; consider for v2.0. |
| Recurrence rules (`rec:`) | Complex date arithmetic, edge cases (business days, month-end, leap years), not in spec. | The CLI `postpone` command handles the common case. |
| Vim-style command line (`:`) | Duplicates the CLI; adds a mini-parser for little gain. | Use `a`/`e`/`d`/`x` key shortcuts instead. |
| Cross-session undo history | todo.txt format IS the persistence layer. The file is the undo log. | Users who want history should version-control their todo.txt (git). |
| Plugin system / scripting | Massive complexity, no clear v1.1 user need. | The CLI `--json` output already enables external scripting. |
| Network sync UI | File sync is the user's responsibility (Dropbox, git, rsync). | Not a todo.txt format feature. |
| Calendar view | Different paradigm; requires a custom date-grid widget. | Use the `due:today` filter and `Overdue` count in the status bar. |
| Fuzzy search | Adds a dependency or substantial implementation for marginal gain over substring search. | Substring search via `/` covers 95% of use cases. Add fuzzy in v1.2 if requested. |

---

## Phase-Specific Notes for the Requirements Writer

| Feature | Estimated Complexity | Build Order Note |
|---------|---------------------|-----------------|
| Task list navigation | Low | Foundation — build first. Everything else depends on list state. |
| Status bar | Low | Build alongside navigation; drives visibility of all other features. |
| Mark done / undo | Low | Single `todotxt-core` call + cursor preservation. |
| Delete with confirmation | Low | Modal overlay pattern; reusable for future bulk ops. |
| File-watch auto-reload | Low–Medium | Core watcher exists; TUI needs the channel handler + edit-mode guard. |
| Sort toggle | Low | Cycle enum + stable sort; cursor-follow by task ID. |
| Add new task | Medium | `tui-textarea` + autocomplete popup (the popup geometry is the hard part). |
| Inline edit | Medium | Same as Add + edit-mode reload guard. Build after Add. |
| Themeable colors | Medium | Config parsing + `Theme` struct + `NO_COLOR`. Independent — parallelizable with other features. |
| Filter panel | Medium–High | Most widget surface area. Multi-section focus cycling. Build last or as its own phase. |

---

## Sources

- ratatui docs.rs — `List`, `ListState`, `scroll_padding`, `StatefulWidget`, `Clear`, `Scrollbar` widgets (verified 2026-04-18, v0.30.0)
- taskwarrior-tui GitHub README — vim-like navigation, live filter, color conventions, tab completion (verified 2026-04-18, v0.27.0)
- tui-textarea crate — single-line mode, undo/redo, Emacs shortcuts (from STACK.md research)
- General TUI conventions: htop, lazygit, gitui, neomutt — modal patterns, status bars, confirmation dialogs
- todo.txt format spec — task ID = file line number, raw text preservation requirement

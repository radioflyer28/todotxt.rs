# Feature Research — v1.6

## Summary

v1.6 is a power-user polish milestone: it closes the gap between what terminal-native users
expect from a mature task manager and what the TUI currently delivers. The features cluster
into three themes — **action completeness** (archive, bulk done, $EDITOR), **discoverability**
(autocomplete coverage, filter history), and **view control** (presets, persistence,
group-by decoupling). The test automation feature (SEED-005) is a hygiene item that makes
future feature work safer.

---

## Feature Analysis

### TUI Archive Hotkey (SEED-006)

**Table Stakes:**
- `A` key moves all completed tasks (`x ` prefix) from `todo.txt` to `done.txt` in one keystroke
- Status bar confirms: `Archived N task(s) to done.txt` (never silent on success)
- If `done.txt` does not exist, create it automatically — no confirmation required
- Only completed tasks are moved; incomplete tasks are never touched
- Undo available (single undo entry covers the full batch move)

**Differentiators:**
- Status message always includes the count ("Archived 0 tasks" vs "Archived 12 tasks" are meaningfully different information)
- No confirmation dialog for the common case — archive is reversible via undo and the archived file itself
- Cross-referencing with todotxt.net C# behavior: the original app archives silently and reports count; match this
- `archive_path` already resolved at startup — the hotkey always knows where done.txt lives without asking

**Anti-features:**
- **Silent operation** — users who press `A` and see nothing change will press it again; always show a status message
- **Confirmation dialog** — archive is not destructive (tasks are moved, not deleted); a confirmation step adds friction with no safety benefit when undo exists
- **Archiving incomplete tasks** — the action must filter `task.completed == true` only; archiving everything in view would be catastrophic

**UX Pattern:**
- taskwarrior `task done` + `task purge` — but the TUI collapses these into one keystroke since done.txt is a standard todo.txt convention
- lazygit stash operations: bulk side-effect action → count displayed in status bar → undo available

**Complexity note:**
Low. The CLI `archive.rs` already implements the full logic. The main risk is mid-write failure —
use a transaction pattern: append to done.txt first, then remove from todo.txt only if append succeeded.

---

### Bulk Mark-Done (SEED-009)

**Table Stakes:**
- `x` with a non-empty multi-selection marks all selected tasks done (not just cursor task)
- Single undo entry covers the entire batch (consistent with bulk delete behavior)
- Selection is cleared after the operation
- Mixed selection (some done, some not) → complete all incomplete ones; do NOT un-complete already-done tasks

**Differentiators:**
- Mixed-selection handling follows least-surprise: `x` means complete, never toggle-each-independently on bulk
- Count in status bar: "Marked 5 task(s) done" gives feedback proportional to the action
- No confirmation required — `x` is reversible via undo; the selection system is the confirmation mechanism (user built the selection intentionally)

**Anti-features:**
- **Toggle behavior on mixed selection** — applying `x` to a selection that includes 5 already-done tasks and un-completing them is almost never the intent; always-complete on bulk is safer
- **Confirmation dialog** — bulk delete has `DeleteConfirm` mode; bulk mark-done should NOT since it is far less destructive than deletion
- **Silent operation** — show a status message with count

**UX Pattern:**
- vim visual-mode + operation: selection is the scope declaration, the key is the action; the operation applies uniformly to all selected items without secondary confirmation
- taskwarrior `task 1 2 3 done` — IDs form the scope, `done` is the action, no confirm step

**Complexity note:**
Low. The bulk delete path is the direct template. Only wrinkle: if the pane hides completed tasks, newly
completed tasks disappear immediately on rebuild — correct behavior, but the status bar count reassures
users the tasks are not lost.

---

### Open Task in `$EDITOR` (SEED-012)

**Table Stakes:**
- `Ctrl+E` in Normal mode opens the cursor task's text in the user's `$EDITOR`
- Editor resolution order: `$VISUAL` → `$EDITOR` → platform fallback (`notepad.exe` on Windows, `vi` on Unix)
- TUI suspends (disable raw mode, leave alternate screen) before exec; resumes correctly after editor exits
- Changes saved in the editor are applied to the task on return
- Temp file is cleaned up after return, even on error

**Differentiators:**
- `$VISUAL` before `$EDITOR` is the POSIX convention (vim, less, mutt all follow this); skipping it alienates users who deliberately set `$VISUAL` for GUI editors
- If user saves with no changes (opens and immediately quits), treat as a no-op — do not create an undo entry
- If no editor is found, show a clear status bar error: `No editor found. Set $EDITOR or $VISUAL.` — never crash
- Validate edited text as a valid todo.txt task before applying (reject blank, reject multi-line content)

**Anti-features:**
- **Suspending without disabling raw mode** — ratatui raw mode left active while an external process writes to stdout produces terminal garbage; `crossterm::terminal::disable_raw_mode()` is required before `Command::status()`
- **Not cleaning up the temp file on error** — leaks temp files; use RAII or `scopeguard`
- **Silently applying only the first line of a multi-line save** — if the user edits and adds newlines, taking only line 1 silently is confusing; show an error instead
- **No fallback on Windows** — `vi` does not exist on Windows by default; `notepad.exe` is the safe fallback; document that users should set `$EDITOR`

**UX Pattern:**
- `git commit` (without `-m`) is the canonical model: suspend terminal, open editor, resume on exit, apply result
- `crontab -e`, `visudo`, `kubectl edit` — all follow the same write-temp / exec / read-back / validate / apply pattern
- helix shell interaction and vim's `!` — TUI→editor→TUI handoff is well-understood by terminal power users

**Complexity note:**
Medium. The suspend/resume ratatui pattern has prior art but requires care: `disable_raw_mode` +
`LeaveAlternateScreen` before exec, `enable_raw_mode` + `EnterAlternateScreen` + full redraw after return.
The terminal may also have been resized while the editor was open — trigger a resize event on return to
avoid stale layout.

---

### Fix `+` Project Autocomplete (SEED-013)

**Table Stakes:**
- Typing `+` in the task editor shows a popup of existing project tags (identical to `@` for contexts)
- Popup narrows as additional characters are typed after `+`
- `+` and `@` autocomplete behavior is indistinguishable to the user — if one works, both work

**Differentiators:**
- If no `+project` tags exist in `todo.txt`, show a hint in the popup: "(no existing projects)" rather than a silent empty popup — users need to know the system is working, not broken
- Popup appears even if there are no matching entries (the `+` itself is the trigger, not the presence of matches)

**Anti-features:**
- **Silent empty popup** — the most confusing state; the popup appears briefly and immediately disappears, leading users to conclude the feature is broken
- **`++myproject` double-prefix** — if `t.projects` stores `"+myproject"` (with prefix) and `accept_completion` prepends another `+`, the result is invalid; fix the data representation first
- **Wrong rfind trigger position** — if a task already has `@work`, typing `+` should trigger on `+`, not `@`; `rfind` must return the last trigger character's position

**UX Pattern:**
- This is a bug fix restoring an established pattern that already works for `@`; the target behavior is exact parity
- zsh/fish completion: both sigil types behave identically; asymmetry is always a bug

**Complexity note:**
Low (bug fix). Three candidate root causes are distinguishable in 15 minutes of debugging. Fix SEED-013
before implementing SEED-014 — filter autocomplete will inherit the same data representation bug if
`t.projects` stores wrong values.

---

### Autocomplete in Filter Input + Incremental Narrowing (SEED-014)

**Table Stakes:**
- Typing `@` or `+` in the filter input bar shows the same autocomplete popup as in the task editor
- The popup narrows as characters after the sigil are typed
- Accepting a completion inserts into the filter bar, not the task editor

**Differentiators:**
- **Incremental narrowing while popup is open**: once the popup is visible, continuing to type further narrows the list without requiring the user to close and reopen it; this is the critical UX gap
- While popup is focused (navigating with `↑/↓`), typing a character adds to the current filter prefix and re-narrows, rather than closing the popup
- `accept_completion` inserts the selected completion into the filter bar and immediately applies the filter, showing results in real time

**Anti-features:**
- **Popup closes on any character when navigating** — once the user presses `↓` to navigate the popup, pressing any character should continue narrowing, not close the popup
- **Inserting into task editor instead of filter bar** — `accept_completion` currently targets `self.editor`; the target must be switched when in Filtering mode
- **Not deduplicating suggestions** — if `@work` appears in 50 tasks, it should appear once in the popup; `collect_tokens` must dedup

**UX Pattern:**
- fish shell inline completion: as you type, completions narrow in real time without a separate "activate narrowing" gesture
- fzf: input string continuously narrows the displayed list; the mental model is "type to filter, not type then filter"
- helix `:open` command palette: each character narrows matches incrementally while arrow keys navigate

**Complexity note:**
Medium. Three sub-items (edit-mode gap audit, filter-bar autocomplete, incremental narrowing) are
independently shippable. The hardest part is `accept_completion` target routing — a `target_editor: EditorTarget`
enum (TaskEditor vs FilterEditor) avoids divergent code paths. The incremental narrowing may already partially
work via `update_autocomplete` on each keypress; test before implementing a new system.

---

### Filter Input History (SEED-011)

**Table Stakes:**
- `Ctrl+H` (or `Ctrl+R`) in the filter input opens a popup showing recently used filter expressions
- Within-session history is available immediately (in-memory; no persistence required for the basic feature)
- Deduplication: re-typing the same filter does not add a second entry
- Maximum history size: 50 entries — avoids unbounded growth

**Differentiators:**
- **Separate from named presets** — history (`Ctrl+H`) and named presets (`↑/↓`) must use different keys; mixing them causes named presets to drift out of expected positions, which is disorienting
- **Cross-session persistence via SEED-007 state file** — if view state persistence is implemented, filter history rides along in the same file
- **History popup that narrows as you type** — `Ctrl+H` opens popup, then characters narrow the history list (same pattern as `Ctrl+R` in zsh); far more powerful than raw `↑/↓` cycling for users with long histories

**Anti-features:**
- **Polluting history with preset applications** — pressing `1`–`9` applies a named preset; this must NOT be recorded in the ad-hoc history ring (presets are intentional, history is for ad-hoc expressions)
- **Using `↑/↓` for history** — these keys already cycle named presets; repurposing them for history breaks existing behavior and creates a confusing dual meaning
- **Storing blank or whitespace-only entries** — clearing the filter (`0` key) must not be recorded as a history entry

**UX Pattern:**
- zsh/bash `Ctrl+R`: the canonical reverse-history-search model; terminal users already have muscle memory for this
- fish history: deduplication by default, cross-session persistence, incremental narrowing — the gold standard
- The separate key (`Ctrl+H` vs `↑/↓` for presets) maps to the mental distinction: "things I saved intentionally" vs "things I typed recently"

**Complexity note:**
Medium. Two distinct sub-features: in-memory ring buffer (low complexity) and cross-session persistence
(requires SEED-007 coordination). The ring buffer is a standalone struct in `state.rs`. The key binding must
not conflict with `↑/↓` preset navigation — both `Ctrl+R` and `Ctrl+H` are available in the filter input.

---

### Expand Numeric Presets (SEED-015)

**Table Stakes:**
- `1`–`9` preset keys can optionally specify `sort`, `group`, and other view dimensions in addition to `filter`
- Existing `filter`-only preset configs continue to work without modification (fully backwards compatible)
- All specified dimensions are applied atomically in a single rebuild
- Unspecified dimensions are left as-is (a sort-only preset does not reset the filter)

**Differentiators:**
- **Atomic application** — when a preset specifies sort + group + filter, all three change simultaneously with one rebuild; multi-step application creates visible flicker and breaks undo granularity
- **Partial preset semantics** — "absent field = no-op" is more powerful than "absent field = reset to default"; users can define a sort-only preset without destroying their current filter
- **Future pane focus field** — `pane_focus: Option<usize>` fits naturally in the same struct and enables workspace-switching workflows

**Anti-features:**
- **Resetting unspecified dimensions** — if a partial preset wipes dimensions the user didn't ask to change, the preset becomes unpredictable and users stop using it
- **Breaking existing configs** — if old `filter`-only TOML entries stop parsing after the struct change, users get a silent startup failure; all new fields MUST be `Option<T>` with `None` as the absent-field default
- **Polluting filter history** — applying a preset via `1`–`9` must not write to the ad-hoc filter history ring; this design contract must be explicit in code, not emergent

**UX Pattern:**
- tmux session workspaces: a numbered preset is a complete named environment, not just one parameter; pressing it "teleports" to a known configuration
- lazygit custom commands: short keys apply compound operations atomically
- i3/sway workspace numbers: the analogy is exact — number key switches to a pre-configured view layout

**Complexity note:**
Low–Medium. The struct extension is mechanical (all `Option<T>`); the apply handler needs two additional
conditional branches. Main design decision: when SEED-007 view state persistence is also active, explicit
preset application overrides persisted state for the specified dimensions — this precedence must be documented.

---

### View State Persistence (SEED-007)

**Table Stakes:**
- Sort order, grouping toggle, filter query, and active pane index survive a clean exit and restart
- State is loaded at startup before config defaults are applied (runtime state overrides static config)
- If the state file is absent (first run), fall back silently to `config.toml` defaults — no error
- If the state file is corrupted (malformed TOML), fall back silently to config defaults — no crash, log a warning

**Differentiators:**
- **Atomic write on exit** — write to `.tui-state.toml.tmp` then rename; prevents a partial write from corrupting the state file on crash or power loss
- **Sidecar file placement** — `.tui-state.toml` next to `todo.txt` means state travels with the todo file; users who sync via Dropbox or git get workspace-specific state automatically
- **Per-pane state for multi-pane layouts** — each pane has independent sort/filter/group; persist per-pane, not just a global snapshot
- **Write only on clean exit** — not on SIGKILL or panic; a stale file is better than a corrupted one

**Anti-features:**
- **Writing state on every keypress** — expensive and unnecessary; write once on graceful exit
- **Persisting config-defined panes without user modification** — if a pane was defined in `config.toml` and never interactively changed, its source of truth is the config file; persisting it separately makes config changes invisible until the state file is manually deleted
- **Global state file shared between multiple todo.txt files** — if the user has two different `todo.txt` files, state must be scoped to each; a global state file mixes unrelated contexts

**UX Pattern:**
- vim `viminfo` / neovim `shada`: session state (marks, jumplist, registers) persists transparently; users set it up once and stop thinking about it
- lazygit: panel positions and current branch view survive restarts; users expect their "last position" to be the starting position
- The C# todotxt.net reference app persists via `User.settings` (Windows registry); the Rust port provides equivalent guarantee without Windows registry dependency

**Complexity note:**
Medium. The `Pane` struct already has all the fields that need persisting. Main complexity: (1) atomic write
on exit, (2) platform-appropriate file path, (3) serde derive on `Pane` and nested types, (4) graceful error
handling on load. File placement: `.tui-state.toml` next to `todo.txt` is workspace-scoped, user-visible,
and easy to inspect or delete.

---

### Decouple Group-By from Sort Order (SEED-008)

**Table Stakes:**
- `G` (Shift+G) cycles the group-by category independently of sort order
- `o` continues to cycle intra-group sort order (existing behavior, unchanged)
- `g` continues to toggle grouping on/off (existing behavior, unchanged)
- Status bar shows both active group-by category and intra-group sort order when grouping is enabled

**Differentiators:**
- **Meaningful status bar** — when grouping is on, show: `Group: Project | Sort: Due Date`; showing only one dimension is less informative than what was available before the decoupling
- **`GroupBy` as a distinct type** — not aliased to `SortOrder`; prevents future confusion where `SortOrder::FileOrder` makes no sense as a group-by category
- **Sensible group-by cycle** — `Priority`, `Project`, `Context`, `DueDate` are meaningful groups; `Alphabetical` and `FileOrder` produce trivial or degenerate groupings and may be included but documented as edge cases

**Anti-features:**
- **`G` conflicting with vim-style goto-end** — the TUI is not a vim clone, but power users have muscle memory; mitigate by documenting that `G` is remappable via `[keymap]` config
- **Single-bucket groups** — if `GroupBy::FileOrder` puts all tasks in one unlabeled bucket, it should either be excluded from the cycle or labeled "ungrouped"
- **Not persisting `group_by` separately from `sort_order`** — when SEED-007 is implemented, `group_by` must be a distinct field in the state file; conflating it recreates the exact coupling being fixed

**UX Pattern:**
- taskwarrior `rc.report.next.sort` vs `rc.report.next.group`: separate config keys; grouping and sorting are orthogonal axes in every mature task manager
- lazygit: file changes can be grouped by type and sorted by time — independently selectable
- The `g` / `G` / `o` tri-key scheme (toggle / cycle-group / cycle-sort) maps cleanly to three independent operations without overloading any single key

**Complexity note:**
Medium. Adding `GroupBy` enum and `group_by` field to `Pane` is mechanical. The structural change is
refactoring `group_key_for` to accept `&GroupBy` instead of `&SortOrder` — this touches `rebuild_display_indices`
and `rebuild_all_panes`. The status bar update to show two fields requires layout work but is cosmetically bounded.

---

### Automate Phase 22 Tests (SEED-005)

**Table Stakes:**
- All 11 manual-only validation gaps from the Phase 22 VALIDATION.md are covered by automated unit tests
- Tests use the existing `make_app_with_tasks` helper pattern (or an extended `make_app_with_keymap` variant)
- Tests run in < 5 seconds total (no real file I/O except for the `task_list.reload()` test)
- Phase 22 transitions to `nyquist_compliant: true`

**Differentiators:**
- **`make_app_with_keymap` helper** — a reusable test constructor that accepts a keymap config makes all 11 tests straightforward; equally useful for v1.6 feature tests (mode transitions for archive, bulk-done, $EDITOR will all need the same pattern)
- **Tests as living documentation** — good test names serve as behavioral specs: `test_pressing_A_archives_and_shows_count`, `test_pressing_x_with_selection_marks_all_done`

**Anti-features:**
- **Tests requiring a real `todo.txt` path** — `App::new` requires a file; tests that create real temp files are slow and flaky; `make_app_with_tasks` should mock the TaskList with in-memory data
- **Testing implementation details instead of behavior** — assert `app.mode == AppMode::X` and `app.filter_query == "Y"`, not internal state like `app.effective_keymap.get("n").is_some()`
- **Skipping the `task_list.reload()` test** — the tempfile test is slightly harder but `std::env::temp_dir()` + the `tempfile` crate makes it manageable; leaving it out leaves a real behavioral gap

**UX Pattern:**
- This is a developer-experience feature; the "UX" is `cargo test` returning green with 11 new test names
- helix and ratatui test suites: construct app state programmatically, simulate key events, assert state — no terminal required

**Complexity note:**
Low. The seed identifies all 11 test cases with exact code locations. The only structural work is the
`make_app_with_keymap` helper. Estimated effort: ~2 plans, ~4 hours. High value for small investment —
makes every subsequent v1.6 feature safer to develop. Implement first.

---

## Cross-Feature Notes

### Interaction Map

| Feature A | Feature B | Interaction |
|-----------|-----------|-------------|
| View presets (SEED-015) | View state persistence (SEED-007) | Preset application overrides persisted state for specified dimensions; implement this precedence explicitly |
| View presets (SEED-015) | Filter history (SEED-011) | Applying a preset via `1`–`9` must NOT write to the ad-hoc filter history |
| Group-by decoupling (SEED-008) | View state persistence (SEED-007) | `group_by` must be persisted as an independent field alongside `sort_order` |
| Group-by decoupling (SEED-008) | View presets (SEED-015) | Future: `group_by` becomes a 4th optional preset field in `TuiPreset` |
| Fix `+` autocomplete (SEED-013) | Filter autocomplete (SEED-014) | Fix SEED-013 first; SEED-014 will inherit the data representation bug if `t.projects` values are wrong |
| Phase 22 tests (SEED-005) | All other features | The `make_app_with_keymap` helper built for SEED-005 should be reused by all v1.6 feature tests |

### Recommended Implementation Order

1. **SEED-005** — test infrastructure; enables safe development of everything else
2. **SEED-013** — bug fix (highest priority, smallest scope); unblocks SEED-014
3. **SEED-006** — archive hotkey (small, self-contained, high user value)
4. **SEED-009** — bulk mark-done (small, direct analogue of existing bulk delete)
5. **SEED-012** — $EDITOR (small, well-established pattern, high power-user value)
6. **SEED-008** — group-by decoupling (medium, foundational for presets)
7. **SEED-007** — view state persistence (medium, foundational for history and presets)
8. **SEED-015** — expand presets (medium, depends on SEED-007 and SEED-008 being clear)
9. **SEED-014** — autocomplete coverage (medium, depends on SEED-013 fix)
10. **SEED-011** — filter history (medium, depends on SEED-007 for cross-session persistence)

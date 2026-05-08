# Research Summary — v1.6: TUI Fixes and Power User Improvements

**Synthesized:** 2026-05-04
**Sources:** STACK.md, FEATURES.md, ARCHITECTURE.md, PITFALLS.md

---

## Stack Additions

**No new crates required.** All ten v1.6 features are implementable with the existing workspace dependency set.

| Action | Crate | What's Needed |
|--------|-------|---------------|
| Promote to runtime dep | `tempfile = "=3.27.0"` | SEED-012 ($EDITOR temp file). Already in workspace; already a `[dev-dependencies]` in `todotxt-tui`. Move one line in `Cargo.toml`. |
| Already available | `crossterm =0.28.1` | Suspend/resume raw mode and alternate screen for SEED-012. |
| Already available | `toml =0.8.23` + `serde =1.0.228` | `tui-state.toml` sidecar (SEED-007); new `GroupBy` enum (SEED-008); expanded `TuiPreset` (SEED-015). |
| Already available | `ratatui =0.29.0` | Popup/overlay rendering for filter history (SEED-011) and filter autocomplete (SEED-014). `Clear` + `Block` + `Layout` are sufficient — no `tui-popup` crate needed. |
| Already available | `directories =6.0.0` | Resolve `tui-state.toml` path via existing `ProjectDirs` call in `config.rs`. |

**Critical constraint:** Every new field on any serialized struct (`TuiPreset`, `PaneConfig`, `TuiViewState`) **must** carry `#[serde(default)]`. The existing codebase pattern (see `PaneSort` in `config.rs`) is the established template. Missing this on a single field breaks startup for all existing users with that key set in their config.

---

## Feature Table Stakes

### Action Completeness

**SEED-006 — TUI Archive Hotkey (`A`)**
- Moves all completed tasks from `todo.txt` to `done.txt` in one keystroke
- Status bar always shows count: `Archived N task(s) to done.txt` — never silent
- Creates `done.txt` if absent — no confirmation required
- Atomic write: append to done.txt succeeds *before* removing from todo.txt
- Cursor is re-clamped and display is rebuilt immediately after archive

**SEED-009 — Bulk Mark-Done (`x` with multi-selection)**
- `x` with a non-empty selection marks all selected tasks done (never toggles)
- Mixed selection (some already done): complete only the incomplete ones
- Status bar shows count: `Marked N task(s) done`
- Single undo entry covers the entire batch; selection cleared after operation
- `rebuild_display` called once after all mutations — never between individual completions

**SEED-012 — Open Task in `$EDITOR` (`Ctrl+E`)**
- Editor resolution: `$VISUAL` → `$EDITOR` → `notepad.exe` (Windows) / `vi` (Unix)
- TUI suspends (disable raw mode + leave alternate screen) before exec; fully resumes after exit
- Temp file is cleaned up after return, even on error
- If user saves with no changes: no-op, no undo entry created
- Validates edited text: reject blank, reject multi-line (show error, do not silently truncate)
- If no editor found: status bar error `No editor found. Set $EDITOR or $VISUAL.` — never crash

### Discoverability

**SEED-013 — Fix `+` Project Autocomplete (bug fix)**
- Typing `+` in task editor shows a project popup, identical to `@` for contexts
- Popup narrows as characters are typed after `+`
- Root cause: `collect_tokens()` uses byte-exact dedup; `get_existing_projects()` is case-insensitive — align both to use the same dedup logic
- Fix `++project` double-prefix bug: `accept_completion` must delete the typed prefix before inserting
- **Fix this before SEED-014** — filter autocomplete inherits the same data representation

**SEED-014 — Autocomplete in Filter Input + Incremental Narrowing**
- `@` or `+` in filter bar shows the autocomplete popup (same as task editor)
- Popup narrows incrementally as characters are typed — no separate "activate narrowing" gesture
- `accept_filter_completion()` inserts into filter bar, applies filter in real time
- While popup is navigated (`↑/↓`), typing continues narrowing — does not close popup

**SEED-011 — Filter Input History (`Ctrl+R`)**
- Opens a popup of recently-used filter expressions (in-memory ring buffer, cap 50)
- Deduplication: same filter typed twice adds only one entry
- Separate from named presets — `Ctrl+R` is history; `↑/↓` stays for preset cycling
- Applying a preset via `1`–`9` does NOT record to ad-hoc history
- Cross-session persistence via SEED-007 `tui-state.toml` (rides along in same file)

### View Control

**SEED-015 — Expand Numeric Presets (`1`–`9`)**
- Preset keys can optionally specify `sort`, `group_by`, and `label` in addition to `filter`
- Absent fields are no-ops — a sort-only preset preserves the current filter
- All specified dimensions applied atomically in one rebuild (no flicker)
- Existing filter-only TOML configs must continue to work without modification

**SEED-007 — View State Persistence**
- Sort, group-by, filter, and active pane index survive clean exit and restart
- Written to `tui-state.toml` (next to `todo.txt`, not in config dir)
- Atomic write on exit (write to `.tmp` then rename); never write on SIGKILL/panic
- Absent or corrupted file: fall back silently to `config.toml` defaults — never crash

**SEED-008 — Decouple Group-By from Sort Order**
- `G` (Shift+G) cycles group-by category independently of sort order
- `o` continues cycling intra-group sort (existing behavior, unchanged)
- Status bar shows both dimensions when grouping is enabled: `Group: Project | Sort: Due Date`
- `GroupBy` is a new enum — not aliased to `SortOrder`

### Quality

**SEED-005 — Automate Phase 22 Tests**
- All 11 manual-only gaps from Phase 22 VALIDATION.md covered by unit tests
- Uses extended `make_app_with_config()` / `make_app_with_panes()` helpers
- Tests run in < 5 seconds; no real file I/O except the `task_list.reload()` test
- Phase 22 transitions to `nyquist_compliant: true`

---

## Suggested Build Order

Architecture drives this order — three hard dependency chains:
1. `GroupBy` (SEED-008) must exist before `TuiPreset.group_by` (SEED-015) and before `ViewState` schema is finalized (SEED-007)
2. `collect_tokens` fix (SEED-013) must land before filter autocomplete (SEED-014) inherits the bug
3. `FilterHistory` struct (SEED-011) must exist before it can be wired into filter autocomplete navigation (SEED-014)

```
Wave 1 — Self-contained, no structural deps (can be built independently)
  SEED-013  Fix + project autocomplete       bug fix, unblocks SEED-014
  SEED-012  Open task in $EDITOR             suspend/resume in tui.rs, independent
  SEED-009  Bulk mark-done                   mirrors existing bulk delete path
  SEED-006  TUI archive hotkey               App::archive(), ~35 lines of file I/O

Wave 2 — Structural (must land before anything that depends on GroupBy)
  SEED-008  Decouple GroupBy                 GroupBy enum + Pane.group_by replaces .grouping
  SEED-005  Extend test helpers              make_app_with_config/panes valid after SEED-008

Wave 3 — Builds on GroupBy
  SEED-015  Expand numeric presets           TuiPreset gains sort/group_by; requires GroupBy
  SEED-011  Filter history                   FilterHistory struct in state.rs; needed by SEED-014

Wave 4 — Builds on fixed autocomplete + history
  SEED-014  Filter autocomplete + narrowing  requires SEED-013 fix + SEED-011 history

Wave 5 — Serializes the final schema
  SEED-007  View state persistence           serializes GroupBy (SEED-008) + FilterHistory (SEED-011)
```

**Phase grouping suggestion:**
- Phase A: Wave 1 (4 self-contained features — fast wins, establish test patterns)
- Phase B: SEED-008 + SEED-005 (structural change + tests, ship together)
- Phase C: SEED-015 + SEED-011 (both build on Wave 2 output)
- Phase D: SEED-014 (autocomplete wiring, builds on SEED-013 + SEED-011)
- Phase E: SEED-007 (persistence, depends on final schema from all prior phases)

---

## Watch Out For

Ranked by risk level (CRITICAL → HIGH → MEDIUM):

### 1. CRITICAL — Terminal left in raw mode if `$EDITOR` crashes (SEED-012)
If the suspend/resume sequence fails or the child process exits with an error, crossterm raw mode stays active and the user's shell becomes unusable.
**Mitigation:** Wrap the entire spawn-wait-restore block in a `Drop`-based guard. Call `disable_raw_mode()` + `LeaveAlternateScreen` before `Command::spawn()`. Restore raw mode and alternate screen even if the child exits non-zero. After resume, flush with `Clear(ClearType::All)` to handle residue from editors that modify terminal state (e.g., nvim with plugins).

### 2. CRITICAL — `accept_completion()` writes to wrong widget in filter mode (SEED-014)
The current implementation dispatches to `self.editor` unconditionally. If `AppMode::Filtering` is active and the filter bar is a separate `TextArea`, an accepted autocomplete token lands in the task editor buffer — silent data corruption.
**Mitigation:** Add a mode guard at the top of `accept_completion()`; if in `Filtering` mode, target the filter widget. Architecture research recommends two methods (`accept_completion()` and `accept_filter_completion()`) over a shared path with side-channel dispatch.

### 3. HIGH — `GroupBy` config migration breaks existing users (SEED-008)
`PaneConfig.group: bool` → `PaneConfig.group_by: GroupBy` is a breaking TOML schema change. Any `config.toml` with `group = true` under a `[[panes]]` block fails to deserialize on upgrade.
**Mitigation:** Keep the existing `group: bool` field as `#[serde(default)]` deprecated fallback. In `panes_from_config()`, detect `group = true` with no `group_by` key and map it to `GroupBy::Priority`. Add a smoke test that deserializes an old-format config and confirms `group_by` gets the correct inferred value.

### 4. HIGH — Archive write failure after task already removed from todo.txt (SEED-006)
If `done_file` is not configured or not writable, the current naive implementation would remove tasks from `task_list` before discovering the write failure — permanent data loss.
**Mitigation:** Check `done_file` is `Some(_)` before starting. Open done.txt for append *first*; only remove tasks from `task_list` after the append write succeeds. Use a transaction pattern: write → verify → mutate.

### 5. HIGH — View state TOML forward-compatibility (SEED-007)
If `TuiViewState` uses `#[serde(deny_unknown_fields)]` or any field lacks `#[serde(default)]`, a downgrade or partial-write file will panic at startup and lock the user out.
**Mitigation:** Never use `deny_unknown_fields` on any state struct. Every field gets `#[serde(default)]`. On any deserialization error, rename the file to `tui-state.toml.bak` and continue with defaults — never hard-fail startup on a corrupted state file.

---

## Key Decisions to Make

These must be locked before implementation begins for the affected phases.

### Decision 1 — Archive scope: all-done vs visible-done (SEED-006)
**Options:**
- A) Archive all completed tasks in `task_list` regardless of current filter
- B) Archive only completed tasks visible in the current filtered view

**Recommended:** Option A. The `A` hotkey meaning "archive everything done" is the standard todo.txt convention (matches C# app behavior). Scoping to the filter would surprise users who expect a global operation.

### Decision 2 — Is archive undoable? (SEED-006)
**Options:**
- A) Not undoable; archive gets a confirmation dialog
- B) Undoable via the existing `undo_entry` single-snapshot mechanism (restores `task_list` but leaves duplicates in `done.txt`)
- C) Full two-phase undo (complex: must also truncate `done.txt`)

**Recommended:** Option A. Archive is not destructive (tasks exist in `done.txt`). A one-time confirmation dialog is a lower-risk contract than a partial undo that silently leaves duplicates. Bulk mark-done (SEED-009) uses single-snapshot undo as-is (no `done.txt` involvement).

### Decision 3 — `accept_completion` architecture (SEED-014)
**Options:**
- A) Single shared method with a mode guard
- B) Two separate methods: `accept_completion()` (task editor) and `accept_filter_completion()` (filter bar)

**Recommended:** Option B. Borrow checker makes the shared helper awkward. Filter completion may diverge (e.g., no date autocomplete in filter mode). Separation is cleaner long-term.

### Decision 4 — `tui-state.toml` placement (SEED-007)
**Options:**
- A) Next to `todo.txt` (workspace-scoped; travels with the data file)
- B) In the OS config dir alongside `config.toml`

**Recommended:** Option A. State is specific to a todo file, not to the machine. Users with multiple `todo.txt` files get independent state automatically. Use `ViewState::path_from_config(config_path)` helper to derive the path.

### Decision 5 — Filter history key binding (SEED-011)
**Options:**
- A) `Ctrl+R` (canonical zsh/bash reverse-search muscle memory)
- B) `Ctrl+H` (mnemonic for "history")

**Recommended:** Option A (`Ctrl+R`). Terminal power users have deep muscle memory for this. `Ctrl+H` risks collision with backspace in some terminal emulators. Make the binding user-configurable via `[keymap]` config.

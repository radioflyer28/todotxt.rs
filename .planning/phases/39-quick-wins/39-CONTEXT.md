# Phase 39 — Quick Wins: Discussion Context

**Phase:** 39 — Quick Wins  
**Milestone:** v1.6 — TUI Fixes and Power User Improvements  
**Date:** 2026-05-04  
**Status:** Discussed — ready for planning

## Phase Goal

Users gain four previously-missing TUI capabilities (archive, bulk mark-done, external
editor, autocomplete `+` fix) in a single pass of independent feature work.

## Requirements in Scope

- ARCH-01/02/03 — Archive workflow
- BDONE-01/02 — Bulk mark-done
- XEDIT-01/02/03 — External editor launch
- AC-01 — `+` autocomplete verification / fix

## Depends On

Nothing — all four features are independent.

## Prior Decisions Inherited

From Phase 33 context:
- D-06: Autocomplete keyboard: Up/Down navigates, Tab/Enter accepts, Esc cancels
- D-05: Case-insensitive prefix-first matching, near-matches after
- D-04: Candidate source is deduplicated token corpus from `t.contexts` / `t.projects`

From Phase 36 context:
- D-01: `Ctrl+Z` for undo (no single-key shortcuts for destructive actions)

From Phase 35 context:
- D-02: If clipboard write fails, surface warning and no-op (safe degradation pattern)

---

## Decisions Made in This Discussion

### ARCH-A: Archive Confirmation Flow

**Decision:** Add `AppMode::ArchiveConfirm` — same pattern as `DeleteConfirm`.  
Triggered by `A` key. Shows `"Archive N completed tasks? [y/n]"` in a dedicated panel
row. `y` proceeds, any other key cancels. Count is the number of incomplete-filtered
completed tasks visible (per ARCH-02 requirement: all completed, not just visible).

**Rationale:** Consistent with existing bulk-delete confirm pattern. User gets count
preview before an irreversible file write.

**Implementation anchor:** Mirror `AppMode::DeleteConfirm` → `handle_delete_confirm_key()`
pattern. New `AppMode::ArchiveConfirm`, new `handle_archive_confirm_key()`.

---

### ARCH-B: Undo Behavior for Archive

**Decision:** Push `push_undo_entry()` before archive (snapshot todo.txt tasks).
Undo restores `todo.txt` only — `done.txt` is **not** reverted.

**Rationale:** `done.txt` is append-only by convention and may grow large over time.
Snapshotting it in memory per archive is inefficient. The user gets a practical safety
net (restore the tasks to todo.txt) without the complexity of full bilateral undo.

**Linked seed:** SEED-016 — rotate done.txt like Linux log files. If done.txt rotation
is implemented in a future milestone, done.txt would always be small and full bilateral
undo becomes practical again. The two features are intentionally linked.

**UX note:** Status bar after archive: `"Archived N tasks  (Ctrl+Z to restore to todo.txt)"`.
After undo: `"Undo: N tasks restored to todo.txt (done.txt unchanged)"`.

---

### XEDIT-A: Raw Mode Suspend/Restore Strategy

**Decision:** Use a Drop guard (RAII) — `struct RawModeGuard` that calls
`crossterm::terminal::disable_raw_mode()` on construction and
`crossterm::terminal::enable_raw_mode()` on `Drop`.

**Rationale:** Protects against panics, early `?` returns, and any error path between
disable and re-enable. Identified as CRITICAL risk in PITFALLS.md — Drop guard is the
correct mitigation.

**Implementation sketch:**
```rust
struct RawModeGuard;
impl RawModeGuard {
    fn new() -> color_eyre::Result<Self> {
        crossterm::terminal::disable_raw_mode()?;
        Ok(RawModeGuard)
    }
}
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = crossterm::terminal::enable_raw_mode();
    }
}
```

---

### XEDIT-B: Windows Editor Fallback

**Decision:** Platform fallback when both `$VISUAL` and `$EDITOR` are unset:
- Windows: `notepad.exe`
- Linux/macOS: `nano` if available, else `vi`

**Detection order:** `$VISUAL` → `$EDITOR` → platform fallback → error if fallback
binary not found.

**Note:** `notepad.exe` opens a GUI window on Windows, not inline in the terminal.
This is acceptable for a fallback — the TUI sits idle (blocked on `child.wait()`)
while the user edits in the GUI window.

---

### XEDIT-C: What the External Editor Opens

**Decision:** Open `todo.txt` directly (full file).

**Rationale:** User explicitly chose this. Enables batch editing beyond the single
task. After editor exits, reload from disk via `task_list.reload()` (or equivalent
full reload + `rebuild_all_panes()`).

**Safety notes for implementation:**
- Push `push_undo_entry()` before opening the editor (snapshot pre-edit state)
- After reload, cursor position should be preserved as best-effort (may shift if
  lines were added/removed)
- If editor exits non-zero, show status bar warning but still reload (user may have
  made valid edits and the editor just returned a non-zero code)

---

### BDONE: Undo Entry Timing for Bulk Mark-Done

**Decision:** One `push_undo_entry()` before the loop — consistent with `bulk_delete`.

**Rationale:** The undo system is single-slot (`Option<UndoEntry>`). Pushing per-task
would just overwrite the slot each time; only the last snapshot would survive anyway.
Pushing once before the loop gives the correct semantic (restore all tasks to their
pre-done state together) and is consistent with the existing bulk-delete pattern.

---

### AC-01: `+` Autocomplete Verification

**Decision:** Mark as **needs-verification** before coding. Phase 39 plan should
include a test step (or manual verification) to confirm whether `+` autocomplete is
actually broken before writing any fix.

**Rationale:** Code inspection shows `@` and `+` take identical paths through
`update_autocomplete()` and `collect_tokens()`. The parser stores projects without
the `+` prefix (`t.projects = ["work"]`, not `["+work"]`), and `accept_completion()`
inserts `trigger + token`, so the reconstruction is correct. The original bug report
(AC-01 in REQUIREMENTS.md) may have been based on a symptom that was already fixed
or may be a rare edge case.

**If verification confirms a bug:** Fix `update_autocomplete()` or `accept_completion()`
as needed.  
**If verification finds no bug:** Close AC-01 as no-issue and note it in Phase 39
VERIFICATION.md.

---

## Key Codebase Anchors

| Feature | Primary file | Key functions |
|---------|-------------|---------------|
| Archive | `crates/todotxt-tui/src/app.rs` | mirror `handle_delete_confirm_key()` |
| Archive file I/O | `crates/todotxt-cli/src/commands/archive.rs` | `run_archive()` — reuse/adapt |
| Archive config | `crates/todotxt-tui/src/config.rs:133` | `archive_path: PathBuf` |
| Bulk done | `crates/todotxt-tui/src/app.rs:2735` | `pane_toggle_done()` — single task pattern |
| Bulk done key handler | `app.rs:1994` | mirror `handle_delete_confirm_key()` bulk path |
| External editor | `crates/todotxt-tui/src/app.rs` | new `launch_external_editor()` fn |
| Raw mode | crossterm | `disable_raw_mode()` / `enable_raw_mode()` |
| Autocomplete | `app.rs:1847` | `update_autocomplete()` |
| Autocomplete tokens | `app.rs:1836` | `collect_tokens()` |
| Project storage | `crates/todotxt-core/src/task.rs:407` | bare names, no `+` prefix |

## Success Criteria (from ROADMAP.md)

1. `A` → ArchiveConfirm dialog with count → status bar message after archive
2. `x` with selection → all incomplete tasks marked done; already-done left unchanged
3. `Ctrl+E` → opens todo.txt in `$VISUAL`/`$EDITOR`/platform fallback; suspends raw
   mode before; restores after (including on crash via Drop guard)
4. If no editor found: status bar error, no data loss
5. `+` autocomplete: verified working (or bug confirmed and fixed)

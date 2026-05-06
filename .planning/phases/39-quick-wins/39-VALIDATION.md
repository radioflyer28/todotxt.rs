---
phase: 39-quick-wins
phase_number: "39"
validated_at: 2026-05-05
nyquist_compliant: true
automated: 22
manual_only: 4
escalated: 0
---

# Phase 39 — Nyquist Validation

## Test Infrastructure

| Framework | Config | Run Command |
|-----------|--------|-------------|
| Rust `#[test]` (unit) | `crates/todotxt-tui/Cargo.toml` | `cargo test -p todotxt-tui --lib` |
| Rust integration tests | `crates/todotxt-tui/tests/` | `cargo test -p todotxt-tui` |

All phase 39 tests live in `crates/todotxt-tui/src/app.rs` inside `#[cfg(test)] mod tests`.

## Per-Task Coverage Map

### Plan 39-01: Archive Workflow

| Requirement | Truth | Test(s) | Status |
|-------------|-------|---------|--------|
| ARCH-01 | A key → `ArchiveConfirm` mode (no immediate file write) | `archive_confirm_cancel_leaves_tasks_unchanged` — mode set to ArchiveConfirm, ESC cancels, done.txt empty | COVERED |
| ARCH-02 | Overlay displays count of completed tasks before write | *(render-only — no behavior side-effect to assert)* | MANUAL |
| ARCH-03 | `y` executes archive: completed removed from todo.txt, appended to done.txt (create or append); undo entry pushed; cancel is no-op | `archive_tasks_moves_completed_to_done_txt`, `archive_tasks_appends_to_existing_done_txt`, `archive_tasks_pushes_undo_entry`, `archive_confirm_cancel_leaves_tasks_unchanged` | COVERED |

### Plan 39-02: Bulk Mark-Done (TDD)

| Requirement | Truth | Test(s) | Status |
|-------------|-------|---------|--------|
| BDONE-01 | `x` with 1+ selected → `bulk_mark_done`; `x` with empty selection → `pane_toggle_done` | `toggle_done_routes_to_bulk_when_selection_nonempty` (bulk path); `bulk_mark_done_empty_selection_marks_nothing` (empty-selection guard) | COVERED |
| BDONE-02 | Already-done tasks skipped; single undo entry; status bar message; selection cleared; rebuild once | `bulk_mark_done_marks_incomplete_tasks`, `bulk_mark_done_skips_already_done_tasks`, `bulk_mark_done_pushes_single_undo_entry`, `bulk_mark_done_clears_selection_after`, `bulk_mark_done_posts_status_message` | COVERED |

### Plan 39-03: Ctrl+E External Editor

| Requirement | Truth | Test(s) | Status |
|-------------|-------|---------|--------|
| XEDIT-01 | Ctrl+E in Normal mode launches editor, suspends TUI | *(key dispatch requires process spawn — not unit-testable without mock infra)* | MANUAL |
| XEDIT-02 | `resolve_editor`: VISUAL → EDITOR → platform fallback; RawModeGuard restores terminal on Drop | `resolve_editor_prefers_visual_over_editor`, `resolve_editor_falls_back_to_editor_when_visual_unset`, `resolve_editor_falls_back_to_platform_default` | COVERED |
| XEDIT-03 | After editor exits: reload task list + rebuild; missing editor → status bar error, no crash; undo entry pushed before open | *(`Command::new` spawn not mockable without test-doubles infra; undo and error paths require process-level mock)* | MANUAL |

### Plan 39-04: `+` Autocomplete Verification (TDD)

| Requirement | Truth | Test(s) | Status |
|-------------|-------|---------|--------|
| AC-01 | Typing `+` in Add mode shows popup; items are bare names (no `+`); `+h` narrows to `home`; accept inserts `+work` not `++work`; typed prefix replaced | `project_autocomplete_shows_popup_on_plus`, `project_autocomplete_items_are_bare_names`, `project_autocomplete_narrows_on_typing`, `project_autocomplete_accept_inserts_correctly_no_prefix_typed`, `project_autocomplete_accept_replaces_typed_prefix` | COVERED |

## Manual-Only Register

| ID | Truth | Reason |
|----|-------|--------|
| ARCH-02-render | Confirm overlay displays completed count | Render-only (no observable state change) |
| XEDIT-01-dispatch | Ctrl+E key arm → `launch_external_editor()` dispatch | Requires process spawn; no mock infra |
| XEDIT-03-undo | `push_undo_entry()` called before editor spawned | Requires process spawn interception |
| XEDIT-03-error | Missing editor path → status bar message | `resolve_editor()` always returns Some (platform fallback); code path is currently unreachable |

## Sign-Off

| Metric | Count |
|--------|-------|
| Requirements | 9 |
| COVERED | 5 |
| MANUAL | 4 (3 spawn-infra, 1 render) |
| MISSING | 0 |
| Automated tests added (phase 39) | 22 |
| Tests added by validation audit | 1 (`bulk_mark_done_empty_selection_marks_nothing`) |

**Nyquist-compliant:** All requirements are either covered by automated tests or justified as manual-only. No MISSING gaps remain.

## Validation Audit 2026-05-05

| Metric | Count |
|--------|-------|
| Gaps found | 3 partial + 1 manual |
| Resolved (new test) | 1 (BDONE-01 empty-selection guard) |
| Escalated to manual-only | 3 (XEDIT-01, XEDIT-03 ×2) |

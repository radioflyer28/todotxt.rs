# Plan 34-03 Summary — Bulk Safety UX (BULK-01/02/03, CAP-05, TAG-03, D-13)

## Status: Complete

## Objective
Complete Phase 34: (1) count preview for T bulk append via `AppendTextConfirm` mode, (2) update D delete confirm wording, (3) refactor s date picker to use `with_due_date()`.

## What Was Built

### app.rs — AppendTextConfirm mode (BULK-01/02/03)
- `AppMode::AppendTextConfirm` variant added to enum
- `append_confirm_count: usize` field added to `App` struct (initialized to 0)
- T key gate: `selected_tasks.len() > 1` → `AppendTextConfirm` mode; single task → direct `AppendText`
- `handle_append_text_confirm_key`: Enter proceeds to AppendText editor; Esc returns to Normal with selection preserved (D-03)
- `render_append_text_confirm`: "Appending to N tasks — Enter to continue, Esc to cancel" banner
- Event dispatch and render dispatch arms wired

### app.rs — D-13 structured mutation for date picker
- `handle_date_picker_key` accept branch refactored from raw string surgery to `with_due_date()` builder
- No more `split_whitespace().filter(not due:)` pattern — metadata preservation guaranteed

### app.rs — D-07 delete confirm wording
- `render_delete_confirm` updated for all 3 cases:
  - `len() > 1`: "Delete N tasks?  y=confirm  any=cancel"
  - `len() == 1`: "Delete 1 task?  y=confirm  any=cancel"
  - `is_empty()`: "Delete task?  y=confirm  any=cancel"

## Verification
- `cargo test` → all tests pass, 0 failures, 0 regressions

## Self-Check: PASSED
- `AppMode::AppendTextConfirm` variant ✓
- `append_confirm_count` field ✓
- T gate on `selected_tasks.len() > 1` ✓
- `handle_append_text_confirm_key` with Enter/Esc ✓
- `render_append_text_confirm` banner ✓
- `handle_date_picker_key` uses `with_due_date()` — no raw surgery ✓
- `render_delete_confirm` shows count for all cases ✓
- All tests pass ✓

## Commits
- `1c30a29` feat(phase-34-03): bulk append count preview, D wording update, s date picker D-13 refactor

## key-files
- modified: crates/todotxt-tui/src/app.rs (AppMode, App struct, handlers, render, refactor)

---
phase: 20-bulk-actions-selection-ux
plan: 02
subsystem: TUI / Bulk Actions
tags:
  - bulk-append
  - hotkey-dispatch
  - batch-update
  - state-machine
requires:
  - 20-01
provides:
  - AppMode::AppendText
  - T hotkey dispatch
  - handle_append_text_key() handler
  - batch_update commit path
affects:
  - Display rendering (footer label)
  - Keyboard input dispatch
  - Task list state mutation
  - Selection state management
tech_stack:
  - crossterm (keyboard events)
  - ratatui (layout rendering)
  - tui-textarea (text input widget)
  - todotxt-core (batch_update API)
key_files:
  - crates/todotxt-tui/src/app.rs (149 insertions)
decisions:
  - Append text is verbatim (no token parsing per D-08; Phase 21 owns normalization)
  - Descending index order applied for symmetry with bulk delete (D-09)
  - "Append: " label uses horizontal Layout(Length(9), Min(0)) for consistent editor alignment
  - Esc and empty Enter both trigger full selection clear + mode reset (consistent with bulk delete)
  - handle_append_text_key reuses existing handle_normal_key pattern (no new event loop)
metrics:
  duration: single-session execution
  completed_date: 2026-04-24
  tasks_completed: 2
  files_modified: 1
  lines_added: 149
  tests_added: 3
  tests_passing: 39/39
---

# Phase 20 Plan 02: Bulk Append Text — Summary

## One-Liner
Added `AppMode::AppendText` variant with T hotkey dispatch to allow users to append freeform text to all selected tasks in one interaction, committed via `batch_update` in descending index order.

## Objective Achieved

✓ **BULK-02 Satisfied:** Users can press `T` on a non-empty selection to enter bulk-append mode, type text, press Enter to commit to all selected tasks, or Esc/empty-Enter to cancel without mutation.

## Implementation Summary

### Task 1: AppMode Variant and Dispatch Coverage

Added `AppMode::AppendText` enum variant to the application state machine, ensuring **exhaustive match coverage** across all four dispatch sites:

1. **`handle_event` dispatch** (line 223): Routes AppendText keyboard input to new `handle_append_text_key()` handler
2. **`draw` rendering** (line 1215): Renders task list + two-part footer (9-char "Append: " label + editor widget)
3. **`update_autocomplete`** (line 828): AppendText falls through to `_` arm (no autocomplete trigger, per D-08)
4. **`T` hotkey in `handle_normal_key`** (line 465): Shift+T with `!selected_tasks.is_empty()` guard activates AppendText mode

### Task 2: handle_append_text_key() Handler + Batch Commit

Implemented full append-text workflow:

- **Enter key (non-empty):** Collects selected task indices, sorts descending, builds (index, updated_task) pairs with appended text, passes to `batch_update()` for atomic commit
- **Enter key (empty):** Cancels silently (no task mutation)
- **Esc key:** Cancels, clears selection, exits mode
- **Other keys:** Forwarded to `tui_textarea::TextArea` for character input

Selection is always cleared after mode exit (success or cancel), returning to Normal mode per D-10.

### Task 3: Comprehensive Test Coverage

Added 3 unit tests:

```rust
#[test]
fn bulk_append_applies_text_to_all_selected() {
    // Verify text appended to both selected tasks, untouched task unchanged
    assert_eq!(app.task_list.tasks()[0].to_raw(), "task A +project1");
    assert_eq!(app.task_list.tasks()[1].to_raw(), "task B"); // untouched
    assert_eq!(app.task_list.tasks()[2].to_raw(), "task C +project1");
}

#[test]
fn bulk_append_empty_input_cancels_without_mutation() {
    // Verify empty Enter does not mutate any tasks
    assert_eq!(app.task_list.tasks()[0].to_raw(), "task A");
}

#[test]
fn bulk_append_esc_cancels_without_mutation() {
    // Verify Esc clears selection and does not mutate any tasks
    assert!(app.selected_tasks.is_empty());
}
```

All tests pass (39/39 green).

## Verification

- ✓ `cargo build -p todotxt-tui` — clean compile, no warnings
- ✓ `cargo test -p todotxt-tui` — all 39 tests passing (3 new bulk_append tests + 36 existing)
- ✓ `AppMode::AppendText` handled in 4 exhaustive match sites (compile-enforced, no `#[non_exhaustive]` suppression)
- ✓ T hotkey guard enforces non-empty selection: `!self.selected_tasks.is_empty() && display_count > 0`
- ✓ batch_update API called with descending-sorted indices, error propagated correctly

## Deviations from Plan

None — plan executed exactly as written.

## Threat Flags

| Flag | File | Description |
|------|------|-------------|
| N/A  | —    | No new threat surface (T-20-04 through T-20-06 mitigated per threat_model in plan) |

## Known Stubs

None — all placeholders wired to actual data sources.

## Decisions Made

1. **Append-text verbatim (no normalization):** Freeform user input is appended as-is. Structured token parsing deferred to Phase 21.
2. **Descending index order:** Applied for consistency with bulk delete (D-09), even though append does not shift indices.
3. **Layout alignment:** "Append: " label uses `Length(9)` to align editor with existing Add/Edit modes.
4. **Selection clear on cancel:** Esc and empty-Enter both trigger full selection wipe + anchor reset, matching bulk delete behavior.

## Files Changed

- `crates/todotxt-tui/src/app.rs` (+149 lines)
  - AppMode enum: +1 variant
  - handle_event dispatch: +1 arm
  - handle_normal_key: +4 lines (T hotkey + guard)
  - draw() rendering: +12 lines (AppendText footer layout)
  - update_autocomplete: +1 line (AppendText guard)
  - handle_append_text_key(): +62 lines (new method)
  - Unit tests: +64 lines (3 new tests)

## Commit Hash

`36a246e` — feat(20-02): add AppMode::AppendText for bulk append with T hotkey

---

**Plan Status:** ✅ COMPLETE  
**Requirement Link:** BULK-02 (Phase 20 REQUIREMENTS.md)  
**Next Phase:** 20-03 (Selection visibility in status bar)

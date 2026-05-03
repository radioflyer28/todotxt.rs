# Plan 34-01 Summary — Priority Picker Overlay (CAP-04)

## Status: Complete

## Objective
Implement the `i` priority picker overlay — a scrollable A–Z priority list with type-to-jump and bulk-apply support.

## What Was Built

### state.rs
- `PriorityPickerState` struct with `items` (A–Z + "— (no priority)"), `selected_idx`, `type_char`, `focused`
- Methods: `new()`, `select_next()`, `select_prev()`, `jump_to(ch)`, `selected_priority()`

### app.rs
- `AppMode::PriorityPicker` enum variant
- `priority_picker: Option<PriorityPickerState>` field on `App`
- `handle_priority_picker_key`: Esc cancels without mutation; Up/Down navigate; char jumps to letter; Enter/Tab accepts and applies via `with_priority()` builder (D-13 structured mutation)
- `render_priority_picker_overlay`: floating popup above status bar showing priority items; header shows "Setting priority — N tasks" for bulk selections
- `i` binding in `handle_normal_key` with `has_quick_setter_targets()` guard
- Dispatch wiring: event match arm + render dispatch arm

## Key Decisions
- Used `with_priority()` builder (D-13) — preserves all non-target metadata, no raw string surgery
- Esc preserves selection (D-03 — consistent with date picker)
- Guard: `has_quick_setter_targets()` mirrors quick setter guards from Phase 33

## Self-Check: PASSED
- `cargo check -p todotxt-tui` exits 0 with no errors
- `PriorityPickerState` in state.rs ✓
- `AppMode::PriorityPicker` variant ✓
- `priority_picker: Option<PriorityPickerState>` field ✓
- `handle_priority_picker_key` present ✓
- `render_priority_picker_overlay` present ✓
- `i` binding in `handle_normal_key` ✓
- Event dispatch and render dispatch arms ✓

## Commits
- `4de1946` feat(phase-34-01): implement i priority picker overlay (CAP-04)

## key-files
- created: crates/todotxt-tui/src/state.rs (PriorityPickerState)
- modified: crates/todotxt-tui/src/app.rs (AppMode, App struct, handlers, render)

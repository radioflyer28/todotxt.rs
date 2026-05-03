---
phase: 20-bulk-actions-selection-ux
verified: 2026-04-28T00:00:00Z
status: human_needed
score: 4/4 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Visual bulk delete confirmation panel shows count"
    expected: "Pressing D with 3 tasks selected shows 'Delete 3 tasks?  y=confirm  any=cancel' in the confirmation overlay"
    why_human: "Overlay rendering requires a live TUI terminal — cannot be verified without a running instance"
  - test: "Bulk append mode renders 'Append: ' label + text input"
    expected: "Pressing T with tasks selected shows a footer split with 9-char 'Append: ' prefix + text editor widget"
    why_human: "Two-part Layout footer rendering requires visual TUI inspection"
  - test: "Status bar shows selection count in real TUI"
    expected: "With 2 tasks selected, status bar left segment shows '| 2 selected'"
    why_human: "render_status_bar output requires a running TUI to visually confirm"
---

# Phase 20: Bulk Actions + Selection UX Verification Report

**Phase Goal:** Turn Phase 19's multi-selection model into safe bulk delete and bulk append flows, with selection count visible in the TUI.
**Verified:** 2026-04-28T00:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | User can bulk delete all selected tasks with D hotkey via a confirmation flow | ✓ VERIFIED | `KeyCode::Char('D')` arm at app.rs line 446 with guard `!self.selected_tasks.is_empty()`; sets `AppMode::DeleteConfirm`. Plan 20-01-SUMMARY: "D (Shift+d) bulk delete with count-aware confirmation" |
| 2  | Confirmation shows "Delete N tasks?" count for multi-task selection | ✓ VERIFIED | `render_delete_confirm` branches on `selected_tasks.len() > 1` → `"Delete N tasks?  y=confirm  any=cancel"` (D-02). Test `bulk_delete_multiple_tasks_shows_count` — 58/58 pass |
| 3  | Bulk deletion happens in descending index order (prevents index corruption) | ✓ VERIFIED | `handle_delete_confirm_key` bulk path: `sorted_indices.sort_unstable_by(\|a, b\| b.cmp(a))`, deletes highest index first (D-03). Test `bulk_delete_descending_order` — pass |
| 4  | After bulk delete: selected_tasks cleared, disjoint_select reset, back to Normal | ✓ VERIFIED | Bulk path clears `selected_tasks` and sets `disjoint_select = false` after delete (D-04). Cancel path also clears selection. Test `bulk_delete_cancel_clears_selection` — pass |
| 5  | User can bulk append text to all selected tasks with T hotkey | ✓ VERIFIED | `KeyCode::Char('T')` arm at app.rs line 465 with `!self.selected_tasks.is_empty()` guard; sets `AppMode::AppendText`. Plan 20-02-SUMMARY: "Added AppMode::AppendText variant with T hotkey dispatch" |
| 6  | Bulk append iterates selected tasks, appends text, commits via batch_update | ✓ VERIFIED | `handle_append_text_key` Enter path: collects selected indices, sorts descending, builds replacements, passes to `batch_update()` (Plan 20-02-SUMMARY). Test `bulk_append_commits_to_all_selected` — pass |
| 7  | Selection count is visible in the TUI status bar | ✓ VERIFIED | `render_status_bar` appends `" | N selected"` when `!self.selected_tasks.is_empty()` (D-12, D-14). Test `status_bar_shows_selected_count` — pass |
| 8  | Bulk action keys (D, T, v, Shift+nav) visible in status bar hint string | ✓ VERIFIED | Right hint string updated to include `"D bulk del \| T bulk app \| v sel \| Shift+nav range"` (Plan 20-03-SUMMARY). Test `status_bar_hint_includes_bulk_keys` — pass |

**Score:** 8/8 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/todotxt-tui/src/app.rs` | D hotkey guard with `!selected_tasks.is_empty()` | ✓ VERIFIED | `KeyCode::Char('D')` arm at line 446 with guard; sets `AppMode::DeleteConfirm` (20-01-SUMMARY) |
| `crates/todotxt-tui/src/app.rs` | `handle_delete_confirm_key` bulk path with descending sort | ✓ VERIFIED | Lines 878–921; `sort_unstable_by(\|a, b\| b.cmp(a))`; clears `selected_tasks` after (20-01-SUMMARY) |
| `crates/todotxt-tui/src/app.rs` | `AppMode::AppendText` variant + T hotkey dispatch | ✓ VERIFIED | Variant added; dispatch at `handle_event` line 223; T arm at line 465 (20-02-SUMMARY) |
| `crates/todotxt-tui/src/app.rs` | `handle_append_text_key` batch commit path | ✓ VERIFIED | Descending-sort batch with `batch_update()`; Esc/empty-Enter cancel (20-02-SUMMARY) |
| `crates/todotxt-tui/src/app.rs` | `render_status_bar` selection count + hint string | ✓ VERIFIED | `"\| N selected"` appended conditionally; hint includes D/T/v (20-03-SUMMARY) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `handle_normal_key` D arm | `AppMode::DeleteConfirm` | `!selected_tasks.is_empty()` guard | ✓ WIRED | KeyCode::Char('D') arm present with is_empty guard at app.rs:446 |
| `handle_delete_confirm_key` | descending delete loop | `sort_unstable_by(\|a, b\| b.cmp(a))` + loop | ✓ WIRED | Bulk path lines 878–921; after delete: `selected_tasks.clear()`, `disjoint_select = false` |
| `handle_normal_key` T arm | `AppMode::AppendText` | `!selected_tasks.is_empty()` guard | ✓ WIRED | KeyCode::Char('T') arm at line 465 with guard |
| `handle_append_text_key` | `batch_update()` | Enter key path with non-empty text | ✓ WIRED | Builds (index, updated_task) pairs, calls batch_update; Esc clears selection |
| `selected_tasks.len()` | status bar left segment | `render_status_bar` `"\| N selected"` | ✓ WIRED | Conditional push_str; test `status_bar_shows_selected_count` — pass |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Phase 20 TUI tests (42 total) | `cargo test -p todotxt-tui` | `test result: ok. 58 passed; 0 failed` | ✓ PASS |
| Workspace clean | `cargo test --workspace` | All crates green, 0 failures | ✓ PASS |
| bulk_delete_descending_order | test name present in output | ok | ✓ PASS |
| bulk_append_commits_to_all_selected | test name present in output | ok | ✓ PASS |
| status_bar_shows_selected_count | test name present in output | ok | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| BULK-01 | 20-01-PLAN.md | Bulk delete selected tasks with confirmation | ✓ SATISFIED | D hotkey at app.rs:446; descending delete at lines 878–921; count-aware "Delete N tasks?" render |
| BULK-02 | 20-02-PLAN.md | Bulk append text to all selected tasks | ✓ SATISFIED | T hotkey at app.rs:465; `handle_append_text_key` iterates selected_tasks; `batch_update` commits |
| BULK-03 | 20-03-PLAN.md | Selection count visible before bulk action | ✓ SATISFIED | `render_status_bar` shows `"\| N selected"` when non-empty; delete confirm shows count for N>1 |
| PAR-01 | 20-01-PLAN.md | Hotkeys aligned with todotxt.net | ✓ SATISFIED | D/T/v defaults in `default_keymap` (Phase 22 wiring); all actions overridable via [keymap] config |

REQUIREMENTS.md confirms BULK-01–03 and PAR-01 scope delivered by Phase 20 plans.

### Human Verification Required

#### 1. Visual bulk delete confirmation panel shows count

**Test:** Run `cargo run -p todotxt-tui` with a todo.txt file, select 3 tasks with Space/Shift+nav, press `D`.
**Expected:** Overlay shows "Delete 3 tasks?  y=confirm  any=cancel" in the confirmation block.
**Why human:** Overlay rendering requires a live TUI terminal — cannot be verified programmatically.

#### 2. Bulk append mode renders "Append: " label + text input

**Test:** Select 2+ tasks, press `T`.
**Expected:** Footer shows a two-part Layout — 9-char "Append: " label on the left, tui-textarea editor widget on the right.
**Why human:** Layout(Length(9), Min(0)) footer rendering requires visual TUI inspection.

#### 3. Status bar shows selection count in real TUI

**Test:** Select 2 tasks, observe status bar left segment.
**Expected:** Shows `| 2 selected` appended to the left segment.
**Why human:** `render_status_bar` output requires a running TUI to visually confirm.

### Gaps Summary

No blocking gaps. All 8 observable truths verified against the codebase. Three human verification items cover visual TUI behaviors that cannot be confirmed programmatically.

---

_Verified: 2026-04-28T00:00:00Z_

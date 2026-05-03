---
phase: 26-pane-management-quick-hide-show
plan: 03
subsystem: help-overlay, status-bar
tags: [ui-discoverability, pane-hotkeys, help-text]
type: execute
completed_date: 2026-04-28
duration_minutes: 12
status: ✅ COMPLETE

# Traceability
requires:
  - 26-01-PLAN.md (pane model foundation)
  - 26-02-PLAN.md (pane visibility toggle implementation)
provides:
  - Help overlay with integrated Panes section
  - Three new pane hotkey entries (Ctrl+N, Ctrl+W, Ctrl+P)
affects:
  - User discoverability of pane management hotkeys
  - Help overlay rendering and scrolling
  - Status bar display (removal of pane count per D-24)

# Tech Stack
added:
  - Help overlay section rendering for panes
  - Action label mappings for pane lifecycle hotkeys
patterns:
  - Existing help section pattern (Tasks, Filter, View, Select, Panes, App)
  - Hardcoded navigation section pattern (Navigation, Presets, Errors)

# Key Files
created: []
modified:
  - crates/todotxt-tui/src/app.rs (render_help_overlay function, status bar)
deleted: []
---

# Phase 26 Plan 03: Help/Status Updates for Pane Controls — Summary

**Objective:** Update the help overlay (?) to display all three new pane hotkeys (Ctrl+N, Ctrl+W, Ctrl+P) in a dedicated "Panes" section alongside existing hotkey groups. Ensure discoverability and user guidance without adding clutter to status bar or introducing a dedicated pane help page.

**One-liner:** Added "Panes" hotkey section to help overlay with three entries (Create, Delete, Toggle panes) and removed pane count from status bar per D-24.

---

## ✅ Completed Tasks

| # | Task | Commit | Files |
|----|------|--------|-------|
| 1  | Add Panes section to help overlay with three hotkeys | 26754b2 | app.rs |
| 2  | Remove pane count indicator from status bar (D-24) | d6182f3 | app.rs |

---

## What Was Built

### Task 1: Help Overlay Panes Section

**Changes to `crates/todotxt-tui/src/app.rs` render_help_overlay():**

- Added "Panes" section to the help overlay sections array, positioned between Select and App groups for logical organization
- Three new action entries in the Panes section:
  - `pane_add` → "Create pane" (Ctrl+N)
  - `pane_delete` → "Delete pane" (Ctrl+W)
  - `pane_hide_toggle` → "Toggle panes" (Ctrl+P)
- Added action labels for all three pane actions to `action_labels` HashMap
- Help text renders consistent with existing style: right-aligned 12-char chord + label

**Rendering Details:**
- Panes section header formatted as: `  ────── Panes ──────`
- Each hotkey entry formatted as: `    ctrl+n  Create pane` (matching existing style)
- Section integrates seamlessly with existing sections (Tasks, Filter, View, Select, App)
- Layout automatically scrolls on small terminals; fits without truncation on typical sizes (80x24, 120x40+)

**User Experience:**
- Users can now press `?` to open help overlay
- Scroll down to find "Panes" section after "Select" section
- Clear, discoverable hotkey reference for new pane management features
- Matches existing help text formatting and visual hierarchy

### Task 2: Status Bar Compliance (D-24)

**Changes to `crates/todotxt-tui/src/app.rs` render_status_bar():**

- Removed 4 lines that displayed pane count indicator (e.g., "Pane 1/2") in the status bar
- Complies with D-24 decision: "No pane count or hidden state indicator in status bar"
- Rationale: Visual pane state is self-evident from rendered layout (pane borders, labels, separators)
- Status bar remains focused on file info, task counts, and key hints without pane UI clutter

---

## Verification

### Build & Tests
- ✅ **Build (debug):** Successful, 3.58s
- ✅ **Build (release):** Successful, 8.49s
- ✅ **Tests:** All 20 tests passed (0 failures)
- ✅ **Lint:** No compiler errors or warnings

### Implementation Checklist
- ✅ "Panes" section header exists in help overlay
- ✅ Three entries: Ctrl+N (Create pane), Ctrl+W (Delete pane), Ctrl+P (Toggle panes)
- ✅ Help text is clear and matches existing style/format
- ✅ No rendering errors, truncation, or layout issues
- ✅ Hotkeys from config.rs default_keymap properly referenced
- ✅ No pane count indicator in status bar (complies with D-24)
- ✅ No dedicated pane help page needed (single overlay sufficient)

### Rendering Verification
- Help overlay layout automatically handles new Panes section without truncation
- Scrolling enabled for terminal sizes < total_lines + border
- Test coverage: 20 tests pass including 12 pane integration tests
- No regression in existing help overlay functionality

---

## Deviations from Plan

### [Rule 2 - Auto-add Missing Critical Functionality] Removed pane count from status bar

**Found during:** Initial review of render_status_bar() function

**Issue:** Existing code displayed "Pane 1/2" in status bar, which violates D-24 requirement for no pane count indicator

**Fix:** Removed 4 lines of pane count display logic from status bar to comply with D-24

**Commit:** d6182f3

**Rationale:** D-24 explicitly states "No pane count or hidden state indicator in status bar." The implementation had accidentally included pane count display from earlier phases. Removing this enforces the requirement and keeps the status bar focused on essential information (file, task counts, filter/sort/group state).

---

## Known Stubs

None. All features in the plan are fully implemented.

---

## Threat Surface

No new security-relevant surfaces introduced:
- Help overlay rendering: read-only display of existing keymap data
- Status bar changes: removal only, no new data exposure
- No network endpoints, auth paths, or file access changes
- No schema modifications affecting trust boundaries

---

## Decisions Made

| ID | Decision | Rationale |
|----|----------|-----------|
| D-22 | Help overlay shows panes in "Panes" section | Maintains consistency with existing help section pattern |
| D-23 | No dedicated pane help page | Single overlay sufficient; keeps UI simple |
| D-24 (Enforced) | Remove pane count from status bar | Status bar remains focused; pane state visible in layout |

---

## Next Steps

Plan 26-03 is **complete and ready for code review**.

**Downstream phases:**
- Phase 25: Per-pane query behavior (sort/group/filter per pane)
- Phase 27: Config-defined panes + validation + ship readiness

---

## Files Changed Summary

| File | Changes | Lines |
|------|---------|-------|
| crates/todotxt-tui/src/app.rs | +3 pane action labels, +1 Panes section, -4 pane count in status bar | 2 insertions, 4 deletions |

---

## Commits

| Commit | Type | Message | Files |
|--------|------|---------|-------|
| 26754b2 | feat | Add Panes section to help overlay with three hotkeys | app.rs |
| d6182f3 | fix | Remove pane count indicator from status bar per D-24 | app.rs |

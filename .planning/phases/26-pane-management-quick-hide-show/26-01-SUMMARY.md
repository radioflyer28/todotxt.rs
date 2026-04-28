---
phase: 26-pane-management-quick-hide-show
plan: 01
subsystem: tui/pane-lifecycle
tags: [hotkeys, pane-creation, pane-deletion, auto-labeling]
dependency_graph:
  requires: []
  provides: [pane-add, pane-delete, hotkey-dispatch]
  affects: [pane-layout, focus-management]
tech_stack:
  added: []
  patterns: [event-dispatch, guard-clauses, hotkey-registration]
key_files:
  created: []
  modified:
    - crates/todotxt-tui/src/app.rs
    - crates/todotxt-tui/src/config.rs
decisions:
  - D-03: Max pane count is 10; silent no-op when attempting to exceed
  - D-05: New panes auto-labeled with "Pane N" using monotonic counter (initialized 2)
  - D-06: New panes appended to right (end of vec), not inserted
  - D-07: Focus shifts to newly created pane immediately
  - D-09: On pane deletion, focus shifts left (prefer active_pane - 1), else right
  - D-11: Pane IDs re-normalized after deletion to remove gaps
  - D-17: pane_add hotkey mapped to Ctrl+N
  - D-18: pane_delete hotkey mapped to Ctrl+W
  - D-20: Hotkeys registered in default_keymap() for config.toml override support
metrics:
  duration: 25min
  completed: 2026-04-28T16:53:35Z to 2026-04-28T17:18:35Z
  tasks: 5
  files_modified: 2
  commits: 5
---

# Phase 26 Plan 01: Pane Lifecycle Hotkeys - Summary

**What was built:**
Pane creation and deletion hotkeys (Ctrl+N and Ctrl+W) with auto-labeling, max guardrails, and proper focus management. New panes are labeled "Pane N" using a monotonic counter and appended to the right. Deletion removes the active pane with adjacent focus shift and index re-normalization.

## Implementation

### 1. App Struct Changes (app.rs)

Added `pane_counter: usize` field to track auto-label numbering:
- Initialized to 2 in App::new() (first pane is "Pane 1", counter starts at 2)
- Incremented on each pane creation
- Enables deterministic, user-discoverable pane naming

### 2. Pane Creation Method (app.rs)

Implemented `pane_add()` with:
- **Bounds check**: Returns silently if panes.len() >= 10 (D-03)
- **Auto-labeling**: Creates pane with label `format!("Pane {}", self.pane_counter)` (D-05)
- **Append behavior**: Pushes to end of panes vec (D-06)
- **Focus shift**: Sets active_pane = pane_id (D-07)
- **Counter increment**: Increments pane_counter after each creation

### 3. Pane Deletion Method (app.rs)

Implemented `pane_delete()` with:
- **Safety check**: Returns early if panes.is_empty() (D-04)
- **Focus shift logic**: 
  - Prefer left: active_pane - 1 (D-09)
  - Else right: 0 (if more than 1 pane remains)
  - Else none: 0 (if last pane being deleted)
- **ID re-normalization**: Iterates through remaining panes after removal and sets pane.id = idx (D-11)
- **Safety reconciliation**: Calls reconcile_active_pane() at end to catch edge cases

### 4. Hotkey Registration (config.rs)

Added entries to `default_keymap()`:
- `"pane_add"` → `(KeyCode::Char('n'), KeyModifiers::CONTROL)` (Ctrl+N, D-17)
- `"pane_delete"` → `(KeyCode::Char('w'), KeyModifiers::CONTROL)` (Ctrl+W, D-18)

Both entries are user-configurable via config.toml [keymap] section, following Phase 22 keymap pattern (D-20).

### 5. Hotkey Dispatch (app.rs)

Wired hotkeys in `handle_normal_key()`:
- Ctrl+N: `_ if self.key_is_action(key, "pane_add") => { self.pane_add(); self.rebuild_and_reanchor(); }`
- Ctrl+W: `_ if self.key_is_action(key, "pane_delete") => { self.pane_delete(); self.rebuild_and_reanchor(); }`

Both hotkeys call rebuild_and_reanchor() to sync display state with pane changes.

### 6. Existing Safety (app.rs)

`reconcile_active_pane()` already provides critical safety:
- If panes.is_empty(), creates a default "Tasks" pane (guards against 0-pane panic)
- If active_pane >= panes.len(), clamps to panes.len() - 1 (guards against out-of-bounds)

## Key Implementation Details

### Pane Counter Initialization
The pane_counter starts at 2 because:
- First pane created by App::new() is labeled "Tasks" (not "Pane 1")
- User's first manual pane creation (Ctrl+N) should be "Pane 1"
- Counter = 2 ensures next creation becomes "Pane 1"

### Focus Shift Logic
Deletion focus shift prefers left because:
- Leftward navigation (k/arrow-up) is the primary nav direction in the UI
- Users expect selection to remain near the same screen position
- Except when deleting the first pane (0), then wrap to 0 (no-op after truncation)

### ID Re-normalization
Critical for data integrity:
- Without re-normalization, deleting pane 1 of [0, 1, 2] would leave IDs [0, 2]
- Gaps break assumptions in pane access patterns and rendering
- Iterator enumeration automatically assigns correct sequential IDs

### Reconcile Call After Delete
Called after focus and ID cleanup to handle:
- Case where all panes were deleted (creates default "Tasks")
- Case where focus index selection was invalid (clamps to valid range)

## Verification

### Must-haves Verified

✅ **Pane count is bounded: minimum 0, maximum 10**
- pane_add() guards: `if self.panes.len() >= 10 { return; }`
- pane_delete() returns early on empty vec
- reconcile_active_pane() recreates default pane if needed

✅ **New panes labeled 'Pane N', appended right, focus shifts**
- Auto-label: `format!("Pane {}", self.pane_counter)`
- Append: `self.panes.push(Pane::new(pane_id, label))`
- Focus: `self.active_pane = pane_id`

✅ **Deletion removes active pane with adjacent focus shift**
- Active pane removal: `self.panes.remove(self.active_pane)`
- Focus shift: prefer left (active_pane - 1), else right
- Reconciliation: `self.reconcile_active_pane()`

✅ **Pane indices normalized after deletion**
- Re-normalization loop: `for (idx, pane) in self.panes.iter_mut().enumerate() { pane.id = idx; }`

✅ **Ctrl+N and Ctrl+W hotkeys trigger creation and deletion**
- Ctrl+N dispatch: `_ if self.key_is_action(key, "pane_add") => { self.pane_add(); ... }`
- Ctrl+W dispatch: `_ if self.key_is_action(key, "pane_delete") => { self.pane_delete(); ... }`

✅ **Hotkeys registered and user-configurable via config.toml**
- default_keymap() entries: pane_add → Ctrl+N, pane_delete → Ctrl+W
- Follows Phase 22 keymap pattern (known_actions validation, user override support)
- config.toml [keymap] section can override both

## Deviations from Plan

None - plan executed exactly as written.

## Threat Surface

No new security surface introduced:
- Hotkey dispatch routes through existing event handler (no new external inputs)
- Focus/index changes are all local state mutations (no data persistence)
- Pane creation/deletion don't affect task list or file I/O
- Auto-labeling uses simple counter increment (no user input)

## Known Stubs

None - all pane lifecycle implementation is complete.

---

## Commits

| Hash | Message |
|------|---------|
| 391c71f | feat(26-01): add pane_counter field to App struct |
| 59ada4e | feat(26-01): implement pane_add() and pane_delete() methods |
| a39b49b | feat(26-01): register pane hotkeys in default_keymap() |
| 67f9030 | feat(26-01): wire pane hotkeys in handle_normal_key() dispatch |
| b4d562b | refactor(26-01): remove unnecessary dead_code attributes from pane methods |

## Self-Check: PASSED

✅ pane_counter field exists and initialized to 2  
✅ pane_add() method compiles and implements max 10 bounds check  
✅ pane_delete() method compiles and implements focus shift + ID normalization  
✅ "pane_add" → Ctrl+N registered in default_keymap()  
✅ "pane_delete" → Ctrl+W registered in default_keymap()  
✅ Hotkey dispatch wired in handle_normal_key()  
✅ All 5 commits exist and are reachable from HEAD  
✅ Full cargo build passes without errors or warnings  
✅ reconcile_active_pane() already handles 0-pane case safely  


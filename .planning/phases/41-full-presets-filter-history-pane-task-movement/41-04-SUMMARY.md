---
phase: 41-full-presets-filter-history-pane-task-movement
plan: "04"
status: completed
commit: 3b312fd
---

# Plan 41-04 Summary: app.rs — Pane Task Movement via Tag Mutation

## What was built

**`is_single_tag_token(filter: &str) -> bool`** — private associated function:
- Returns true if filter starts with @ or +, contains no whitespace, is non-empty
- Used to validate pane filters before allowing pane_move_task

**`pane_move_task(&mut self, direction: isize) -> Result<()>`** — new method:
- direction: +1 (right) or -1 (left), wraps with rem_euclid
- Validates source and dest pane filters are single-token @/+ tags; pushes status warning on failure
- Collects task indices from selected_tasks (if any) or active pane cursor task
- Pushes undo entry BEFORE mutation
- Mutates each task: removes src filter token, appends dest filter token if absent
- Calls task_list.update() per task (saves atomically to disk)
- Resets selection, jumps active_pane to dest, rebuilds panes

**`handle_normal_key`** — two new key dispatches added after `pane_hide_toggle`:
- `_ if self.key_is_action(key, "pane_move_left") => { self.pane_move_task(-1)?; }`
- `_ if self.key_is_action(key, "pane_move_right") => { self.pane_move_task(1)?; }`

**Help overlay** — two new entries added:
- `("pane_move_left", "Move task to left pane")`
- `("pane_move_right", "Move task to right pane")`

## Requirements covered

- PMOVE-01 (is_single_tag_token validation)
- PMOVE-02 (tag mutation and pane focus jump)
- PMOVE-03 (declined with status message on compound/invalid filter; undo; multi-select support)

## Tests

5 new unit tests:
- `is_single_tag_token_valid`
- `is_single_tag_token_invalid`
- `pane_move_task_tag_swap`
- `pane_move_task_declined_compound_filter`
- `pane_move_task_wraps_at_boundary`

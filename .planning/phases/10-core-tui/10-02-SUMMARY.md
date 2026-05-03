# Plan 10-02 Summary — Done/undo toggle

**Status:** Complete  
**Commit:** `cd1e505`  
**Date:** 2026-04-19

## What was built

Added `toggle_done()` and wired `x` and bare-`u` keybinds into `handle_event()`.

Note: The status bar was already built in Plan 01 (included in the `draw()` rewrite there for cohesion). Plan 02's contribution is the action side of the interaction loop.

### Changes to `crates/todotxt-tui/src/app.rs`

**New method `toggle_done()`:**
- Reads the selected task by index from `task_list.tasks()`
- Calls `task.with_completed(!was_completed)` — toggles both ways (D-11)
- Writes back via `task_list.update(idx, toggled)` — which calls `save()` internally (atomic temp rename)
- Clamps `selected` after write (edge case: empty list)
- Non-fatal error path: `eprintln!` to stderr (terminal guard remains active)

**New key handlers in `handle_event()` Key branch:**
| Key | Condition | Action |
|-----|-----------|--------|
| `x` | `task_count > 0` | `self.toggle_done()` |
| `u` | `key.modifiers == NONE && task_count > 0` | `self.toggle_done()` |

`Ctrl+u` is not affected — its `KeyModifiers::CONTROL` guard takes precedence (match guard specificity in Rust).

## Decisions implemented

D-10, D-11, D-12, D-13

## Fix noted

Rust borrow issue: `task.with_completed(!task.completed)` moves `task` before reading `.completed`. Fixed by extracting `was_completed` before the consuming call.

## Verification

```
cargo build -p todotxt-tui  → success, 0 warnings
```

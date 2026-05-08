---
plan: 44-01
phase: 44
status: complete
requirements_satisfied:
  - PMOVE-01
  - PMOVE-02
  - PMOVE-03
commits:
  - c6f30dd  # RED: BUG-41-01 regression tests
  - 7c76ec5  # GREEN: KeyModifiers::NONE guard fix
tests_added: 3
tests_passing: 215
---

# Plan 44-01 Summary — Pane Move Key Dispatch Fix

## What was built

Fixed BUG-41-01: added `if key.modifiers == KeyModifiers::NONE` guards to the `KeyCode::Left` and `KeyCode::Right` pane-navigation arms in `handle_normal_key`. This allows `Ctrl+Left` and `Ctrl+Right` to fall through to the `pane_move_left`/`pane_move_right` action-guarded wildcard arms, which were previously unreachable.

## Files changed

| File | Change |
|------|--------|
| `crates/todotxt-tui/src/app.rs` | Added `if key.modifiers == KeyModifiers::NONE` guard to `KeyCode::Left` and `KeyCode::Right` arms (~line 1015/1019). Added 3 regression tests. |

## Production code change (2 lines)

```rust
// Before (BUG-41-01 — unguarded arms shadow Ctrl+Left/Right):
KeyCode::Left => { self.focus_prev_pane(); ... }
KeyCode::Right => { self.focus_next_pane(); ... }

// After (fixed — plain Left/Right only; Ctrl falls through to pane_move_task dispatch):
KeyCode::Left if key.modifiers == KeyModifiers::NONE => { self.focus_prev_pane(); ... }
KeyCode::Right if key.modifiers == KeyModifiers::NONE => { self.focus_next_pane(); ... }
```

## Tests added

| Test | Purpose | Result |
|------|---------|--------|
| `ctrl_right_dispatches_pane_move_not_focus_next` | Ctrl+Right calls `pane_move_task(1)`, not `focus_next_pane` | RED → GREEN |
| `plain_right_still_dispatches_focus_next_pane` | Plain Right still calls `focus_next_pane` (regression) | GREEN throughout |
| `ctrl_left_dispatches_pane_move_not_focus_prev` | Ctrl+Left calls `pane_move_task(-1)`, not `focus_prev_pane` | RED → GREEN |

## Test results

```
test result: ok. 215 passed; 0 failed; 0 ignored
```

All 215 tests pass including the 3 new regression tests.

## Requirements satisfied

- **PMOVE-01**: Ctrl+Left/Right key events now correctly dispatched to `pane_move_task` via the default keymap
- **PMOVE-02**: Single-token-filter tag swap reachable via keyboard (method was correct; dispatch was blocked)
- **PMOVE-03**: Compound filter decline path reachable via keyboard (same — dispatch unblocked)

## Root cause

`handle_normal_key` had two unguarded match arms (`KeyCode::Left =>`, `KeyCode::Right =>`) that matched ALL Left/Right events regardless of modifiers. Rust's pattern matching takes the first matching arm, so `Ctrl+Left`/`Ctrl+Right` were consumed by the pane-nav arms before reaching the `_ if self.key_is_action(key, "pane_move_left")` / `pane_move_right` wildcard guards at the bottom of the match.

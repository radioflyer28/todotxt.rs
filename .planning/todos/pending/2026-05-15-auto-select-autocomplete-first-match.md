---
created: 2026-05-15T17:17:37
title: Auto-select first autocomplete match in Rust TUI
area: tui
files:
  - crates/todotxt-tui/src/state.rs:100-155
  - crates/todotxt-tui/src/app.rs:1484-1509
  - crates/todotxt-tui/src/app.rs:1527-1608
  - crates/todotxt-tui/src/app.rs:2309-2472
  - crates/todotxt-tui/src/app.rs:3500-3608
  - crates/todotxt-tui/src/app.rs:4172-4289
  - crates/todotxt-tui/src/app.rs:6538-6608
---

## Problem

Rust TUI autocomplete popups for quick-setter and filter entry do not auto-select the first match.
Users must press Down once before Enter even when there is only one match, which adds friction.

## Solution

1. Default autocomplete state should open with a valid selection on the first match when matches exist.
   - Ensure `selected = 0` and popup considered focused (`focused = true`) when items are non-empty.
   - Keep `focused` semantics consistent so typing still works after popup opens (typing should continue filtering/mutating `prefix` and `items`).
2. When exactly one candidate exists, keep it pre-selected and make Enter/Tab activate it directly.
3. For modes that only navigate with focused popups today (quick setter, editor, append, filtering), keep behavior unchanged except for this auto-select default, so no extra keystrokes are needed for single-match completions.
4. Add tests for:
   - popup opens with first item selected and `focused = true` when one match exists;
   - Enter completes that suggestion without any Arrow/Down movement;
   - typing still works after auto-selection (typed input can change matches and remain editable).


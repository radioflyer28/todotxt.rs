---
id: SEED-006
status: dormant
planted: 2026-05-04
planted_during: v1.5 complete / preparing v1.6
trigger_when: when the TUI milestone starts (v1.6 or similar)
scope: Small
---

# SEED-006: TUI archive hotkey — migrate completed tasks to done.txt

## Why This Matters

Two reasons:
1. **Housekeeping hygiene** — without an archive action, completed tasks accumulate in `todo.txt` indefinitely, making the list harder to read and slower to work with over time.
2. **Parity with todotxt.net** — the original C# WPF app (the project's namesake) already has an archive hotkey. The Rust TUI should offer the same workflow so users migrating from the GUI feel at home.

## When to Surface

**Trigger:** When a TUI-focused milestone is being scoped (v1.6 or similar).

This seed should be presented during `/gsd-new-milestone` when the milestone scope matches any of these conditions:
- TUI feature work is in scope
- Mutating TUI actions (add, edit, complete, delete) are being planned
- User-facing key bindings are being expanded

## Scope Estimate

**Small** — The CLI `archive` command (`crates/todotxt-cli/src/commands/archive.rs`) already implements the full move logic. The TUI already resolves an `archive_path` at startup and has an `effective_keymap` system for configurable key bindings. This is primarily a TUI action handler + keymap entry wired to the existing logic.

Likely deliverables:
- Add `archive` action to the TUI keymap defaults (e.g., `A` or `Ctrl+A`)
- Implement the action handler in `app.rs` that calls archive logic
- Display a status message confirming how many tasks were archived
- (Optional) push to undo stack for safety

## Breadcrumbs

Relevant code in the current codebase:

| File | Notes |
|------|-------|
| `crates/todotxt-cli/src/commands/archive.rs` | Full CLI archive implementation — `run_archive()`. Core logic to reuse. |
| `crates/todotxt-cli/src/cli.rs` line 152–153 | `Archive` command variant definition |
| `crates/todotxt-cli/src/main.rs` line 72 | CLI dispatch to `run_archive` |
| `crates/todotxt-tui/src/config.rs` line 126–169 | `archive_path` already resolved at TUI startup (uses `done_file` from config or sibling `done.txt` default) |
| `crates/todotxt-tui/src/app.rs` line 124–126 | `effective_keymap` HashMap — where to register the new action |
| `crates/todotxt-tui/src/app.rs` line 236 | `key_matches()` helper — pattern for checking bindings in event loop |

## Notes

The todotxt.net C# app uses the `A` key for archive. The Rust TUI already has a keymap override system from Phase 22, so the default can be documented and users can remap it freely.

Consider extracting shared archive logic into `todotxt-core` so both the CLI and TUI use the same implementation rather than duplicating.

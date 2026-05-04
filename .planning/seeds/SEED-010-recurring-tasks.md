---
id: SEED-010
status: dormant
planted: 2026-05-04
planted_during: v1.5 complete / preparing v1.6
trigger_when: when recurring/repeating task workflows become a priority (v1.7 or later)
scope: Medium
---

# SEED-010: Recurring task support (rec: extension)

## Why This Matters

The `rec:` extension (used by todo.txt-cli, Simpletask, and others) lets a task auto-generate the next occurrence when completed. Without it, users with recurring work (weekly reviews, monthly reports, daily habits) must manually re-add completed tasks. The todotxt.net C# app had a "postpone" mechanism; the Rust port has `postpone` in the CLI but nothing for automatic recurrence on completion.

## When to Surface

**Trigger:** When recurring/repeating task workflows become a product priority (v1.7 or later, or if user demand is high).

Matches when:
- Productivity/workflow features are being added
- `rec:` or repeating task support is requested
- The core library gets extension token parsing improvements

## Scope Estimate

**Medium** — Two recurrence modes exist in the todo.txt ecosystem:

| Mode | Syntax | Behavior |
|------|--------|----------|
| Strict | `rec:+Nd/Nw/Nm/Ny` | Next due = completed due_date + interval |
| Relative | `rec:Nd/Nw/Nm/Ny` (no `+`) | Next due = completion_date + interval |

Key work items:
- Parse `rec:` extension token in `todotxt-core` Task struct (currently lands in body as unknown token)
- On `toggle_done` for a task with `rec:`, generate a new incomplete task with the next due date
- Append the new task rather than replacing the completed one (completed task stays in file until archived)
- CLI: `todo do <id>` should also trigger recurrence generation
- TUI: recurrence generation happens transparently on `x` key
- Visual indicator in the TUI for tasks that will recur (`rec:` token visible or icon)

## Breadcrumbs

| File | Notes |
|------|-------|
| `crates/todotxt-core/src/task.rs` line 201 | Note that `rec:+1w` lands in body as unknown token — first thing to fix |
| `crates/todotxt-core/tests/normalize_tests.rs` line 105–109 | Existing test asserting `rec:+1w` lands in body (will need updating) |
| `crates/todotxt-tui/src/app.rs` line 2700–2712 | `toggle_done()` — recurrence generation hook goes here |
| `crates/todotxt-cli/src/commands/` | CLI `do` command would also need recurrence awareness |
| `crates/todotxt-cli/src/cli.rs` line 132–144 | `Postpone` command — related concept, provides interval arithmetic reference |

## Notes

Decide between two recurrence models upfront:
1. **Auto-create on complete** — the new task is immediately written to `todo.txt` when the old one is marked done. Simple but potentially surprising.
2. **Prompt on complete** — TUI shows "This task recurs (rec:+1w). Create next occurrence?" with a confirm/skip. More explicit, less magic.

Option 2 is safer for a v1 implementation. The C# app prompted for this.

Also note: tasks with both `rec:` and `due:` are common. The recurrence engine needs to handle the `due:` → `t:` threshold dance correctly (some users use `t:` as the show-after date alongside `due:` as the deadline).

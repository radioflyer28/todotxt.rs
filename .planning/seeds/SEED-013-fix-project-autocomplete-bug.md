---
id: SEED-013
status: dormant
planted: 2026-05-04
planted_during: v1.5 complete / preparing v1.6
trigger_when: next milestone (v1.6) — bug fix priority
scope: Small
---

# SEED-013: Fix `+` project autocomplete not showing suggestions

## Why This Matters

`@context` autocomplete works correctly — typing `@` in the task editor shows a suggestion popup. `+project` autocomplete shows no entries despite the same code path being used for both. This is a defect: users adding project tags get no completion assistance, which is especially painful for project names that are long or inconsistently remembered.

## When to Surface

**Trigger:** Next milestone (v1.6) — this is a bug fix and should be treated as higher priority than new features.

## Scope Estimate

**Small** — The `@` and `+` paths share `collect_tokens()` and `update_autocomplete()`. The bug is likely one of:

1. **Tasks in the list have no `+project` tags** — `t.projects` is empty so `collect_tokens('+')` returns nothing. If the user's `todo.txt` happens to have contexts but no projects, `@` works and `+` appears broken. (Cosmetic issue — add an "no existing projects" hint rather than silent empty popup.)

2. **`t.projects` stores tokens with the `+` prefix** — If the parser stores `"+myproject"` instead of `"myproject"`, then `filter(|t| t.starts_with(""))` works but `format!("{}{}", '+', token)` renders `"++myproject"`. Needs investigation in the task parser.

3. **`rfind(['@', '+'])` returns the wrong position** — If a task already contains `@work`, typing `+` might cause `rfind` to find the `@` instead of `+`, mis-identifying the trigger.

## Breadcrumbs

| File | Notes |
|------|-------|
| `crates/todotxt-tui/src/app.rs` line 1836–1843 | `collect_tokens()` — check what `t.projects` actually contains |
| `crates/todotxt-tui/src/app.rs` line 1875–1902 | `update_autocomplete()` token branch — `rfind(['@', '+'])` and prefix extraction |
| `crates/todotxt-core/src/task.rs` line 35 | `pub projects: Vec<String>` — verify whether values include or exclude `+` prefix |
| `crates/todotxt-core/src/` (parser) | Task parser — source of truth for how project tokens are stored |

## Notes

Reproduce: Create a `todo.txt` with at least one `+project` tag (e.g., `Fix bug +myproject`). Open the TUI, press `n` to add a task, type `+` — if the popup doesn't appear, the bug is confirmed.

Also check: does the popup appear but immediately close? Or never appears at all? That distinguishes between a data issue (empty list) and a render/trigger detection issue.

Candidate fix: if `collect_tokens('+')` returns an empty list but there IS at least one task with projects (checked independently), there is a data representation mismatch to fix in the parser.

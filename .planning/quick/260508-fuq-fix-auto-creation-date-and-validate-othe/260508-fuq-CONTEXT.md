# Quick Task 260508-fuq: fix auto_creation_date and validate other config.toml options are applied - Context

**Gathered:** 2026-05-08
**Status:** Ready for planning

<domain>
## Task Boundary

Fix `auto_creation_date = true` in config.toml having no effect on new tasks created in the TUI
(`AppMode::Adding`). Validate that `normalize_append` and `normalize_edit` are also applied
correctly. Provide tests proving the behavior.

User's config.toml (provided):
```toml
auto_creation_date = true
normalize_append = true
normalize_edit = true
```

</domain>

<decisions>
## Implementation Decisions

### Paste path (p key)
- Do NOT apply `auto_creation_date` to pasted tasks. Paste is raw content (D-12: no transformation).
  User controls dates in pasted lines.

### User-typed date
- Only inject today's date if the parsed task has `creation_date == None`.
  If the user typed a date manually (e.g. `2026-06-01 buy milk`), preserve it — do not overwrite.

### Scope
- Primary fix: `save_and_exit()` `AppMode::Adding` arm — inject `Local::now().date_naive()` when
  `self.config.auto_creation_date && task.creation_date.is_none()`.
- `normalize_append` and `normalize_edit` are already wired in `app.rs` — confirm they work;
  add tests if missing, do not refactor working code.

### Agent's Discretion
- Where to place the creation_date injection: after `Task::parse(&text)`, before `task_list.add`.
- Test strategy: unit tests in `app.rs` (existing pattern for quick tasks).

</decisions>

<specifics>
## Specific Ideas

- `chrono::Local` is already imported in `app.rs` — use `Local::now().date_naive()` for today.
- `Task::with_creation_date(Some(date))` is the correct builder (not direct struct mutation).
- Existing `normalize_append` test: see config.rs line 564 — verify it deserializes correctly.
  Behavioral tests for normalize are in `app.rs` test section.

</specifics>

<canonical_refs>
## Canonical References

- `crates/todotxt-tui/src/app.rs` — `save_and_exit()`, `paste_from_clipboard()`
- `crates/todotxt-core/src/task.rs` — `Task::parse`, `Task::with_creation_date`
- `crates/todotxt-tui/src/config.rs` — `TuiConfig::auto_creation_date`, `normalize_append`, `normalize_edit`

</canonical_refs>

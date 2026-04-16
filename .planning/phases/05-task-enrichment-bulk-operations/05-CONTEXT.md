---
phase: "05"
phase_name: "Task Enrichment + Bulk Operations"
status: "Ready for Planning"
gathered: "2026-04-16"
---

# Phase 5 Context: Task Enrichment + Bulk Operations

## Scope

Add priority manipulation (`pri`/`depri`), smart due-date management (`due`/`postpone`), and bulk archive/cleanup (`archive`, `del-done`) — completing the full CLI command surface.

**Out of scope (this phase):** Any new filtering/sorting on enriched fields, TUI/GUI integration, shell completion updates (tracked for Phase 6).

## Canonical Refs

- `.planning/ROADMAP.md` — Phase 5 deliverables, UAT criteria, and requirement IDs (ENRICH-01–04, BULK-01–02)
- `.planning/phases/04-cli-write-commands-update-archive/04-CONTEXT.md` — Established patterns for write commands (renderer, exit codes, multi-ID)
- `crates/todotxt-cli/src/output.rs` — `print_write_result` renderer (carry forward)
- `crates/todotxt-cli/src/commands/complete.rs` — Multi-ID validation pattern (validate all before mutating)
- `crates/todotxt-cli/src/commands/del.rs` — Multi-ID + fail-fast pattern
- `crates/todotxt-cli/src/config.rs` — Config struct to extend with `done_file` field

## Decisions

### D-01: Multi-ID support for all enrichment commands
**Decision:** `pri`, `depri`, `due`, and `postpone` all accept multiple IDs — same pattern as `do`/`undo`/`del` from Phase 4.
**Behavior:** Validate ALL IDs before mutating any (fail-fast). Sort descending + dedup to prevent index shift. Print `print_write_result` for each updated task.
**Rationale:** Consistent with Phase 4 pattern; bulk priority setting is a common workflow.

### D-02: done.txt path resolution
**Decision:** Configurable via `config.toml` with a `done_file` field; default when unset: sibling of `todo.txt` (same directory, filename `done.txt`).
**Implementation:** Add `pub done_file: Option<PathBuf>` to `Config` with `#[serde(default)]`. Resolve at runtime: if `Some(path)`, use it; else derive from `todo_file.parent() / "done.txt"`.
**Rationale:** Matches how todo.txt tools conventionally work; power users can override.

### D-03: Date input scope for `due` command
**Decision:** Strict ROADMAP list only — `today`, `tomorrow`, `monday`–`sunday` (next occurrence of that weekday), `YYYY-MM-DD`.
**No `+N` shorthand** — `postpone` already covers the "+N days" use case.
**Shared utility:** `parse_date_input(s: &str, today: NaiveDate) -> Result<NaiveDate>` in `crates/todotxt-cli/src/date.rs`, used by both `due` and `postpone`.
**Weekday resolution:** "next occurrence of weekday" — if today is Monday and user says `monday`, resolve to next Monday (7 days), not today.

### D-04: `archive` idempotency
**Decision:** Print `"0 tasks archived"` to stderr (info), exit 0. Running `archive` on an already-clean file is always a safe no-op.
**Atomicity:** Both `todo.txt` and `done.txt` written via temp-file + rename before either is committed (matches existing TaskList atomic write pattern). No task should ever appear in both files after a successful archive.
**done.txt creation:** Creates `done.txt` if absent (append mode; create if missing).

### D-05: `del-done` output
**Decision:** Print `"Deleted N completed tasks"` to stderr (info), exit 0. `N = 0` is a valid no-op (print `"Deleted 0 completed tasks"`).
**Consistency:** Mirrors `archive` zero-count behavior.

### D-06: Exit codes (carry forward from Phase 4)
- `0` — success (including no-op archive/del-done)
- `1` — ID not found, file error, or other runtime error
- `2` — validation error (invalid priority letter, invalid date format, no due date on postpone, empty ID list)

### D-07: JSON output (carry forward from Phase 4)
All enrichment and bulk commands must respect `--json` flag:
- Enrichment commands (`pri`/`depri`/`due`/`postpone`): return `json_success(task_dto(idx, task))` per task (same as Phase 4 write commands)
- Bulk commands (`archive`/`del-done`): return `json_success` with a count field — `{"schema_version":1,"status":"ok","count":N}` — no per-task list

### D-08: `postpone` missing due date
**Decision:** Exit code 2, error message to stderr: `"task N has no due date"`. Do NOT invent a due date — explicit error.

## Patterns to Follow

- **Multi-ID validation:** See `commands/complete.rs::validate_id` and `commands/del.rs` — validate ALL before mutating ANY
- **Atomic writes:** `TaskList::save()` already uses temp-file + rename; `archive` needs to extend this to two-file atomic write
- **Renderer:** `Renderer::print_write_result(&self, info: &str, idx: usize, task: &Task)` — same as Phase 4
- **Config:** `Config` struct in `config.rs` — add `done_file: Option<PathBuf>` with `#[serde(default)]`
- **Date dependency:** Add `chrono` to `todotxt-cli/Cargo.toml` (NaiveDate). Check if already present.

## Deferred Ideas

None captured during this discussion.

# Phase 37: Metadata Flexibility + View Continuity — Context

**Gathered:** 2026-04-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 37 delivers two things:

1. **Hierarchical tag support (META-01/02)** — Explicit parent-prefix filter matching so that `@email` matches both `@email` and `@email/waiting`. Tokens like `@email/waiting` and `+client/acme` are already accepted as ordinary tokens and parse/round-trip correctly. What is missing is intentional parent-prefix semantics in the filter engine.

2. **View continuity validation (VIEW-03)** — Regression tests proving that all v1.5 flows (add, edit, delete, toggle, bulk-append, paste, undo) leave pane filter/sort/group state intact. Also document/verify that undo restores the original raw task text (including original tag order).

In scope:
- META-01: `@context` and `+project` metadata remain plain todo.txt tokens — already true, just verify
- META-02: Hierarchical conventions like `@email/waiting` and `+client/acme` are accepted and queryable via parent-prefix matching
- VIEW-03: Existing filter/sort/group views work consistently after all v1.5 mutation flows
- One test per v1.5 flow × view state (filter_query + sort_order + grouping preserved)
- Undo raw-text restoration test (tag order restored after undo)

Out of scope:
- Autocomplete display of hierarchical tokens (noted as a potential backlog idea)
- Saved filter presets
- Any new filter UX beyond query semantics
- Fixing `rebuild_raw` tag reordering (accepted behavior — tags move to end-of-line after mutation)

</domain>

<decisions>
## Implementation Decisions

### Hierarchical Tag Semantics (META-02)

- **D-01:** Parent-prefix filter matching is implemented in **`todotxt-core` (`filter.rs`)**. CLI and TUI both benefit automatically without duplication.

- **D-02:** Two new `FilterTerm` variants:
  - `ContextPrefix(String)` — matches any task whose contexts include the exact string OR a string that starts with `{prefix}/` (slash-delimited prefix). Case-insensitive.
  - `ProjectPrefix(String)` — same semantics for project tags.

- **D-03:** `Filter::from_query` parser detects `@foo` tokens (no `/`) as `ContextPrefix("foo")` and `+foo` tokens (no `/`) as `ProjectPrefix("foo")`. Tokens with a slash (e.g., `@email/waiting`) remain `Include` terms matching on `to_raw()` for exact hierarchical match.

  Parsing rule:
  - `@email` → `ContextPrefix("email")` (matches `@email` and `@email/*`)
  - `@email/waiting` → `Include("@email/waiting")` (exact match via substring on raw)
  - `-@email` → negated equivalent (decide in planning: `NegContextPrefix` or leave as `Exclude`)
  - `+client` → `ProjectPrefix("client")` (matches `+client` and `+client/*`)

- **D-04:** `ContextPrefix` matching logic: `task.contexts.contains(prefix) || task.contexts.iter().any(|c| c.starts_with(&format!("{prefix}/")))`. Case-insensitive comparison via `.to_ascii_lowercase()`.

- **D-05:** `ProjectPrefix` matching logic: same pattern but over `task.projects`.

- **D-06:** Negated prefix forms: decide during planning. Simple approach is `NegContextPrefix(String)` / `NegProjectPrefix(String)` variants — planner's discretion if these are needed for Phase 37 scope or can be left for a later phase.

### Tag Ordering After Mutation (META-01 / known behavior)

- **D-07:** `rebuild_raw` moves `@ctx`/`+proj`/`due:`/`t:` tags to end-of-line after any `with_*` mutation. This is **accepted behavior** — all v1.5 phases use it, sort uses parsed fields not raw position, and no user-visible harm has been identified.

- **D-08:** Undo **restores the original raw text** because `UndoEntry.tasks` captures the full `Task` slice (including the verbatim `raw` field) before any mutation. After `apply_undo()` + `replace_all()`, the file is written from the original raws — tag order is fully restored. Add a regression test proving this.

### VIEW-03 Test Strategy

- **D-09:** One regression test per v1.5 mutation flow. Each test:
  1. Sets up a pane with a filter query, specific sort order, and grouping enabled
  2. Performs the mutation
  3. Asserts that `filter_query`, `sort_order`, and `grouping` are unchanged on the active pane

- **D-10:** V1.5 flows to cover (7 tests):
  1. Add task (save_and_exit from Adding mode)
  2. Edit task (save_and_exit from Editing mode)
  3. Delete single task
  4. Toggle done
  5. Bulk append (`T`)
  6. Paste from clipboard (`p`)
  7. Ctrl+Z undo (apply_undo)

- **D-11:** The tests can reuse existing `App::new_test()` + `TaskList` test harness patterns from Phases 34-36. No new test infrastructure needed.

### Plan Structure

- **Plan 37-01 (TDD):** Add `ContextPrefix`/`ProjectPrefix` variants to `FilterTerm` + update `Filter::from_query` + `matches_with_date` in `todotxt-core`. TDD: tests RED then GREEN. No TUI changes.
- **Plan 37-02 (execute):** VIEW-03 regression tests for all 7 v1.5 flows + undo raw-text restoration test. Tests verify filter/sort/group state preserved and undo restores original raw. `cargo test` passes.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase and Scope Authority
- `.planning/ROADMAP.md` — Phase 37 entry (lines 71-75): requirements META-01, META-02, VIEW-03
- `.planning/REQUIREMENTS.md` — META-01, META-02, VIEW-03 definitions
- `.planning/phases/34-bulk-action-safety-metadata-preservation/34-CONTEXT.md` — D-13/D-14: structured Task mutation via with_* builders (used by all property setters)

### Core Library Files
- `crates/todotxt-core/src/filter.rs` — Current `FilterTerm` enum + `Filter::from_query` + `matches_with_date` — **the file to modify for D-01/D-03/D-04**
- `crates/todotxt-core/src/task.rs` — `extract_tags`, `rebuild_raw`, `Task` struct — understand how contexts/projects are parsed and stored
- `crates/todotxt-tui/src/app.rs` — `apply_undo` (line ~280) + `UndoEntry` — context for D-08 undo raw restoration test
- `crates/todotxt-tui/src/state.rs` — `UndoEntry` struct — contains `tasks: Vec<Task>` including raw fields
- `crates/todotxt-core/src/task_list.rs` — `replace_all` — the restore path that writes original raws back to disk

### Test Patterns
- `crates/todotxt-core/src/filter.rs` (existing tests) — current filter test patterns to extend
- `crates/todotxt-tui/tests/single_pane_test.rs` — pane filter/sort/group state test patterns (reuse)
- `crates/todotxt-tui/src/app.rs` (mod tests) — Phase 36 undo tests for App::new_test() pattern

</canonical_refs>

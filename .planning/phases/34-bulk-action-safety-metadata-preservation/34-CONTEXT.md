# Phase 34: Bulk Action Safety + Metadata Preservation — Context

**Gathered:** 2026-04-30
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 34 delivers three things:

1. **Unified affected-count preview** before all high-impact bulk actions — `s` (due-date overwrite), `i` (priority overwrite), `T` (bulk append), `D` (bulk delete) — when N > 1 tasks are targeted.
2. **`i` priority picker** (CAP-04 gap from Phase 33) — scrollable overlay, same UX family as `s`, with letter-type-to-jump and Enter to confirm.
3. **Metadata preservation audit** — refactor property setters to use structured Task mutation rather than raw token surgery, guaranteeing all non-target fields survive.

In scope:
- Count preview for `s`, `i`, `T`, `D` when >1 task affected
- Cancel path that preserves current selection
- `i` priority picker with scrollable overlay and type-to-filter interaction
- Structured Task-model mutation for `s` and `i` setters
- Full metadata preservation: completion prefix, completion date, creation date, `t:` threshold, priority, `due:`, all `@`/`+` tokens

Out of scope for this phase:
- Clipboard (cut/copy/paste) — Phase 35
- Undo stack — Phase 36
- Completion date picker — noted as a future backlog idea

</domain>

<decisions>
## Implementation Decisions

### Count Preview — Trigger and Scope

- **D-01:** The unified count preview applies to all high-impact bulk actions: `s` (due-date overwrite), `i` (priority overwrite), `T` (bulk append), `D` (bulk delete).
- **D-02:** Preview activates only when N > 1 tasks are targeted. Single-task operations apply directly without a preview step (no extra keypress for the common case).
- **D-03:** On cancel from any count preview step, the current selection is **preserved** — the user can re-trigger a different action without re-selecting.
- **D-04:** `@`/`+` quick setters do NOT get a count preview — they are add-only/idempotent and considered low-risk.

### Count Preview — UX by Action

- **D-05:** For `s` (due-date picker) and `i` (priority picker): the count appears **inline in the picker overlay header** — e.g., `"Setting due date — 5 tasks"` or `"Setting priority — 5 tasks"`. No separate confirmation step after selection; Enter applies directly.
- **D-06:** For `T` (bulk append): a brief count banner/header appears before the text entry bar opens — e.g., `"Appending to 5 tasks — Enter to confirm, Esc to cancel"`. User then types text and presses Enter as today.
- **D-07:** For `D` (bulk delete): the existing `DeleteConfirm` panel wording is updated to align with the unified model (ensure count is clearly shown). The pattern stays as a separate confirmation panel (not changed to inline).

### `i` Priority Picker (CAP-04)

- **D-08:** `i` opens a scrollable picker overlay — same UX family as the `s` date picker (overlay, Up/Down navigation, Enter confirms).
- **D-09:** The priority picker additionally supports **typing a letter** (A–Z, case-insensitive) to filter/jump within the list; pressing Enter after typing a letter confirms that priority. This is a superset of the `s` picker behavior.
- **D-10:** The `i` picker targets selected tasks when `selected_tasks` is non-empty; otherwise targets the active cursor task — same semantics as `s`.
- **D-11:** Count preview (D-05) applies to `i` when N > 1 tasks selected.
- **D-12:** A "clear priority" option should be included in the picker (e.g., `"— (no priority)"`) so users can remove priority without editing the task manually.

### Metadata Preservation

- **D-13:** All property setter mutations (`s`, `i`) must use **structured Task-model mutation** — parse the task to the `Task` struct, mutate only the target field (e.g., `task.due_date = Some(new_date)`, `task.priority = Some('A')`), then serialize back. No raw whitespace-split token surgery.
- **D-14:** Full metadata preservation is required: `x` completion prefix, completion date, creation date, `t:` threshold, `due:`, priority `(A)`, and all `@`/`+` tokens must survive any property setter operation unchanged.
- **D-15:** Setters work on completed tasks — the `x` prefix and completion date are preserved; only the target field is mutated.
- **D-16:** `normalize_line()` behavior is trusted as-is — no special bypass needed for setter mutations.

### Bulk Action Targeting (BULK-03)

- **D-17:** Existing descending-index ordering for bulk mutations is preserved and applies to `s` and `i` setters (already done in Phase 33 for `s`; apply same to `i`).
- **D-18:** Multi-select and grouped/pane view selection targeting must remain stable — no regressions from Phase 19/20 invariants.

### Agent's Discretion

- Exact overlay widget reuse strategy between `s` picker and `i` picker (shared component vs. copy with adaptation)
- Exact wording of count labels in the overlay header
- Whether the `i` picker list is pre-filtered to A–Z only or includes the "no priority" item at the top/bottom of the list
- Visual styling of the count label (subtitle line vs. title suffix)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase and Scope Authority

- `.planning/ROADMAP.md` — Phase 34 scope, requirement mapping (CAP-05, TAG-03, BULK-01, BULK-02, BULK-03)
- `.planning/REQUIREMENTS.md` — CAP-04, CAP-05, TAG-03, BULK-01, BULK-02, BULK-03 full text
- `.planning/phases/34-bulk-action-safety-metadata-preservation/34-CONTEXT.md` — this file

### Existing Bulk Action Implementation

- `crates/todotxt-tui/src/app.rs` — `handle_date_picker_key()` (current `s` implementation, lines ~1882–1960), `handle_delete_confirm_key()` (D pattern), `handle_append_text_key()`, `handle_quick_setter_key()`, `apply_token_to_tasks()`
- `crates/todotxt-tui/src/state.rs` — `DatePickerState`, `AutocompleteState`, `AutocompleteMode`

### Task Model (for structured mutation, D-13)

- `crates/todotxt-core/src/task.rs` — `Task` struct fields and serialization
- Look for: `Task::due_date`, `Task::priority`, `Task::completion_date`, `Task::creation_date`, `Task::contexts()`, `Task::projects()`, serialization/to_raw methods

### Prior Phase Contracts (must not regress)

- `.planning/phases/19-selection-model-multi-select-foundation/19-CONTEXT.md` — selection state invariants
- `.planning/phases/20-bulk-actions-selection-ux/20-CONTEXT.md` — D-01 through D-14 (bulk delete/append patterns, descending-index rule)
- `.planning/phases/33-fast-capture-property-pickers/33-CONTEXT.md` — D-07 through D-09 (token application rules, idempotency, targeting)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `DatePickerState` + `render_date_picker_overlay()` — directly reusable for `i` picker with a different data source (A–Z priority list instead of date suggestions)
- `AppMode::DeleteConfirm` + `render_delete_confirm()` — existing count-confirmation pattern; wording update only for D-07
- `handle_date_picker_key()` — Tab/Enter acceptance flow is the template for `i` picker acceptance
- `apply_token_to_tasks()` — idempotent token application already implements descending-index targeting

### Established Patterns

- Picker overlays follow: enter mode → render overlay → handle nav keys (Up/Down) → Tab/Enter accepts → Esc cancels, restores to Normal
- All bulk mutations use `self.task_list.batch_update(replacements)` followed by `rebuild_and_reanchor()`
- Count-preview wording: "Delete N tasks? (y/n)" — Phase 34 unifies all actions to show count before committing

### Integration Points

- `handle_normal_key()` — entry point for `s`, `i`, `D`, `T` key handlers; count preview logic gates here
- `AppMode` enum — new variant needed for `i` picker (e.g., `PriorityPicker`); possibly a count-preview variant for `T`
- `Task` struct in `todotxt-core` — D-13 requires understanding the struct fields before refactoring `s` setter

</code_context>

<specifics>
## Specific Ideas

- User explicitly requested: `i` picker should allow typing a priority letter to jump + Enter to confirm (type-to-filter on top of scrollable list)
- Completion date picker noted as a future idea (not in scope for this phase)
- Completed tasks should be mutable via property setters — `x` prefix and completion date are preserved, only the target field changes

</specifics>

<deferred>
## Deferred Ideas

- **Completion date picker** — user noted it would be useful to add a date picker for marking tasks complete on dates other than today. Belongs in a future phase or backlog.

</deferred>

---

*Phase: 34-bulk-action-safety-metadata-preservation*
*Context gathered: 2026-04-30*

# Phase 20: Bulk Actions + Selection UX — Context

**Gathered:** 2026-04-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire the Phase 19 selection foundation to actual bulk operations and make selection state visible to the user.

Phase 20 delivers:
1. **Bulk delete** — `D` on a non-empty selection triggers a count-confirmation prompt, then deletes all selected tasks atomically with descending-index safety
2. **Bulk append** — `T` on a non-empty selection opens the inline editor bar; text entered is appended to every selected task
3. **Selection visibility** — status bar shows `| N selected` when any tasks are selected (BULK-03)

Phase 21 (Smart Text Normalization) owns structured token handling. Phase 22 (Keymap Parity) owns full help-screen updates. This phase delivers the behavior; cosmetic parity of the help hints is Phase 22.

</domain>

<decisions>
## Implementation Decisions

### Bulk Delete (BULK-01)

- **D-01:** Hotkey `D` triggers bulk delete when `!selected_tasks.is_empty()` (PAR-01 parity with todotxt.net). When selection is empty, `D` retains existing single-delete behavior (no regression).
- **D-02:** Confirmation uses the existing `AppMode::DeleteConfirm` mode. When bulk (>1 task), the confirm panel message changes to `"Delete N tasks? (y/n)"` instead of showing a task preview. When single-task-via-selection (1 task in set), show the existing preview panel behavior.
- **D-03:** Deletion is performed in **descending canonical index order** — highest index first — so each deletion does not invalidate the indices of remaining tasks. Required for correctness when deleting multiple non-contiguous tasks.
- **D-04:** After bulk delete completes, `selected_tasks` is cleared and `disjoint_select` is reset to `false`. Mode returns to `Normal`.
- **D-05:** `Group header rows` are never in `selected_tasks` (enforced by Phase 19) — no defensive checks needed here.

### Bulk Append (BULK-02)

- **D-06:** Hotkey `T` triggers bulk append when `!selected_tasks.is_empty()` (PAR-01 parity — todotxt.net binds `T` to AppendText). `T` is currently unbound in the TUI, no regression risk.
- **D-07:** Append text is entered via the **existing inline editor bar** at the bottom of the terminal. Implement as a new `AppMode::AppendText` variant (alongside existing `Adding`, `Editing`) so the existing `handle_adding_key` / commit-edit path can be reused with minimal duplication.
- **D-08:** The user types arbitrary freeform text. What gets appended is that literal text — no structured token parsing (Phase 21 owns normalization). Empty input (Enter with no text) cancels without mutating any tasks.
- **D-09:** Append is applied in **descending canonical index order** for consistency with delete, even though append does not shift indices. This keeps the two bulk paths symmetric.
- **D-10:** After bulk append completes, `selected_tasks` is cleared and mode returns to `Normal`.
- **D-11:** The inline editor bar prompt label changes to `"Append: "` when in `AppendText` mode (vs `"Add: "` / `"Edit: "`).

### Selection Visibility / Status Bar (BULK-03)

- **D-12:** When `!selected_tasks.is_empty()`, the status bar left segment appends `| N selected` after the task count. Example: `"todo.txt | 12/47 tasks | 2 selected"`. Uses the existing left-segment string-building pattern in `render_status_bar`.
- **D-13:** No separate mode indicator string (e.g., `[v-mode]`) in the status bar for now. The `| N selected` count is sufficient discoverability for v1.3. Mode labeling is Phase 22 / help-parity scope.
- **D-14:** When `disjoint_select` is true AND tasks are selected, the indicator is the same `| N selected` — no separate `[v]` prefix. Keeps the status bar uncluttered.

### Agent's Discretion

- Whether `AppMode::AppendText` shares the text-box input widget with `AppMode::Adding` via a shared helper, or duplicates the ~10 lines of editor rendering — planner decides the cleanest approach given app.rs size.
- Whether the bulk-delete confirmation panel is a refactor of `render_delete_confirm` or a new function — planner decides.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary integration file
- `crates/todotxt-tui/src/app.rs` — `App` struct, `selected_tasks: HashSet<usize>`, `disjoint_select: bool`, `AppMode` enum, `handle_normal_key()`, `handle_delete_confirm_key()`, `render_status_bar()`, `render_delete_confirm()`, `commit_edit()`

### Styling
- `crates/todotxt-tui/src/theme.rs` — `StyleSheet`, `Theme` — follow existing `Modifier` patterns

### Reference implementation
- `Client/Controls/MainWindow.xaml` — `SelectionMode="Extended"`, `D`→DeleteTask, `T`→AppendText, `WhenTasksSelectedCanExecute` gate confirms that both operate on the full selection
- `Client/MainWindowViewModel.cs` — `GetSelectedTasks()` / `SetSelectedTasks()` pattern for preserving selection across list refreshes

### Phase 19 contracts (must not be broken)
- `.planning/phases/19-selection-model-multi-select-foundation/19-CONTEXT.md` — D-01 through D-20 (selection state model, rendering rules, persistence invariants)

### Requirements
- `.planning/REQUIREMENTS.md` — BULK-01 (bulk delete), BULK-02 (bulk append), BULK-03 (selection count visible), PAR-01 (hotkey parity)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AppMode::DeleteConfirm` + `render_delete_confirm()` + `handle_delete_confirm_key()` — complete single-delete flow; bulk delete reuses this with a modified confirmation message
- Inline editor bar (`AppMode::Adding`, `AppMode::Editing`) — keyboard input, rendering, commit path; bulk append adds a third mode variant with the same shape
- `render_status_bar()` left-segment string building — straightforward to append `| N selected` to the existing interpolation
- Navigation header skip logic — established pattern for iterating only `DisplayRow::Task` rows

### Established Patterns
- Key dispatch in `handle_normal_key` dispatches on `(key, mode, condition)` — bulk hotkeys follow the same pattern with `!selected_tasks.is_empty()` guard
- All multi-step operations (delete, edit) use `AppMode` as a state machine gate — bulk operations fit naturally as new arms or guards on existing arms
- Descending-index deletion: standard approach for safe multi-index removal in Rust `Vec` without cloning

### Integration Points
- `handle_normal_key`: add `D` guard and `T` handler
- `App::new`: no changes (fields initialized in Phase 19)
- `render_status_bar`: one insertion point in the left-segment build
- New `handle_append_text_key` function (analogous to `handle_delete_confirm_key`)

</code_context>

<discussion_log>
## Discussion Log

**2026-04-24** — User delegated all Phase 20 decisions to agent judgment. Agent applied:
- todotxt.net hotkey parity (D=delete, T=append) from Client/Controls/MainWindow.xaml KeyBinding inspection
- Existing AppMode state machine pattern for confirmation and input flows
- Minimal status bar insertion following existing render_status_bar string-building pattern
- Descending-index safety for both bulk operations (symmetry)
</discussion_log>

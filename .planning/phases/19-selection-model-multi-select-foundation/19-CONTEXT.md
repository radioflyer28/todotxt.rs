# Phase 19: Selection Model + Multi-Select Foundation - Context

**Gathered:** 2026-04-24
**Status:** Ready for planning

<domain>
## Phase Boundary

Add canonical multi-task selection state to the TUI. Phase 19 delivers:
1. **Selection data model** — `HashSet<usize>` of canonical indices on `App`; anchor tracking for shift-range
2. **Contiguous range selection** — Shift+j/k and Shift+arrow extend selection from anchor; Shift+Ctrl+D/U half-page range
3. **Disjoint selection mode** — `v` toggles a flag on `App`; `Space` marks/unmarks the cursor task; `Esc` clears selection and exits mode
4. **Selection persistence** — canonical indices survive regroup, resort, refilter, and reload (valid indices kept, gone indices dropped)
5. **Rendering** — cursor uses existing `REVERSED`; selected non-cursor rows use `Bold` + `>` prefix; cursor-on-selected row uses `REVERSED` + `Bold`

Bulk actions (delete, append) are Phase 20. Keymap config is Phase 22.

</domain>

<decisions>
## Implementation Decisions

### Selection State

- **D-01:** Selection state is a `HashSet<usize>` of **canonical file indices** — consistent with the existing `display_indices: Vec<usize>` pattern; fast membership testing
- **D-02:** An **anchor index** (`Option<usize>`) tracks the start of the most recent shift-range operation; stored on `App`
- **D-03:** Selection is identified by canonical index (not raw text); on reload, indices that no longer exist in the task list are silently dropped from the set

### Disjoint Selection Mode

- **D-04:** Implemented as a **boolean flag** `disjoint_select: bool` on `App`, NOT a new `AppMode` variant — normal navigation keys continue to work while disjoint mode is active
- **D-05:** `v` key toggles disjoint selection mode on/off (vi visual-line selection mnemonic; PAR-01 parity)
- **D-06:** `Space` marks or unmarks the cursor task in the selection set while `disjoint_select` is true
- **D-07:** `Esc` while `disjoint_select` is true: **clears the entire selection** and exits disjoint mode (does not keep the selection)
- **D-08:** Non-task rows (group headers) are never added to the selection set — `Space` on a group header row is a no-op

### Shift-Range Selection

- **D-09:** **Both** `Shift+j`/`Shift+k` and `Shift+Down`/`Shift+Up` extend the contiguous range from the anchor
- **D-10:** `Shift+Ctrl+D` / `Shift+Ctrl+U` extend the range by half-page (consistent with plain `Ctrl+D`/`Ctrl+U`)
- **D-11:** First shift-nav with no prior anchor: set anchor to current cursor position, then extend
- **D-12:** Non-shift navigation clears the anchor (but does NOT clear the selected set — that requires Esc)

### Rendering

- **D-13:** **Cursor row** (focused navigation highlight): `Modifier::REVERSED` — unchanged from current behavior
- **D-14:** **Selected non-cursor row**: `Modifier::BOLD` + `>` prefix glyph (e.g., `> Buy groceries`)
- **D-15:** **Cursor row that is also selected**: `Modifier::REVERSED | Modifier::BOLD` (both modifiers combined)
- **D-16:** Normal (unselected, non-cursor) rows: unchanged styling
- **D-17:** Group header rows: unchanged styling regardless of selection state

### Selection Persistence (SEL-03)

- **D-18:** After any `rebuild_display_indices()` call (triggered by regroup, resort, refilter), the selected set is unchanged — canonical indices remain valid
- **D-19:** After a `FileChanged` reload: retain indices `< task_list.len()` that still exist; drop any that fell out of range
- **D-20:** After a filter change that hides selected tasks: selected indices are retained in the set even when not visible — they re-appear as selected if the filter is cleared

### Agent's Discretion

- Exact field names for the new `App` struct fields (`selected_tasks`, `anchor`, `disjoint_select`) — planner decides naming
- Whether `Space` also moves the cursor to the next task after marking (like vi `V` line-mark mode) — planner decides
- Status bar indicator for selection count and disjoint mode (e.g., `[2 selected]`, `[v-mode]`) — Phase 20 owns the status bar polish, but planner may add a minimal count indicator here if it aids testing

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### TUI app state and event loop
- `crates/todotxt-tui/src/app.rs` — `App` struct, `display_rows: Vec<DisplayRow>`, `display_indices: Vec<usize>`, `rebuild_display_indices()`, `rebuild_and_reanchor()`, `canonical_selected()`, `handle_normal_key()` — all primary integration points for this phase

### Display model
- `crates/todotxt-tui/src/app.rs` (lines ~80–90) — `DisplayRow` enum: `Task(usize)` and `GroupHeader(String)` — selection must only ever apply to `Task(idx)` variants

### Theme and styling
- `crates/todotxt-tui/src/theme.rs` — `StyleSheet`, `Theme` — `Modifier::REVERSED` (cursor), `Modifier::DIM` (completed/deferred); new selection style (`BOLD`) follows same pattern

### Prior phase context
- `.planning/phases/17-tui-grouping-sorting-alignment-status-polish/17-CONTEXT.md` — D-03/D-04: group header rows are decorative and non-selectable; nav already skips them
- `.planning/phases/12-filter-sort/12-CONTEXT.md` — D-10/D-11: view model design, sort-is-view-only, canonical index pattern

### Requirements
- `.planning/REQUIREMENTS.md` — SEL-01 through SEL-04 (this phase), BULK-01 through BULK-03 (Phase 20 — planner should not implement bulk actions here, only the selection foundation they depend on)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `rebuild_and_reanchor()` — already preserves cursor via canonical index after any rebuild; selection persistence (D-18) follows the same pattern and can reuse this function
- `canonical_selected() -> Option<usize>` — returns the canonical index for the cursor row; selection set operations use the same canonical index type
- `Modifier::REVERSED` — already applied per-row in `render_task_list()`; adding `BOLD` and glyph prefix follows the same render path
- Navigation skip logic for `GroupHeader` rows — already implemented in `j`/`k` handlers; `Space` mark key needs the same guard (D-08)

### Established Patterns
- `AppMode` enum gates modal behavior; this phase deliberately does NOT add a new `AppMode` (D-04) — disjoint mode is a flag, keeping Normal mode key dispatch intact
- Shift key detection: `key.modifiers.contains(KeyModifiers::SHIFT)` in `handle_normal_key()` — same pattern as existing `KeyModifiers::CONTROL` checks
- `display_indices: Vec<usize>` is the canonical list of visible task indices in display order — the selection set uses the same index space

### Integration Points
- `handle_normal_key()` — add `Shift+j/k`, `Shift+Down/Up`, `Shift+Ctrl+D/U`, `v`, `Space` cases
- `render_task_list()` — per-row styling check: if index is in `selected_tasks`, apply D-14/D-15 styling
- `rebuild_and_reanchor()` — after reload, prune `selected_tasks` of indices ≥ `task_list.len()` (D-19)
- `App::new()` — initialize `selected_tasks: HashSet::new()`, `anchor: None`, `disjoint_select: false`

</code_context>

<specifics>
## Specific Ideas

- `v` key chosen for disjoint selection (vi visual-line mnemonic; PAR-01 parity alignment)
- `Space` marks/unmarks the cursor task (familiar from file manager and email client selection UX)
- `Esc` in disjoint mode clears selection entirely — intentionally destructive so users can start fresh
- `>` glyph prefix for selected rows (vi-style visual selection indicator)
- Bold + `>` combo works in NO_COLOR/monochrome terminals without requiring theme color additions
- Shift+Ctrl+D/U half-page range is included in Phase 19 scope (consistent with plain half-page nav)
- Selected tasks stay in the set even when hidden by filter — they reappear as selected when filter clears

</specifics>

<deferred>
## Deferred Ideas

- Status bar selection count/mode indicator — planner may add a minimal count here, but full polish (BULK-03) belongs to Phase 20
- Shift+G / Shift+gg for select-to-end / select-to-start range — not discussed; planner may include if trivial
- Clipboard copy of selected task text — v2 backlog (BULK-06)

</deferred>

---

*Phase: 19-selection-model-multi-select-foundation*
*Context gathered: 2026-04-24*

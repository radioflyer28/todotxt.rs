# Phase 26: Pane Management + Quick Hide/Show - Context

**Gathered:** 2026-04-28
**Status:** Ready for planning

<domain>
## Phase Boundary

Add hotkeys for pane creation and deletion (lifecycle management), and a single-key toggle to hide all panes / restore them to the prior visible state. Covers PANE-05 and VIEW-02 requirements, plus help overlay updates for discoverability.

</domain>

<decisions>
## Implementation Decisions

### Pane Count Guardrails
- **D-01:** Minimum pane count is **0** — 0 panes is a valid state (e.g., no panes defined in config.toml on startup)
- **D-02:** Maximum pane count is **10** (indices 0–9) — arbitrary upper bound; user manages their own layout density by deleting panes
- **D-03:** Attempting to create a pane when 10 already exist is silently blocked (no error dialog, just a no-op)
- **D-04:** Attempting to delete a pane when 0 panes exist is a no-op

### Pane Creation Behavior
- **D-05:** New panes are auto-labeled (e.g., "Pane 1", "Pane 2") — no prompt for name on create
- **D-06:** New pane is appended to the right (end of the `panes` vec) with default state: empty filter, `SortOrder::FileOrder`, grouping off
- **D-07:** Focus shifts to the newly created pane after creation

### Pane Deletion Behavior
- **D-08:** The **active pane** is deleted — not always the last/rightmost
- **D-09:** After deletion, focus shifts to the adjacent pane (prefer left/prev; if none, right/next; if none, no pane)
- **D-10:** No confirmation dialog — delete is immediate
- **D-11:** Pane IDs/indices are re-normalized after deletion (no gap-based numbering)

### Hide/Show Toggle Semantics
- **D-12:** One hotkey toggles between hidden and visible states (same key hides and shows)
- **D-13:** "Hidden" renders as single-pane view (VIEW-01 behavior) — no pane borders or labels, standard task list
- **D-14:** Pane structure, count, and per-pane filter/sort/group state are **fully preserved** while hidden — no data loss
- **D-15:** On restore (toggle from hidden → visible), all panes return exactly as they were (same count, same state per pane)
- **D-16:** The hidden state is session-only — no persistence across restarts

### Hotkey Assignments
- **D-17:** `pane_add` → `Ctrl+N`
- **D-18:** `pane_delete` → `Ctrl+W`
- **D-19:** `pane_hide_toggle` → `Ctrl+P`
- **D-20:** All three actions registered in `default_keymap()` in `config.rs` — user-configurable via `config.toml` (consistent with Phase 22 keymap pattern)
- **D-21:** Hotkeys verified safe from tmux (Ctrl+B prefix) and screen (Ctrl+A prefix) interception

### Help Overlay
- **D-22:** Pane controls appear in the existing help overlay (`?`), in a new "Panes" section alongside existing hotkey groups
- **D-23:** No dedicated pane help page — single overlay is sufficient
- **D-24:** No pane count or hidden state indicator in the status bar — visual pane state is self-evident from the rendered layout

### the agent's Discretion
- Exact label numbering strategy for auto-labels (counter vs slot-based)
- Internal field name for tracking hidden state (`panes_hidden: bool` or similar on `App`)
- How `reconcile_active_pane` handles the 0-pane case (must not panic)
- Whether pane re-normalization after delete preserves `Pane.id` field or reassigns it

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Foundation
- `.planning/phases/25-per-pane-query-behavior/25-CONTEXT.md` — Per-pane query state decisions; D-13 through D-18 establish the pane routing patterns Phase 26 builds on
- `.planning/REQUIREMENTS.md` §PANE-05, §VIEW-02 — Lifecycle and visibility toggle requirements

### Existing Code
- `crates/todotxt-tui/src/app.rs` — `App.panes: Vec<Pane>`, `App.active_pane: usize`, `reconcile_active_pane()`, `active_pane_mut()`, `focus_next_pane()`, `focus_prev_pane()`
- `crates/todotxt-tui/src/state.rs` — `Pane::new(id, label)` constructor and `Pane` struct fields
- `crates/todotxt-tui/src/config.rs` — `default_keymap()` (where new actions must be registered), `resolve_keymap()` conflict detection pattern

### Phase 22 Keymap Pattern
- `.planning/phases/22-keymap-help-parity/22-CONTEXT.md` (if exists) — configurable keymap pattern all hotkeys must follow

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`Pane::new(id, label)`** — constructor ready; Phase 26 calls this for new panes
- **`reconcile_active_pane()`** — clamps `active_pane` to valid range; must be extended to handle `panes.len() == 0` without panicking
- **`focus_next_pane()` / `focus_prev_pane()`** — pane navigation; deletion focus-shift logic can reuse these
- **`default_keymap()`** — registration point for all 3 new hotkey actions

### Established Patterns
- **Configurable keymap (Phase 22):** Every user-facing action gets a string key in `default_keymap()`, conflict detection in `resolve_keymap()`, and appears in the help overlay
- **Active-pane routing (Phase 24/25):** `app.active_pane` index gates all hotkey dispatch; new pane ops follow the same gate
- **No-confirmation deletes:** Consistent with existing `delete` task action (no dialog)

### Integration Points
- **`handle_input` in `app.rs`:** Must dispatch `Ctrl+N` → `pane_add`, `Ctrl+W` → `pane_delete`, `Ctrl+P` → `pane_hide_toggle`
- **Render loop:** Must check hidden state flag before rendering multi-pane layout; fall back to single-pane render when hidden
- **Help overlay (`components/`):** Add a "Panes" section with the three new hotkeys

</code_context>

<specifics>
## Specific Ideas

- 0 panes is a first-class state — the app must not crash or show broken UI when `panes.is_empty()`
- "Pane N" auto-label should use a monotonically increasing counter (not slot index), so deleting "Pane 2" and adding a new one yields "Pane 3", not "Pane 2" again (avoids confusion)

</specifics>

<deferred>
## Deferred Ideas

- Pane rename / relabeling after creation — belongs in a future phase or Phase 27 if it serves config-defined panes
- Reorder panes interactively — v2 requirement (PANE-06)
- Persist hidden state across restarts — out of scope for this phase

</deferred>

---

*Phase: 26-pane-management-quick-hide-show*
*Context gathered: 2026-04-28*

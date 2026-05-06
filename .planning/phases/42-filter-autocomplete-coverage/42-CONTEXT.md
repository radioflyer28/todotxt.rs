# Phase 42: Filter Autocomplete Coverage — Context

**Gathered:** 2026-05-06
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 42 delivers token autocomplete (`@` / `+` triggers) in the filter input — matching
the behavior that already exists in the task editor. Three requirements in scope:

- **AC-02:** Typing `@` or `+` in the filter input shows a suggestion popup with known
  contexts or projects from the task list.
- **AC-03:** Selecting a suggestion from the popup while in filter input mode inserts it
  into the filter field (not the task editor).
- **AC-04:** Each character typed after the trigger re-filters the candidate list,
  narrowing suggestions incrementally.

Out of scope: changes to task editor autocomplete, filter history ring, preset system,
view persistence, or any other filter-input behavior added in Phase 41.

</domain>

<decisions>
## Implementation Decisions

### FilterHistory vs TokenAutocomplete Priority (AC-02)

- **D-01:** When the user types `@` or `+` in the filter input, **TokenAutocomplete
  replaces the FilterHistory popup**. The single `self.autocomplete` slot switches from
  `AutocompleteMode::FilterHistory` to `AutocompleteMode::TokenAutocomplete(char)` —
  whichever is set last wins. No coexistence or merging of modes.

  *Rationale:* The existing `AutocompleteState` slot is single-valued. Letting the token
  trigger overwrite is the cleanest implementation: one code path, no ambiguity about
  which popup is rendered. The user typed `@` — they want tag suggestions, not history.

### Post-Acceptance Behavior (AC-03)

- **D-02:** Accepting a suggestion from the popup (Tab or Enter) **inserts the token
  into the filter input and keeps the filter panel open** (stays in `AppMode::Filtering`).
  The autocomplete popup closes; the user can continue typing to build a compound
  expression (e.g., `@work +project`). Filter applies on Enter as usual.

  *Rationale:* Consistent with AC-03 ("inserts it into the filter field") without
  side effects. Enables compound filter building. Matches the mental model that the
  filter panel is a text editor, not a single-selection UI.

### Multi-Token Mid-Expression Triggering (AC-04 / AC-02)

- **D-03:** Autocomplete is **cursor-aware** — it triggers whenever the word being typed
  at the cursor position starts with `@` or `+`, regardless of what precedes it in the
  filter expression. Completing a suggestion inserts at the cursor position, replacing
  only the typed prefix after the trigger character.

  *Example:* `done:false @w` → popup shows contexts matching "w" → selecting `@work`
  yields `done:false @work` with the rest of the expression intact.

  *Rationale:* Makes autocomplete genuinely useful for compound filter queries, which is
  the primary use case in a multi-pane TUI where filters like `@work due:today` are
  common.

### Incremental Narrowing Implementation (AC-04)

- **D-04:** Each keypress after the trigger character updates `self.autocomplete` with a
  fresh `AutocompleteState::new(trigger, updated_prefix, candidates)`. The prefix is
  re-extracted from the filter input on every character event in `handle_filtering_key`.
  The existing `AutocompleteState` filtering logic (prefix-first matching via `rank_matches`)
  handles the narrowing — no new narrowing mechanism needed.

  *This is the agent's working model; the planner verifies exact re-construction vs.
  in-place mutation of `items`.*

### Keyboard Behavior (inherited from Phase 33)

- **D-05 (inherited):** Up/Down navigates popup, Tab/Enter accepts selected suggestion,
  Esc dismisses popup. These bindings apply unchanged in the filter input context.
- **D-06 (inherited):** Case-insensitive prefix-first matching, near-matches ranked after.
  Candidate source: `get_existing_contexts()` for `@`, `get_existing_projects()` for `+`.

### Agent's Discretion

- Whether to extract the "current word under cursor" via `TextArea::cursor()` column
  position or by scanning the last whitespace-delimited token in the filter text —
  planner picks the simplest approach given `tui_textarea::TextArea` API.
- Whether the popup dismisses automatically when the cursor moves past the trigger word
  boundary (e.g., user types a space after the token) — follow existing task editor
  behavior for consistency.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Autocomplete machinery (existing — reuse directly)
- `crates/todotxt-tui/src/state.rs` — `AutocompleteState`, `AutocompleteMode`
  (`TokenAutocomplete`, `FilterHistory` variants), `get_existing_contexts`,
  `get_existing_projects`, `rank_matches` — all reused for filter token autocomplete
- `crates/todotxt-tui/src/app.rs` — `self.autocomplete: Option<AutocompleteState>` field
  (line ~76) — the single autocomplete slot that Phase 42 writes into from `Filtering` mode

### Filter input key handler (primary change site)
- `crates/todotxt-tui/src/app.rs` — `handle_filtering_key()` (line ~1873) — the `_` catch-all
  arm that processes character input in `Filtering` mode is the primary insertion point.
  The `KeyCode::Esc`, `KeyCode::Enter`, `KeyCode::Down`, `KeyCode::Up` arms will also
  need autocomplete-accept handling analogous to `handle_editor_key`.

### Task editor autocomplete (reference implementation)
- `crates/todotxt-tui/src/app.rs` — `handle_editor_key()` — existing `@`/`+` token
  autocomplete in the task editor; Phase 42 mirrors this logic in `handle_filtering_key`.
  The exact accept/dismiss/navigate arms in `handle_editor_key` are the reference pattern.

### Requirements
- `.planning/REQUIREMENTS.md` — AC-02, AC-03, AC-04 (Autocomplete Fixes and Coverage section)
- `.planning/phases/39-quick-wins/39-CONTEXT.md` — AC-01 fix context + autocomplete
  keyboard decisions inherited (D-04, D-05, D-06 from Phase 33 via Phase 39)
- `.planning/phases/41-full-presets-filter-history-pane-task-movement/41-CONTEXT.md` —
  D-12 through D-14: FilterHistory mode decisions; D-12 established that `AutocompleteState`
  is reused in `AppMode::Filtering` for history — Phase 42 extends this to token triggers.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `AutocompleteState::new(trigger, prefix, items)` — creates token autocomplete state;
  already used in `handle_editor_key` for `@`/`+`
- `get_existing_contexts(&task_list)` / `get_existing_projects(&task_list)` — deduplicated
  candidate pool functions; called from `handle_editor_key` today
- `rank_matches(prefix, items)` — prefix-first ranked matching; used in existing autocomplete
- `AutocompleteMode::FilterHistory` — shows that `self.autocomplete` is already active in
  `AppMode::Filtering`; Phase 42 adds a second trigger path in the same mode

### Established Patterns
- `handle_editor_key` autocomplete accept (Tab/Enter): reads `self.autocomplete`, extracts
  chosen token, inserts via `self.editor.insert_str()`, sets `self.autocomplete = None`
- `handle_filtering_key` `_` arm: calls `state.editor.input(key)`, extracts filter text,
  currently sets `self.autocomplete` to `FilterHistory` if history is non-empty
- The `_` arm replacement: after `state.editor.input(key)`, extract the current word
  under cursor; if it starts with `@` or `+` → set `TokenAutocomplete`; else if history
  is non-empty → set `FilterHistory`; else → `None`

### Integration Points
- `handle_filtering_key`: `KeyCode::Esc`, `KeyCode::Enter` arms already clear
  `self.autocomplete = None` — these need accept-before-dismiss logic analogous to
  the task editor
- `self.filter_state.as_ref().map(|s| s.editor)` — the `tui_textarea::TextArea`
  instance that receives token insertions (AC-03)

</code_context>

<specifics>
## Specific Ideas

- Post-accept stay-in-editing means Tab/Enter inside the autocomplete popup should
  insert the token but NOT trigger the filter-apply logic in the `KeyCode::Enter` arm.
  The planner needs to guard the Enter arm: if `self.autocomplete` is `Some`, accept
  from popup; only apply the filter if `self.autocomplete` is `None`.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 42-filter-autocomplete-coverage*
*Context gathered: 2026-05-06*

# Phase 33: Fast Capture + Property Pickers - Context

**Gathered:** 2026-04-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 33 delivers fast capture/edit behavior plus picker-driven metadata updates for due date, priority, context, and project tokens.

Within scope:
- Keep add/edit interactions low-friction and keyboard-predictable
- Add due-date setter (`s`) and priority setter (`i`) for active/selected tasks
- Add quick context (`@`) and project (`+`) setters in Normal mode for active/selected tasks
- Add autocomplete for `@`/`+` pickers (match list, arrow navigation, tab-to-complete)
- Add date autocomplete for partial `due:` / `t:` input and align `s` picker options

Out of scope for this phase:
- Workspace switching and file-picker behavior
- Undo stack/recovery implementation details
- New metadata schema beyond todo.txt conventions

</domain>

<decisions>
## Implementation Decisions

### Trigger and Entry Semantics
- **D-01:** In Normal mode, `@` and `+` open quick setters that target selected tasks when `selected_tasks` is non-empty; otherwise they target the active cursor task.
- **D-02:** If no actionable task row is selected (for example header-only focus), `@` / `+` are no-op with a brief status hint rather than mode-switching.
- **D-03:** Quick setters run as lightweight inline picker overlays (same interaction family as existing edit-mode autocomplete), avoiding full-screen mode changes.

### @ / + Autocomplete Behavior
- **D-04:** Candidate source is the deduplicated token corpus already present in tasks (`contexts` for `@`, `projects` for `+`) plus the current typed token.
- **D-05:** Matching is case-insensitive with prefix matches first; near-matches (substring/fuzzy) are shown after prefix matches to expose potentially redundant variants.
- **D-06:** Keyboard behavior is consistent across pickers: Up/Down navigates, Tab/Enter accepts selected candidate, Esc cancels without mutation.

### Token Application Rules
- **D-07:** `@` / `+` quick setters are add-only by default (do not remove existing tokens).
- **D-08:** Applying a token is idempotent per task: duplicates are not added, and all non-target metadata is preserved.
- **D-09:** Bulk token application must use stable canonical index targeting and existing multi-select invariants from phases 19/20.

### Date Autocomplete and Due Picker
- **D-10:** Date autocomplete activates for partial `due:` and `t:` tokens in text-entry flows (add/edit and append-like input surfaces).
- **D-11:** For partial month patterns (for example `due:2026-07-` or `t:2026-07-`), suggestions list valid numeric day values for that month only.
- **D-12:** Date suggestions include weekday labels beside each day (for example `2026-07-14 Tue`) to improve scanability.
- **D-13:** The `s` due-date setter reuses the same month-aware suggestion engine and weekday labeling as typed date autocomplete.
- **D-14:** Invalid partial combinations (month/day out of range) yield no suggestion entries rather than silent coercion.

### Agent's Discretion
- Exact ranking formula for near-match ordering after prefix matches
- Exact popup geometry/placement in relation to footer/editor rows
- Exact weekday label format (`Mon` vs `Monday`) as long as it is consistent
- Whether date suggestions are limited to visible page size with scrolling vs fixed top-N

</decisions>

<specifics>
## Specific Ideas

- User preference: keep the app minimal and fast; avoid complexity that does not directly improve task capture/edit flow.
- User explicitly requested:
  - `@` and `+` quick setters for active/selected tasks
  - Arrow-key list navigation and Tab autocomplete for token setters
  - Date autocomplete for `due:` and `t:` partial input with valid month days and weekday names
  - Parity of date suggestion behavior between typed flows and the new `s` due-date setter

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase and scope authority
- `.planning/ROADMAP.md` — Phase 33 scope and requirement mapping
- `.planning/REQUIREMENTS.md` — CAP-01..CAP-04, TAG-01..TAG-05, DATE-01..DATE-04

### Existing autocomplete and editor behavior
- `crates/todotxt-tui/src/app.rs` — `handle_editor_key`, `collect_tokens`, `update_autocomplete`, `accept_completion`, `render_autocomplete_popup`
- `crates/todotxt-tui/src/state.rs` — `AutocompleteState` model used by popup interactions

### Selection and bulk safety invariants
- `.planning/phases/19-selection-model-multi-select-foundation/19-CONTEXT.md` — canonical selection model invariants
- `.planning/phases/20-bulk-actions-selection-ux/20-CONTEXT.md` — bulk action confirmation and descending-index safety rules

### Keybinding and discoverability conventions
- `.planning/phases/22-keymap-help-parity/22-CONTEXT.md` — keymap strategy and help-surface expectations

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `App.autocomplete: Option<AutocompleteState>` already exists and is rendered via `render_autocomplete_popup`.
- `collect_tokens` and `update_autocomplete` already implement case-insensitive token candidate filtering.
- `handle_editor_key` already supports Up/Down navigation plus Tab/Enter acceptance semantics.

### Established Patterns
- Mode dispatch is centralized in `handle_key_event` and `handle_normal_key` with `key_is_action` lookups.
- Multi-select uses canonical indices (`selected_tasks: HashSet<usize>`) and stable rebuild/reanchor routines.
- High-impact actions already follow confirmation patterns in delete/bulk-delete flows.

### Integration Points
- New Normal-mode handlers for `@`/`+` quick setters likely belong in `handle_normal_key`.
- Token/date picker state can extend existing autocomplete plumbing rather than introducing a separate popup stack.
- Date suggestion generation can plug into editor/picker input update paths where autocomplete refresh is already called.

</code_context>

<deferred>
## Deferred Ideas

- Workspace switching and quick file picker (deferred to later milestone)
- Explicit cross-workspace move command beyond clipboard primitives
- Rich metadata schema beyond todo.txt token model

</deferred>

---

*Phase: 33-fast-capture-property-pickers*
*Context gathered: 2026-04-29*

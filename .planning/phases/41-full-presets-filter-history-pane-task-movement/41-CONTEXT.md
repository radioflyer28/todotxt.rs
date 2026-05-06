# Phase 41: Full Presets, Filter History, Pane Task Movement — Context

**Gathered:** 2026-05-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 41 delivers three independent power-user workstreams:

1. **Full View Presets (PRST-01, PRST-02):** Extend the preset system from filter-only
   to multi-dimensional (filter, sort, group, group_by, pane count + per-pane config).
   New TOML namespaces introduced: `[presets.filter.N]` and `[presets.panes.name]`.
   Old `[presets.f1]` format is silently dropped.

2. **Session Filter History (FHIST-01–03):** A shared app-wide ring buffer (50 entries,
   deduplicated) surfaces recent filter expressions inside the existing filter UI.
   History is recalled via `Ctrl+R` in the filter UI AND via inline autocomplete-style
   suggestions as the user types. No separate recall key in Normal mode.

3. **Pane Task Movement (PMOVE-01–03):** `Ctrl+Left`/`Ctrl+Right` moves the cursor
   task (or all tasks in an active single-pane multi-selection) to the adjacent pane via
   tag mutation. Move wraps at boundaries. Focus jumps to the destination pane after move.

View state persistence (PRSV) is Phase 43. Filter autocomplete coverage (AC-02/03/04)
is Phase 42. Both are explicitly out of scope here.

</domain>

<decisions>
## Implementation Decisions

### Preset Schema — TOML Namespaces (PRST-01)

- **D-01:** Introduce two distinct TOML preset namespaces:
  - `[presets.filter.1]` through `[presets.filter.9]` — filter-only presets applied to
    the active pane when `1`–`9` is pressed (same runtime behavior as today's `[presets.f1]`)
  - `[presets.panes.name]` — full layout presets that define pane count, order, and
    per-pane view settings (filter, sort, group, group_by)
- **D-02:** Old `[presets.f1]`–`[presets.f9]` blocks are **silently dropped** — not
  read at startup. Users must migrate to `[presets.filter.N]`. No deprecation warning.
- **D-03:** `TuiPreset` struct is repurposed/split into two distinct types:
  - `FilterPreset { filter: Option<String> }` — for `[presets.filter.N]`
  - `PaneLayoutPreset { panes: Vec<PaneConfig>, ... }` — for `[presets.panes.name]`
  The planner decides the exact naming and module placement.

### Preset Schema — Pane Layout Content (PRST-01)

- **D-04:** A `[presets.panes.name]` block fully defines the pane layout:
  - Pane count is replicated exactly as declared in the preset's pane list
  - Each pane entry carries: `filter`, `sort`, `group`, `group_by`, `label`
  - Applying the preset replaces all current panes with the preset's pane definitions —
    existing runtime pane state is discarded for the dimensions the preset declares
- **D-05:** Filter presets (`[presets.filter.N]`) remain scoped to the active pane only.
  Applying `[presets.filter.3]` does NOT change pane count or sort/group state — it
  only updates the active pane's filter query.

### Preset Keys (PRST-02)

- **D-06:** `1`–`9` (no modifier) trigger filter presets (`[presets.filter.N]`) —
  unchanged from today's `[presets.f1]` slot behavior.
- **D-07:** `Ctrl+1` through `Ctrl+9` trigger pane layout presets. Since
  `[presets.panes.name]` uses string names, the binding is positional: the first named
  pane preset in config order maps to `Ctrl+1`, second to `Ctrl+2`, etc.
  (up to 9 slots). Slots with no preset defined are no-ops.
- **D-08:** Command palette for named preset activation is a **deferred idea** — noted
  for a future phase. Not in scope here.

### Filter History Ring (FHIST-01–03)

- **D-09:** The filter history ring is **app-wide and shared** — one ring for all panes.
  Applying a filter in any pane adds it to the shared ring.
- **D-10:** Ring capacity: **50 unique entries**. Deduplication is applied on insertion:
  applying the same filter expression twice records only one entry (FHIST-03). When
  the ring is full, the oldest entry is dropped.
- **D-11:** History is **session-only** — not persisted to disk. (Cross-session
  persistence is explicitly deferred per REQUIREMENTS.md.)

### Filter History UX Integration

- **D-12:** History surfaces inside the existing filter panel UI in two ways:
  1. **Inline suggestions while typing:** As the user types in the filter input, history
     entries matching the current prefix appear in the existing autocomplete popup
     (reusing `AutocompleteState` machinery). History suggestions are ranked above
     preset suggestions when the prefix matches.
  2. **`Ctrl+R` in the filter UI:** Cycles through the full history ring, replacing the
     filter input text live. The filter is not applied until the user presses Enter.
- **D-13:** `Ctrl+R` is scoped to the filter input mode (AppMode::Filtering or equivalent).
  It does NOT function as a recall key in Normal mode.
- **D-14:** The existing filter panel layout is preserved — named presets continue to
  display below the filter input as they do today. History-based inline suggestions
  appear in the autocomplete popup, not as a new section.

### Pane Task Movement — Tag Mutation (PMOVE-01–03)

- **D-15:** Selection is always scoped to the **active pane** — no cross-pane
  multi-selection. `Ctrl+Left`/`Ctrl+Right` operates on the cursor task or all tasks
  in the active pane's multi-selection.
- **D-16:** Movement **wraps at boundaries** — `Ctrl+Right` from the last pane moves
  tasks to pane 1; `Ctrl+Left` from pane 1 moves tasks to the last pane.
- **D-17:** After a successful move, **focus jumps to the destination pane**, with
  the cursor landing on the first moved task.
- **D-18:** Move is **declined** (no-op + status bar explanation) if the source pane
  or destination pane has no filter OR has a compound filter (more than one token, or
  any non-tag token). No task data is modified in a declined move.
- **D-19:** For a valid move, the operation on each task is:
  1. Remove the source pane's single-token filter string from the task's raw text
  2. Append the destination pane's single-token filter string to the task
  This is a raw token mutation — no special metadata field; same behavior as existing
  context/project tag appends.
- **D-20:** Undo support: push an undo entry before the move operation (following the
  existing `push_undo_entry()` pattern established in Phase 36).

### Agent's Discretion

- Exact `PaneLayoutPreset` struct shape — planner decides based on cleanest serde
  integration with existing `PaneConfig`
- Whether `Ctrl+1`–`Ctrl+9` preset slots are resolved at config load time or at
  keypress time (map by index into a `Vec<PaneLayoutPreset>`)
- Whether history suggestions and `Ctrl+R` share one `AutocompleteState` instance or
  are differentiated by an `AutocompleteMode` variant
- Status bar message wording for a declined pane move

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Preset system (existing)
- `crates/todotxt-tui/src/config.rs` — `TuiPreset`, `TuiConfig::presets`, `PaneConfig`,
  `GroupByCategory`, `PaneSort` — all types relevant to preset schema extension
- `crates/todotxt-tui/src/app.rs` lines ~1141–1155 — current `1`–`9` preset key handler
  (`KeyCode::Char(c @ '1'..='9')`) — must be updated or split for the new namespaces
- `crates/todotxt-tui/src/app.rs` line ~101 — `pub presets: Vec<(String, String)>` — current
  runtime preset storage shape

### Filter UI and autocomplete (existing machinery to reuse)
- `crates/todotxt-tui/src/state.rs` — `AutocompleteState`, `AutocompleteMode`,
  `FilteringState`, `FilterDefiningState` — history suggestions reuse AutocompleteState
- `crates/todotxt-tui/src/app.rs` — filter input handlers, `AppMode::Filtering` flow

### Pane and multi-selection (existing)
- `crates/todotxt-tui/src/state.rs` — `Pane` struct, `filter_query`, `group_by` fields
- `crates/todotxt-tui/src/app.rs` — `active_pane_mut()`, `pane_add()`, `push_undo_entry()`,
  multi-selection state — pane move builds on these

### Phase 40 artifacts (group_by just added — must not regress)
- `.planning/phases/40-group-by-decoupling-test-coverage/40-CONTEXT.md` — all GRP-01–04
  decisions (D-01–D-18); `GroupByCategory` enum, `group_by_cycle` action, status bar format

### Requirements
- `.planning/REQUIREMENTS.md` — PRST-01, PRST-02, FHIST-01, FHIST-02, FHIST-03,
  PMOVE-01, PMOVE-02, PMOVE-03

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TuiPreset { filter: Option<String> }` in `config.rs` — will be extended/split into
  `FilterPreset` and `PaneLayoutPreset`
- `PaneConfig` in `config.rs` — already has `label`, `filter`, `sort`, `group`,
  `group_by` — reuse directly as the per-pane block inside `PaneLayoutPreset`
- `AutocompleteState` / `AutocompleteMode` in `state.rs` — extend with a `FilterHistory`
  mode for inline history suggestions
- `push_undo_entry()` in `app.rs` — use before pane move mutations (same as Phase 36 pattern)
- `active_pane_mut()` in `app.rs` — use to get source pane for movement

### Established Patterns
- Preset key dispatch: `KeyCode::Char(c @ '1'..='9')` block in `handle_normal_key()`
  already resolved slot → filter; extend to split filter vs layout preset lookup
- Filter history deduplication: mirrors `get_existing_contexts()` / `get_existing_projects()`
  dedup pattern in `state.rs`
- Move op token mutation: mirrors existing `@context` / `+project` append in tag setter flows

### Integration Points
- `TuiConfig::presets` field — change type from `HashMap<String, TuiPreset>` to split fields:
  `filter_presets: HashMap<String, FilterPreset>` and `pane_presets: IndexMap<String, PaneLayoutPreset>`
  (or equivalent; planner decides the exact field names for clean TOML deserialization)
- `App` struct — add `filter_history: VecDeque<String>` field (bounded to 50, deduplicated on insert)

</code_context>

<specifics>
## Specific Ideas

- Filter history suggestions in the autocomplete popup should be ranked **above** regular
  token suggestions (project/context completions) when the typed prefix matches history —
  history recall intent takes priority over tag completion
- `Ctrl+R` in the filter input should behave like shell reverse-search: each press cycles
  one entry backward through the ring, replacing the input text live (no Enter needed to
  cycle, but Enter still required to apply the filter)
- Pane layout presets are positional by config order (first named preset = Ctrl+1) — this
  keeps the keymap simple and avoids requiring explicit numeric keys in the TOML block

</specifics>

<deferred>
## Deferred Ideas

- **Command palette for named preset activation** — user expressed interest; belongs in
  its own phase once the preset infrastructure is in place (likely v1.7+)
- **Cross-session filter history persistence** — explicitly deferred in REQUIREMENTS.md;
  session history covers the core use case

</deferred>

---

*Phase: 41-full-presets-filter-history-pane-task-movement*
*Context gathered: 2026-05-05*

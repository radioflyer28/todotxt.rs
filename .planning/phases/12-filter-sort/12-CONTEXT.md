# Phase 12: Filter + Sort — Context

**Gathered:** 2026-04-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 12 delivers a live-filter panel and sort cycle for the TUI. Users can:
- Press `f` to open a bottom filter panel with a free-text input field and a numbered preset list
- Type a filter query (full CLI filter syntax) and see the task list narrow live as they type
- Navigate the preset list with arrow keys or press 1–9 to instantly load a named preset
- Press `Esc` to close the panel and clear the filter (returns to full unfiltered list)
- Press `o` to cycle through 8 sort orders (file order is the "no sort" baseline)
- See the active filter query and current sort name in the status bar at all times

Phase 11 owns add/edit/delete and autocomplete (complete).
Phase 13 owns theming, colors, and `[tui]` TOML subsection.

</domain>

<decisions>
## Implementation Decisions

### Filter Panel UX

- **D-01: Filter panel style — bottom panel** — The filter panel is rendered below the task list as a bottom panel, similar to the delete-confirm row in Phase 11 (D-06). It does NOT float as an overlay.

- **D-02: Panel contents — text input + preset list** — The filter panel contains:
  1. **Top row:** Free-text input field (tui-textarea, same pattern as Phase 11 editor)
  2. **Below:** A scrollable numbered list of preset names from `[presets]` in config (may be empty if no presets defined)

- **D-03: Filter syntax — full CLI filter syntax via `Filter::from_query`** — The text input accepts the exact same filter tokens as the CLI: `@context`, `+project`, `due:today`, `due:past`, `due:active`, `due:future`, `DONE`, `-DONE`, `-tag`, etc. No custom TUI syntax — reuse `Filter::from_query()` directly.

- **D-04: Live as-you-type filtering** — The task list updates on every keystroke in the filter input. No Enter required to apply. The display view is recomputed after each key event.

- **D-05: Esc closes and clears** — Pressing `Esc` while the filter panel is open:
  1. Clears the filter text input
  2. Clears the active filter (returns to full unfiltered list)
  3. Closes the filter panel (returns to `AppMode::Normal`)
  No "sticky filter" persists after Esc. To keep a filter visible, the user must leave the panel open.

- **D-06: Preset navigation — arrow keys + number keys 1–9** — Two input paths in the filter panel:
  - **Arrow Up/Down:** Navigate the preset list. As the highlighted preset changes, its filter query is loaded into the text input and applied live.
  - **Number keys 1–9:** Instantly load the Nth preset (1-indexed) into the text input and apply live.
  - Number keys still work even when the preset list has fewer than 9 entries (extras are no-ops).
  - The preset list is populated from `TuiConfig.presets` (needs to be added to `TuiConfig`).

### AppMode

- **D-07: New `AppMode::Filtering` variant** — The `AppMode` enum gains a `Filtering` variant. Key handling in `Filtering` mode:
  - Most keys → `editor.input_without_shortcuts()` + recompute display view
  - `Esc` → clear filter + close panel (D-05)
  - `Down`/`Up` → navigate preset list
  - `1`–`9` → load preset by number
  - `Enter` → close panel keeping filter active (optional convenience, decide in planning)
  - All write operations (`n`, `u`, `e`, `d`) should be blocked while filtering is active

### Sort

- **D-08: Sort keybinding — `o`** — Pressing `o` in Normal mode cycles the sort order. No modifier needed.

- **D-09: Sort cycle order — 8 variants including file order** — The cycle is:
  ```
  FileOrder → Alphabetical → CompletedDate → Context → DueDate → CreationDate → Priority → Project → FileOrder
  ```
  - `FileOrder` = no sort applied; tasks displayed in their original file order (position in `TaskList::tasks()`)
  - `CompletedDate` and `CreationDate` are NEW `SortOrder` variants to add to `todotxt-core/src/sort.rs`
  - The existing `SortOrder` is `#[non_exhaustive]` — adding variants is a non-breaking change
  - Tasks without a completion/creation date sort last (consistent with existing `DueDate` behavior)

- **D-10: Display sort is view-only — canonical order never mutated** — `TaskList::sort()` mutates in-place and must NOT be called for display-only sort. The TUI maintains a `display_indices: Vec<usize>` that maps display row position → canonical task index. All read operations (render, selection tracking) use `display_indices`. All write operations (toggle done, add, update, delete) use canonical indices from `display_indices[selected_row]`.

### Display View Architecture

- **D-11: `display_indices: Vec<usize>` on App** — A `Vec<usize>` field on `App` that holds the canonical task indices to display, in display order. Rebuilt whenever:
  - Filter query changes (any keystroke in filter mode)
  - Sort order changes (`o` key)
  - `FileChanged` reload applies (full rebuild)
  - Tasks are mutated (add/update/delete — rebuild preserving selection if possible)
  - `selected` now refers to a position in `display_indices`, not a canonical index. The canonical index is always `display_indices[self.selected]`.

- **D-12: Selection tracking after filter/sort changes** — When the display view is rebuilt:
  - Try to preserve the same canonical index under the cursor. If it's still visible, move `selected` to its new display position.
  - If not visible (filtered out), clamp to 0.

### Status Bar

- **D-13: Status bar shows filter + sort when active** — The status bar format when filters or sort are active:
  ```
  todo.txt | 3/10 tasks | @work due:today | sort: priority
  ```
  - Left section: `{file_name} | {visible}/{total} tasks`
  - Middle section (shown only when filter is non-empty): `| {filter_query}` (truncated with `…` if too long)
  - Right section (shown only when sort is not FileOrder): `| sort: {sort_name}`
  - Right edge: key hints, truncated to fit remaining width

### Config

- **D-14: Add `presets` field to `TuiConfig`** — Mirror the CLI's `presets: HashMap<String, PresetConfig>` in `TuiConfig`. The `PresetConfig` struct lives in `todotxt-cli` — either duplicate the struct in `todotxt-tui` or consider moving it to `todotxt-core`. Decision: duplicate a minimal `TuiPreset { filter: Option<String> }` in `todotxt-tui` (avoids cross-crate dependency on `todotxt-cli`).

### the agent's Discretion

- Panel height: number of rows for the filter panel is flexible — use `min(preset_count + 1, 6)` rows (1 for text input, up to 5 preset rows). Capped so the panel doesn't dominate on large screens.
- Whether `Enter` in the filter panel closes the panel while keeping the filter active — implement this as a convenience.
- Exact display format of preset names in the numbered list (e.g., "1. work — @work" vs "1. work").
- How to handle the `display_indices` rebuild during Phase 11's `pending_reload` apply path.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core Library Filter + Sort
- `crates/todotxt-core/src/filter.rs` — `Filter`, `FilterTerm`, `Filter::from_query()` — the filter engine to reuse directly
- `crates/todotxt-core/src/sort.rs` — `SortOrder` enum — needs `CompletedDate` and `CreationDate` variants added
- `crates/todotxt-core/src/task_list.rs` — `TaskList::filter()`, `TaskList::sort()` — `sort()` mutates in-place; do NOT use for display-only sort

### TUI Codebase
- `crates/todotxt-tui/src/app.rs` — Full app state, `AppMode`, `handle_event()`, `draw()`, `render_task_list()`, `render_status_bar()` — Phase 12 extends all of these
- `crates/todotxt-tui/src/config.rs` — `TuiConfig` — needs `presets` field added

### CLI Config Reference (for preset struct shape)
- `crates/todotxt-cli/src/config.rs` — `Config.presets: HashMap<String, PresetConfig>` — mirror this in `TuiConfig`

### Prior Phase Decisions
- `.planning/phases/11-edit-mode/11-CONTEXT.md` — D-01 through D-13 (AppMode, footer-swap, tui-textarea, keybinding conventions)

### Requirements
- `.planning/REQUIREMENTS.md` — TUI-FILTER-01 through TUI-FILTER-04

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Filter::from_query(q: &str)` — takes a space-separated query string, returns a `Filter` with AND-combined `FilterTerm`s. Zero integration work — call directly on every keystroke.
- `TaskList::filter(&Filter)` — returns `Vec<(usize, &Task)>` — canonical index already preserved.
- `tui_textarea::TextArea` — already used for add/edit; reuse for filter input field with the same `input_without_shortcuts()` pattern.
- `AppMode` enum — `Filtering` variant slots in cleanly with the existing dispatch pattern.

### Established Patterns
- Footer-swap pattern (Phase 11 D-02): in edit mode, the bottom row swaps status bar → editor. Phase 12 extends this: `AppMode::Filtering` shows the filter panel instead.
- Overlay popup pattern: Phase 11 autocomplete overlays. The filter panel is a non-overlay (bottom panel), so no `ratatui::widgets::Clear` needed — it uses the Layout split directly.
- `AppMode` is `Copy` — match arms on `self.mode` release the borrow before mutation.

### Integration Points
- `render_task_list()` currently iterates `self.task_list.tasks()` directly. Phase 12 changes this to iterate `self.display_indices` and render `self.task_list.tasks()[idx]`.
- `render_status_bar()` currently computes `total`, `visible` separately. Phase 12 makes `visible = self.display_indices.len()`.
- `handle_normal_key()` will need `o` for sort cycle and `f` for filter panel open.
- `clamp_selection()` needs to clamp to `display_indices.len()` not `task_list.len()`.

</code_context>

<specifics>
## Specific Ideas

- Sort cycle order confirmed by user: `FileOrder → Alphabetical → CompletedDate → Context → DueDate → CreationDate → Priority → Project → FileOrder`
- Preset UX: arrow keys for visual browsing (loads preset live on highlight), number keys 1–9 for instant load — combining both behaviors
- Filter input is the same tui-textarea pattern used in Phase 11 editor, not a custom widget

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 12-filter-sort*
*Context gathered: 2026-04-20*

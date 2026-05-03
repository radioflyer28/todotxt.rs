# Phase 12: Filter + Sort — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-20
**Phase:** 12-filter-sort
**Areas discussed:** Filter panel layout, Due-date filter granularity, Keybindings, Status bar when filters active

---

## Filter Panel Layout

### How does the filter panel appear?

| Option | Description | Selected |
|--------|-------------|----------|
| Popup overlay over the task list | Floats on top, similar to autocomplete popup. Task list still visible behind it. | |
| Bottom panel (like delete confirm) | Pushes the task list up, occupies several rows at the bottom. No overlap. | ✓ |
| Full-screen takeover | Replaces the entire view while filter is open. | |

**User's choice:** Bottom panel (like delete confirm)

---

### What does the filter panel display?

| Option | Description | Selected |
|--------|-------------|----------|
| List actual @contexts and +projects from task list | Lists the specific @work, @home, +proj1 etc. values that exist in your task list. User navigates and Space-toggles them. | |
| Category toggles only (generic) | Just three toggleable rows: 'contexts', 'projects', 'due date' — user can't pick a specific one. | |
| Free-text filter input | Shows a text input to type a filter query (similar to the CLI's filter syntax). | ✓ |

**User's choice:** Free-text filter input

---

### When does the filter take effect?

| Option | Description | Selected |
|--------|-------------|----------|
| Live (as-you-type) | Task list narrows as you type — instant visual feedback. | ✓ |
| On Enter | Filter is applied when you press Enter; Esc cancels. | |
| Enter applies, Esc clears | Enter applies, Esc clears and closes filter | |

**User's choice:** Live (as-you-type)

---

### What does Esc do when the filter panel is open?

| Option | Description | Selected |
|--------|-------------|----------|
| Esc closes, filter stays | Esc closes the panel and keeps the filter active. Press the filter key again to edit/clear. | |
| Esc closes and clears filter | Esc closes AND resets the filter to empty (no filter). | ✓ |
| Ctrl+R to reset, Esc to close | Only explicit Ctrl+R resets; Esc just closes panel. | |

**User's choice:** Esc closes and clears filter

---

## Due-Date Filter Granularity

### What filter syntax does the input field accept?

| Option | Description | Selected |
|--------|-------------|----------|
| Full CLI filter syntax (reuse Filter::from_query) | The filter input accepts the same tokens as the CLI: due:today, due:past, due:active, @context, +project, -DONE, etc. Full power. | ✓ |
| Substring + context/project only | Only substring and @context / +project matching. No due: tokens. | |
| Custom / Other | I'll define my own simpler syntax. | |

**User's choice:** Full CLI filter syntax

---

## Keybindings

### Which key opens the filter panel?

| Option | Description | Selected |
|--------|-------------|----------|
| f = filter | f = open filter panel | ✓ |
| / = filter | / = open filter panel (search-like) | |
| F = filter (capital) | F = open filter panel | |

**User's choice:** `f`

---

### Which key cycles sort order?

| Option | Description | Selected |
|--------|-------------|----------|
| s = sort cycle | s = cycle sort | |
| o = sort cycle | o = cycle sort | ✓ |
| S = sort cycle (capital) | S = cycle sort (capital) | |

**User's choice:** `o`

---

### What is the full sort cycle?

User provided freeform input: *"file order, alpha, completed date, context, due date, creation date, priority, project"*

**Resolved cycle:** FileOrder → Alphabetical → CompletedDate → Context → DueDate → CreationDate → Priority → Project → FileOrder

**Notes:** `CompletedDate` and `CreationDate` are new `SortOrder` variants to add to `todotxt-core`. `Task` already has `completion_date` and `creation_date` fields.

---

## Status Bar When Filters Active

### What does the status bar show when filters and sort are active?

| Option | Description | Selected |
|--------|-------------|----------|
| Show filter text + sort name (truncated) | Show active filter query text and sort name, truncated with '…' if too long. E.g.: 'todo.txt \| 3/10 \| due:today @work \| sort: priority' | ✓ |
| Counts + sort only, no filter text | Only show counts and sort. Filter text is NOT shown in status bar. | |
| Filter text replaces hint bar when active | Show filter text only when a filter is active, else show the normal hint bar. | |

**User's choice:** Show filter text + sort name (truncated)

---

## Preset Integration (surfaced mid-discussion)

User asked: *"do we retain the ability to predefine filters and toggle them on/off?"*

Decided to include preset support in Phase 12 (not defer).

### Should Phase 12 include preset toggles?

| Option | Description | Selected |
|--------|-------------|----------|
| Defer presets to a later phase | Phase 12 just gets the free-text input. Presets are a Phase 13+ feature. | |
| Include preset list in filter panel now | Phase 12 includes the ability to select named presets from the TOML config. | ✓ |

**User's choice:** Include preset list in filter panel now

---

### How do presets integrate into the filter panel UX?

| Option | Description | Selected |
|--------|-------------|----------|
| Text input + preset list in one panel | When pressing f, show: [1] text input row + [2] scrollable list of preset names below it. Selecting a preset with Enter populates the text input. | ✓ |
| Separate key for preset panel | Separate key (P?) opens preset panel. f always opens the free-text input only. | |
| Inline autocomplete with : prefix | Presets appear as autocompletions when typing : in the filter input. | |

**User's choice:** Text input + preset list in one panel

---

### How does the user navigate and select presets in the panel?

User provided freeform input: *"1 and 3 would be best UI and speed, allows easy viewing of filters and quick selection"*

**Resolved:** Both arrow key navigation AND number key 1–9 instant selection:
- **Arrow Up/Down:** Navigate preset list; as highlight moves, preset is loaded into text input and applied live
- **Number keys 1–9:** Instantly load the Nth preset into the text input and apply live

---

## the agent's Discretion

- Panel height: `min(preset_count + 1, 6)` rows
- Whether `Enter` closes filter panel while keeping filter active
- Exact display format of preset names

## Deferred Ideas

None.

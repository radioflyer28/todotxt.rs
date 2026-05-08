# Phase 40: Group-By Decoupling + Test Coverage — Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 40 delivers two workstreams:

1. **Group-By Decoupling (GRP-01–04):** Each TUI pane gains an independent
   `group_by` dimension (category to group tasks by) that is fully orthogonal to
   the intra-group `sort_order`. Previously the single `sort_order` field drove both
   sorting and grouping categories; after this phase they are independent controls.

2. **Phase 22 Test Coverage (TST-01, TST-02):** All 11 Phase 22 manual-only test gaps
   are closed with automated unit tests, enabled by a new `make_app_with_config` test
   helper that constructs an `App` with a custom `TuiConfig` (including `[keymap]`).
   The 2 status-bar render tests use ratatui's `TestBackend` for headless rendering.

New capabilities (preset system, filter history, pane task movement) are explicitly
out of scope — those belong in Phase 41+.

</domain>

<decisions>
## Implementation Decisions

### GroupByCategory Type (GRP-01)

- **D-01:** Introduce a new `GroupByCategory` enum with exactly four variants:
  `Project`, `Context`, `Priority`, `DueDate`. Do NOT reuse `SortOrder` — the existing
  `SortOrder` has 7 variants (including `FileOrder`, `Alphabetical`, `CompletedDate`,
  `CreationDate`) that make no semantic sense as group-by dimensions.
- **D-02:** `GroupByCategory` derives `Debug, Clone, Copy, PartialEq, Eq, Default,
  Serialize, Deserialize`. Default is `Priority` (matches todotxt.net visual convention).
- **D-03:** `group_key_for()` is refactored to take `&GroupByCategory` instead of
  `&SortOrder`. All call sites updated accordingly.
- **D-04:** The `Pane` struct in `state.rs` gains a `group_by: GroupByCategory` field
  (defaults to `Priority`). This is independent of `sort_order: SortOrder`.

### Config Schema (GRP-04)

- **D-05:** `PaneConfig` in `config.rs` gains `group_by: Option<PaneSortCategory>`
  (new type mirroring `PaneSort` but for the 4 grouping dimensions only, or the planner
  may use the same `GroupByCategory` if it can be made `Deserialize`-compatible directly).
  TOML string values: `"project"`, `"context"`, `"priority"`, `"duedate"` — consistent
  with the existing `PaneSort` naming style.
- **D-06:** The `group_by` field in `PaneConfig` is optional with no default: if absent,
  the runtime default `GroupByCategory::Priority` is used. Existing configs without
  `group_by` continue to work unchanged (full backward compat).
- **D-07:** At startup, `App::new` reads `pane_cfg.group_by` and writes it to
  `pane.group_by` alongside the existing `pane.grouping = pane_cfg.group` logic.

### Key Bindings (GRP-02)

- **D-08:** Add a new keymap action `group_by_cycle` with default binding `g`
  (KeyCode::Char('g'), KeyModifiers::NONE). This action cycles the active pane's
  `group_by` through `Priority → Project → Context → DueDate → Priority…`.
- **D-09:** Change the default binding for the existing `group_toggle` action from `g`
  to `G` (KeyCode::Char('G'), KeyModifiers::NONE), i.e., Shift+g. The action name
  `group_toggle` is unchanged — only the default key changes.
- **D-10:** Keep `sort_cycle` on `o` (unchanged). The `o` key continues to cycle the
  active pane's `sort_order` through all `SortOrder` variants as before.
- **D-11:** Backward compat: users who had `group_toggle = "g"` in their `[keymap]`
  config will see a keymap conflict warning at startup (two actions bound to the same
  key) and both actions will revert to their defaults via the existing Phase 22 conflict
  resolution logic. No special migration code needed.

### Status Bar Display (GRP-03)

- **D-12:** When a pane has grouping enabled (`pane.grouping == true`), the status bar
  shows both the active group-by category and the active sort order. The planner decides
  the exact format — it **must match the visual style of the existing sort indicator** in
  `pane_list.rs` (which shows `sort:alpha`, `sort:priority`, etc. when `sort_order !=
  FileOrder`). Compact and scannable is preferred over verbose label prefixes.
- **D-13:** When grouping is disabled (`pane.grouping == false`), the status bar behavior
  is unchanged from Phase 25 — only the existing sort indicator is shown.

### Phase 22 Test Helper (TST-02)

- **D-14:** The test helper signature is:
  ```rust
  fn make_app_with_config(task_lines: &[&str], config: TuiConfig) -> App
  ```
  More general than `make_app_with_keymap` — accepts a full `TuiConfig` so any future
  test that needs custom config (not just keymap) can also use it.
- **D-15:** For tests that only need a custom keymap, callers construct a `TuiConfig`
  with the `keymap` field populated:
  ```rust
  let mut cfg = TuiConfig::default();
  cfg.keymap.insert("delete".into(), "backspace".into());
  let app = make_app_with_config(&["Task 1"], cfg);
  ```
- **D-16:** `make_app_with_config` is defined in the same `#[cfg(test)] mod tests` block
  as `make_app_with_tasks` — no separate test module needed.

### Phase 22 Test Coverage (TST-01)

- **D-17:** All 11 manual-only gaps are automated:
  - `22-01-G01`, `22-01-G02` (App::new keymap init, default key dispatch) — unit tests
    calling `app.key_is_action(key, action)` assertions on constructed App
  - `22-02-G01` through `22-02-G04` (status bar warnings, `'!'` mode transitions) —
    unit tests using `make_app_with_config` + direct `handle_normal_key()` call + assert
    `app.mode == AppMode::KeymapErrors`; the 2 status-bar render tests use ratatui
    `TestBackend` + `Terminal` for headless rendering
  - `22-03-G01` through `22-03-G05` (filter clear, preset apply, reload, `'?'` help
    overlay, Esc/q from Help) — unit tests via `handle_normal_key()` + assert
    `app.filter_query`, `app.mode`, etc.
- **D-18:** ratatui `TestBackend` is used for the 2 status-bar render tests. ratatui is
  already a main dependency, so `TestBackend` is available in tests without adding new
  Cargo.toml entries.

### Agent's Discretion

- Whether `GroupByCategory` lives in `state.rs` alongside `Pane`, in `config.rs` alongside
  `PaneSort`, or in a new `types.rs` — planner decides based on import cleanliness.
- The exact cycle order for `group_by_cycle` beyond `Priority → Project → Context → DueDate`
  (e.g., starting variant, wrap-around). The requirements say "cycles through group-by
  categories" — Priority→Project→Context→DueDate→Priority is a sensible default.
- Whether the `group_by_cycle` action is gated on `display_count > 0` like `group_toggle`
  currently is — planner should match the existing guard pattern.
- Exact visual format of the status bar combined group+sort indicator — planner must
  match the existing `sort_name` indicator style already in `pane_list.rs`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary integration files
- `crates/todotxt-tui/src/app.rs` — `App` struct, `group_key_for()`, `handle_normal_key()`,
  `key_is_action()`, `render_status_bar()`, `App::new()`, `#[cfg(test)] mod tests`
- `crates/todotxt-tui/src/state.rs` — `Pane` struct, `DisplayRow` enum — `group_by` field added here
- `crates/todotxt-tui/src/config.rs` — `PaneConfig`, `TuiConfig`, `default_keymap()` — new action
  `group_by_cycle` default and `group_toggle` default change go here; `group_by` field in `PaneConfig`
- `crates/todotxt-tui/src/components/pane_list.rs` — `render_status_bar()` or equivalent status bar
  rendering — combined group+sort indicator added here

### Phase 22 artifacts (MUST understand the 11 gaps before writing tests)
- `.planning/phases/22-keymap-help-parity/22-VALIDATION.md` — canonical list of all 11 manual-only
  gaps (22-01-G01 through 22-03-G05) with test instructions
- `.planning/phases/22-keymap-help-parity/22-CONTEXT.md` — keymap architecture decisions (D-01–D-21)

### Requirements
- `.planning/REQUIREMENTS.md` — GRP-01, GRP-02, GRP-03, GRP-04, TST-01, TST-02

### Phase 22 prior decisions to not break
- `.planning/phases/22-keymap-help-parity/22-CONTEXT.md` §Key Bindings — effective_keymap,
  keymap_warnings, conflict detection — Phase 40 must work within this system

</canonical_refs>

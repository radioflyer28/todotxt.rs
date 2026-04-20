# Phase 13: Theming + Polish — Context

**Gathered:** 2026-04-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 13 delivers a visually polished TUI with two switchable color themes, TOML config wiring for theme selection, NO_COLOR support, and verification that terminal restore and auto-reload are robust.

Users can:
- Run with a `default` (dark) or `light` theme selected in `[tui] theme = "..."` config
- See priority `(A)`/`(B)`/`(C)` and overdue tasks in distinct readable colors
- Set `NO_COLOR=1` to strip all color styling while keeping the TUI fully usable
- Rely on the terminal being fully restored on every exit path (normal quit, Ctrl+C, panic)
- See the task list silently re-anchor to the same task when the file changes externally

Phase 12 owns filter panel, sort cycle, status bar filter/sort display (complete).
Phase 13 does NOT change any task actions, keybindings, or layout structure — only visual styling and config wiring.

</domain>

<decisions>
## Implementation Decisions

### Theme Color Palette

- **D-01: Standard richness — priority and overdue colors** — Both themes color the following elements only:
  1. Priority `(A)` — highest urgency color
  2. Priority `(B)` — medium urgency color
  3. Priority `(C)` — lower urgency color
  4. Overdue tasks (due date in the past, task not completed) — red/warning color
  5. Selected row highlight — existing `Modifier::REVERSED` (kept as-is, not a color change)
  6. Completed tasks — existing `Modifier::DIM` (kept as-is, not a color change)

  No context token coloring (`@work`), no project token coloring (`+project`), no status bar background color — those are deferred beyond Phase 13.

- **D-02: Agent picks readable 16-color ANSI palette colors** — The planner and executor pick colors from the 16-color ANSI palette (names like `Color::Red`, `Color::Yellow`, `Color::Cyan`, `Color::Magenta`) that work on both dark and light terminal backgrounds. Do NOT use RGB or `Color::Indexed(n)` — 16-color palette is universally supported. Suggested palette:

  | Element | Dark theme | Light theme |
  |---------|------------|-------------|
  | Priority (A) | `Color::Red` (bright) | `Color::Red` |
  | Priority (B) | `Color::Yellow` | `Color::Yellow` |
  | Priority (C) | `Color::Cyan` | `Color::Cyan` |
  | Overdue | `Color::Red` fg + `Modifier::BOLD` | `Color::Red` fg + `Modifier::BOLD` |
  | Done | `Modifier::DIM` (no color) | `Modifier::DIM` (no color) |
  | Selected | `Modifier::REVERSED` (no color) | `Modifier::REVERSED` (no color) |

  The agent is free to adjust specific shades within the 16-color palette for readability. Differences between dark and light can use light vs dark variants (`Color::Red` vs `Color::LightRed`) where appropriate.

- **D-03: Two named themes — `default` and `light`** — `Theme` is an enum with two variants: `Theme::Default` (dark-terminal palette) and `Theme::Light` (light-terminal palette). The default when no `[tui]` section is present is `Theme::Default`. Unrecognized theme names in config fall back to `Theme::Default` (no panic, no error).

### `[tui]` Config Subsection

- **D-04: `[tui]` block contains `theme` only** — A new `[tui]` TOML subsection is added with a single field:
  ```toml
  [tui]
  theme = "default"   # or "light"
  ```
  All existing top-level fields (`todo_file`, `done_file`, `auto_creation_date`, `presets`) stay at the root level. This is a purely additive change — existing `config.toml` files need no edits and continue to work as before.

- **D-05: `[tui]` is deserialized into a new `TuiSection` substruct** — Add a `tui: TuiSection` field (with `#[serde(default)]`) to `TuiConfig`:
  ```rust
  #[derive(Debug, Deserialize, Default)]
  pub struct TuiSection {
      #[serde(default)]
      pub theme: String,  // "" or "default" → Theme::Default; "light" → Theme::Light
  }
  ```
  Parse the theme string into the `Theme` enum in `main.rs` after loading config. Store `Theme` on `App` (not the raw string).

### NO_COLOR Behavior

- **D-06: NO_COLOR strips colors only — modifiers are preserved** — Follows the [NO_COLOR standard](https://no-color.org/): when `std::env::var("NO_COLOR")` is present and non-empty, all `Color::*` styling is stripped from rendered `Style` values. `Modifier::REVERSED` (selection highlight) and `Modifier::DIM` (done tasks) are NOT stripped — they are not color codes and are essential for TUI usability.

- **D-07: NO_COLOR is checked once at startup, stored as a bool on App** — Do not call `std::env::var("NO_COLOR")` inside render functions (called every frame). Evaluate once in `main.rs` before `App::new()` and pass a `no_color: bool` field to `App`. Render functions branch on `self.no_color` to decide whether to apply `fg`/`bg` color fields in `Style`.

### Theme Architecture

- **D-08: `Theme` enum + `StyleSheet` struct on `App`** — The planner should define:
  ```rust
  pub enum Theme { Default, Light }

  pub struct StyleSheet {
      pub priority_a: Style,
      pub priority_b: Style,
      pub priority_c: Style,
      pub overdue: Style,
      // Extend in future phases for context/project/status bar
  }
  ```
  `StyleSheet::from_theme(theme: Theme, no_color: bool) -> StyleSheet` builds the palette. `App` stores `pub styles: StyleSheet`. Render functions read `self.styles.priority_a` etc. — no in-line `if theme == Light` branching in render code.

- **D-09: Theme applied during `render_task_list()` only** — Priority coloring requires inspecting `task.priority` per row. Overdue requires `task.due_date` and today's date. Both are already available in `render_task_list()`. No other render function needs theme changes in Phase 13.

### Terminal Restore + Auto-Reload Polish

- **D-10: Terminal restore is already correct — verify only** — `TerminalGuard::Drop` disables raw mode and leaves alternate screen. `color_eyre::install()` is called before `TerminalGuard::new()`. Phase 13 verifies this works under: normal `q` quit, Ctrl+C, and a simulated panic. If any gap is found during planning/execution, fix it. If it's clean, no code change needed — just document as verified.

- **D-11: Auto-reload is already correct — verify only** — `FileWatcher` 500ms debounce + `pending_reload` guard + `rebuild_and_reanchor()` already implement TUI-UX-02 and TUI-UX-03. Phase 13 verifies these work correctly under the human-verify checkpoint. No new code needed unless a gap is found.

### the agent's Discretion

- Exact `Style` values per theme (within the 16-color ANSI palette)
- Whether to separate `StyleSheet` into its own module (`crates/todotxt-tui/src/theme.rs`) or keep it in `app.rs`
- How to detect "overdue" in `render_task_list()` — use `task.due_status()` from `todotxt-core` (already returns `DueStatus::Overdue`)
- Whether `priority_c` deserves a color at all or stays plain — the agent can leave it as `Style::default()` if `(C)` priority is common enough that coloring it is noisy

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### TUI Codebase
- `crates/todotxt-tui/src/app.rs` — `render_task_list()`, `render_status_bar()`, `draw()`, `App` struct — Phase 13 adds `styles: StyleSheet` and `no_color: bool` fields; `render_task_list()` gains priority/overdue coloring
- `crates/todotxt-tui/src/config.rs` — `TuiConfig` — needs `tui: TuiSection` field added (D-05)
- `crates/todotxt-tui/src/tui.rs` — `TerminalGuard` — verify Drop correctness (D-10)
- `crates/todotxt-tui/src/main.rs` — startup sequence: config load → NO_COLOR check → theme parse → App::new()

### Core Library
- `crates/todotxt-core/src/task.rs` — `Task.priority: Option<char>`, `Task.due_date: Option<NaiveDate>`, `Task.due_status()` — used in render_task_list() for coloring
- `crates/todotxt-core/src/filter.rs` — `DueStatus` enum (Today, Overdue, Future, None) — use `DueStatus::Overdue` to detect overdue tasks

### Prior Phase Decisions
- `.planning/phases/12-filter-sort/12-CONTEXT.md` — D-13 (status bar format — Phase 13 does not change this)
- `.planning/phases/11-edit-mode/11-CONTEXT.md` — D-01 through D-13 (AppMode, modifiers used — Phase 13 keeps REVERSED and DIM)

### Requirements
- `.planning/REQUIREMENTS.md` — TUI-THEME-01, TUI-THEME-02, TUI-THEME-03, TUI-UX-02, TUI-UX-04

### External Standard
- NO_COLOR standard: https://no-color.org/ — "When set, callers should not add ANSI color escape codes to output" — modifiers (bold, dim, reverse) are not color codes

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `task.priority: Option<char>` — directly inspectable in render_task_list(); `(A)` = `Some('A')` etc.
- `task.due_status()` — returns `DueStatus` enum; `DueStatus::Overdue` is the overdue check
- ratatui `Style` API: `Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)` — clean builder pattern
- `ratatui::style::{Color, Modifier, Style}` already imported in app.rs

### Established Patterns
- `render_task_list()` already branches on `t.completed` to apply `Modifier::DIM` — same pattern extended for priority/overdue
- `AppMode` is `Copy` — `Theme` enum should also derive `Copy, Clone, PartialEq`
- `TuiConfig` already uses `#[serde(default)]` for optional fields — same pattern for `TuiSection`

### Integration Points
- `App::new()` takes `(task_list, todo_path, presets)` → Phase 13 adds `theme: Theme, no_color: bool` (or wraps in a config struct)
- `main.rs` already evaluates `config.presets` after loading — same place to evaluate `config.tui.theme` and `NO_COLOR`

</code_context>

<specifics>
## Specific Decisions

- Theme richness: Standard — priority (A/B/C) colors + overdue red. No context/project token coloring. No status bar background.
- `[tui]` block: theme field only. Existing root-level fields unchanged. Additive, no migration.
- NO_COLOR: strips Color styling only. `Modifier::REVERSED` (selection) and `Modifier::DIM` (done) preserved.
- Color palette: agent picks from 16-color ANSI palette. No RGB. No indexed colors.
- Terminal restore and auto-reload: already implemented. Phase 13 verifies correctness only.

</specifics>

<deferred>
## Deferred Ideas

- Context token (@work) and project token (+project) coloring — not Phase 13 scope
- Status bar background color — not Phase 13 scope
- User-configurable color overrides (per-element color in config) — future milestone
- Mouse support — already deferred to v1.2 in REQUIREMENTS.md

</deferred>

---

*Phase: 13-theming-polish*
*Context gathered: 2026-04-20*

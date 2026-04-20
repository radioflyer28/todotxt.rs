# Phase 13: Theming + Polish — Research

**Date:** 2026-04-20
**Level:** 1 — Quick verification (codebase patterns confirmed; all decisions pre-locked in CONTEXT.md)

---

## Confirmed Codebase Patterns

### ratatui Style API (confirmed from `app.rs`)
```rust
// Builder pattern — already used in render_task_list() and render_status_bar()
Style::default().add_modifier(Modifier::DIM)
Style::default().add_modifier(Modifier::REVERSED)

// Color extension (pattern confirmed — Color not yet imported in render_task_list):
Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)

// Imports needed in theme.rs:
use ratatui::style::{Color, Modifier, Style};
```

### 16-Color ANSI Names (ratatui 0.29)
Available `Color::*` variants (no RGB needed):
`Color::Red`, `Color::LightRed`, `Color::Yellow`, `Color::LightYellow`,
`Color::Cyan`, `Color::LightCyan`, `Color::Magenta`, `Color::LightMagenta`,
`Color::Blue`, `Color::LightBlue`, `Color::Green`, `Color::LightGreen`,
`Color::Gray`, `Color::DarkGray`, `Color::White`, `Color::Black`

### DueStatus (confirmed from `render_status_bar()`)
```rust
use todotxt_core::DueStatus;
// Used as: tasks[ci].due_status() == DueStatus::Overdue
// task.priority: Option<char> → Some('A'), Some('B'), Some('C'), None
```

### App::new() Current Signature (confirmed)
```rust
pub fn new(task_list: TaskList, todo_path: PathBuf, presets: Vec<(String, String)>) -> Self
```

Phase 13 extends this to:
```rust
pub fn new(task_list: TaskList, todo_path: PathBuf, presets: Vec<(String, String)>, theme: Theme, no_color: bool) -> Self
```

### TuiConfig serde pattern (confirmed from `config.rs`)
```rust
// Existing pattern for optional subsections:
#[serde(default)]
pub presets: HashMap<String, TuiPreset>,

// Same pattern for TuiSection:
#[serde(default)]
pub tui: TuiSection,
```

### Terminal Restore (D-10 confirmed — verify only)
`tui.rs:TerminalGuard::Drop` calls `disable_raw_mode()` + `execute!(LeaveAlternateScreen)` unconditionally.
`color_eyre::install()` in `main.rs` installs panic hook BEFORE `TerminalGuard::new()`. Correct order confirmed.

### Auto-Reload (D-11 confirmed — verify only)
`app.rs:handle_event(AppEvent::FileChanged)`:
- Normal mode → immediate `task_list.reload()` + `rebuild_and_reanchor()`
- Other modes → `pending_reload = true` (applied on mode exit via `apply_pending_reload()`)
`FileWatcher` in `main.rs` uses 500ms debounce from `todotxt-core`. Correct behavior confirmed.

---

## Selected Palette (D-02 — Agent discretion within 16-color ANSI)

| Element | Dark theme (`default`) | Light theme (`light`) | Rationale |
|---------|------------------------|----------------------|-----------|
| Priority (A) | `Color::LightRed` | `Color::Red` | Bright variant pops on dark; standard red on light |
| Priority (B) | `Color::Yellow` | `Color::Yellow` | Universal — readable on both |
| Priority (C) | `Color::Cyan` | `Color::Cyan` | Universal — readable on both |
| Overdue | `Color::LightRed` + `Modifier::BOLD` | `Color::Red` + `Modifier::BOLD` | Highest urgency signal |
| Done | `Modifier::DIM` only | `Modifier::DIM` only | Per D-01 — no color change |
| Selected | `Modifier::REVERSED` only | `Modifier::REVERSED` only | Per D-01 — no color change |

NO_COLOR mode: all `Color::*` stripped → `Style::default()` or `Modifier::BOLD` only.

---

## Module Structure Decision (Agent discretion from CONTEXT.md)

**Decision:** Separate `crates/todotxt-tui/src/theme.rs` module.

**Rationale:** `Theme` enum and `StyleSheet` struct are self-contained and will grow in future phases (context/project token coloring). Keeping them in `app.rs` would couple styling logic to event handling. `mod theme;` declared in `main.rs` (binary crate — no `lib.rs`).

---

## Dont-Hand-Roll

- `Style::default().fg(Color::X)` — use ratatui's builder, never construct `Style { fg: Some(Color::X), .. }` manually
- `#[serde(default)]` on the substruct field — serde handles missing `[tui]` block without custom Deserialize impl
- `std::env::var("NO_COLOR")` — use `is_ok()` for presence check per NO_COLOR standard (value content is irrelevant)

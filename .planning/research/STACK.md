# Stack Research — v1.6

**Researched:** 2026-05-04
**Scope:** Rust crate dependency additions for todotxt-tui v1.6 features

## Summary

All ten v1.6 features can be implemented with the existing workspace dependency set. The workspace already carries `tempfile`, `toml`, `serde`, `crossterm`, and `ratatui 0.29` at versions sufficient for every requirement. No new runtime or dev dependencies are needed.

## Existing Dependencies (relevant)

| Crate | Version | Purpose in v1.6 |
|-------|---------|-----------------|
| `ratatui` | =0.29.0 | Popup/overlay rendering (filter history list, autocomplete in filter input); `Clear` + `Block` + `Layout` already sufficient |
| `crossterm` | =0.28.1 | Raw-mode suspend/resume for `$EDITOR` launch (`disable_raw_mode` / `enable_raw_mode` / `LeaveAlternateScreen` / `EnterAlternateScreen`) |
| `tui-textarea` | =0.7.0 | Filter input widget already in use; autocomplete narrowing extends existing state machine |
| `toml` | =0.8.23 | `tui-state.toml` sidecar read/write for view state persistence (Feature 8); `TuiPreset` expansion (Feature 7) |
| `serde` | =1.0.228 | Serialize/deserialize new `GroupBy` enum and expanded `TuiPreset` fields — `#[serde(rename_all = "snake_case")]` already used in `config.rs` |
| `tempfile` | =3.27.0 | Already a workspace dep and already listed in `todotxt-tui` `[dev-dependencies]`; used by existing integration tests (`fallback_test.rs`, `single_pane_test.rs`, `view_continuity_test.rs`) |
| `directories` | =6.0.0 | Already used for config path resolution; same `ProjectDirs` call suffices for `tui-state.toml` placement |
| `chrono` | =0.4.44 | No new use; unchanged |

## New Dependencies Needed

| Crate | Version | Feature | Justification |
|-------|---------|---------|---------------|
| — | — | — | None required |

## Dependencies That Are NOT Needed

| Feature | Initially Considered | Why Not |
|---------|---------------------|---------|
| Open task in `$EDITOR` (Feature 3) | `tempfile` (new runtime dep) | `tempfile` is **already** a workspace dep used by `todotxt-cli` and as a dev-dep in `todotxt-tui`. Moving it to `todotxt-tui` runtime deps is a one-line `Cargo.toml` edit, not a new addition. `std::env::temp_dir()` + `std::fs` would also suffice to avoid even that move. |
| Open task in `$EDITOR` (Feature 3) | `nix` / `libc` for `SIGTSTP` suspend | Not needed. Pattern: call `disable_raw_mode()` + `execute!(LeaveAlternateScreen)`, spawn editor with `std::process::Command::new(editor).status()`, then re-enter raw mode + alternate screen. No UNIX signal manipulation required; `crossterm` handles terminal state. |
| Filter history persistence (Feature 6) | `serde_json` as persistence format | `toml` already in deps and already used for config; a `[[history]]` array-of-strings section is straightforward. JSON would be a second format for no benefit. |
| View state persistence (Feature 8) | New serialization crate | `toml` + `serde` cover the `tui-state.toml` sidecar entirely. The existing `PaneConfig` / `TuiConfig` round-trip pattern (`toml::to_string` / `toml::from_str`) is the established approach in this codebase. |
| `GroupBy` enum + serde (Feature 9) | Any additional derive macro crate | `serde`'s `#[serde(rename_all = "snake_case")]` already handles enum serialization — see `PaneSort` in `config.rs` as the exact pattern to follow. |
| Phase 22 test automation (Feature 10) | `mockall` or other mocking crate | `App` state-machine tests drive the state struct directly without rendering; `tempfile` (already a dev-dep) handles temp todo files. No mock framework needed. |
| Popup/overlay for filter history & autocomplete (Features 5, 6) | `tui-popup` or similar crate | `ratatui 0.29` ships `Clear`, `Block`, and `Layout::split` sufficient for centered overlay rendering. The existing autocomplete popup already demonstrates this pattern. |

## Integration Notes

**`$EDITOR` suspend/resume pattern** (Feature 3):
```rust
// Suspend TUI
disable_raw_mode()?;
execute!(io::stdout(), LeaveAlternateScreen)?;

// Launch editor (blocks until editor exits)
let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
std::process::Command::new(&editor)
    .arg(&task_file_path)  // or a NamedTempFile with the task text
    .status()?;

// Resume TUI
enable_raw_mode()?;
execute!(io::stdout(), EnterAlternateScreen)?;
terminal.clear()?;
```
`TerminalGuard`'s `Drop` impl already calls `disable_raw_mode` + `LeaveAlternateScreen`, so the suspend path must bypass or temporarily disarm the guard. Cleanest approach: add `suspend()`/`resume()` method pair to `TerminalGuard`.

**`tui-state.toml` sidecar** (Feature 8): Use the same `directories::ProjectDirs` call already in `config.rs` to locate the data directory. Struct: a new `TuiViewState` with `#[derive(Serialize, Deserialize, Default)]`; missing or malformed file silently defaults — same pattern as `TuiConfig`.

**`GroupBy` enum** (Feature 9): Follow the `PaneSort` pattern in `config.rs` exactly — `#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, Default)]` with `#[serde(rename_all = "snake_case")]`. Add `group_by: GroupBy` field to both `TuiPreset` and `PaneConfig` with `#[serde(default)]` so existing configs remain valid.

**Filter history ring buffer** (Feature 6): Implement as `VecDeque<String>` with a configurable cap (e.g., 50). No external crate needed. For cross-session persistence, serialize as a TOML array in `tui-state.toml` (Feature 8 sidecar) rather than a separate file.

**`tempfile` promotion** (Feature 3): If a temp-file buffer is preferred over editing the live todo file, promote `tempfile` from `todotxt-tui`'s `[dev-dependencies]` to `[dependencies]`. Not a new crate — already in the workspace at `=3.27.0`.

## Risks

- **`tempfile` promotion**: Promoting `tempfile` to a runtime dep in `todotxt-tui` adds negligible binary size; the crate is already compiled for dev builds.
- **`toml` serde round-trip for new enum variants**: Adding `GroupBy` or new `TuiPreset` fields **must** use `#[serde(default)]` on every new field, or existing user configs will fail to deserialize. Correctness constraint, not a dependency concern.
- **ratatui 0.29 popup z-ordering**: ratatui has no native z-index system. Overlapping widgets are rendered in draw-call order (last wins). The existing autocomplete popup already handles this; filter history popup must follow the same pattern (render last in the draw cycle).
- **Windows + `$EDITOR`**: `std::process::Command` works on Windows, but `EDITOR` is rarely set. Feature should fall back gracefully (warn in status bar) when `EDITOR` is unset. No platform-specific crate needed.
- **No `tokio` in workspace**: Confirmed — the TUI crate uses a synchronous `crossterm` event loop. All v1.6 features are synchronous state mutations. No async runtime is introduced.

# Changelog

## [1.4.0] - 2026-04-28

### Added
- **Config-defined panes (`[[panes]]`)** — TUI startup can now load pane blueprints from `config.toml`, including per-pane `label`, `filter`, `sort`, and `group` defaults.
- **Safe pane fallback behavior** — Invalid pane entries (for example invalid `sort` values) are skipped with warnings while valid pane entries continue loading.
- **TUI startup path overrides** — `todotxt-tui` now supports `--todo`, `--archive`, and `--config` path flags.
- **Archive sibling fallback** — When `--todo` is provided without `--archive`, archive defaults to `done.txt` in the same directory as the selected todo file.

### Changed
- **Release alignment for v1.4** — Workspace crates are versioned to `1.4.0` for milestone ship readiness.

## [1.3.0] - 2026-04-28

### Added
- **Multi-select + range selection** — Press `v` to enter selection mode; `Space` toggles tasks; `Shift+j/k` extends a contiguous range; `Shift+Ctrl+D/U` extends by half page. Selected tasks survive filter, sort, and reload.
- **Bulk delete** — Press `D` (Shift+d) with tasks selected to enter a count-aware confirmation (`Delete N tasks?`). Deletion runs in descending index order to prevent corruption. After delete: selection cleared.
- **Bulk append** — Press `T` (Shift+t) with tasks selected to open an append prompt. Text is appended to all selected tasks atomically via `batch_update`.
- **Smart text normalization** — Append and edit flows normalize recognized todo.txt metadata: priority tokens moved to canonical prefix, +project and @context tags merged/deduplicated, `due:` and `t:` dates handled with field precedence, unknown metadata preserved verbatim. Normalization is done in `todotxt-core` and toggleable via `normalize_append` / `normalize_edit` config flags.
- **Configurable keymap** — Add a `[keymap]` section to `config.toml` to override action bindings (e.g. `move_down = "j"`). Supports: letter keys, modifier combos (`ctrl+d`), named keys (`backspace`, `enter`, `SPACE`, `f1`–`f12`). All 19 implemented actions are overridable.
- **Keymap conflict detection** — Invalid chords and conflicting bindings fall back to defaults. A warning indicator appears in the status bar; press `!` to see the full warning list.
- **Help overlay** — Press `?` to open a keybindings reference showing all active bindings (including user overrides) in 5 sections: Tasks, Filter, View, Select, App. Press `Esc` or `q` to close.
- **Parity hotkeys** — `0` clears the active filter, `1`–`9` apply filter presets, `.` reloads from disk. All aligned with todotxt.net defaults.
- **Selection count indicator** — Status bar shows `| N selected` when tasks are selected and hints for bulk action keys.
- **CLI `--normalize` flag** — `todotxt-cli` gains `--normalize` to apply smart normalization when adding or appending via the command line.

### Notes
- Deliberate deviations from todotxt.net WPF behavior are documented in `.planning/phases/22-keymap-help-parity/DEVIATION.md` (DEV-01 through DEV-07). Key differences: Shift+nav for range selection (todotxt.net uses mouse), no drag-and-drop priority setting, bulk append is text-only (no per-field batch edit).

## [1.2.0] - 2026-04-23

### Added
- **todo.sh compatibility layer** — `todo.sh` command aliases (`add`/`a`, `do`/`x`, `list`/`ls`, `listall`/`lsa`, `listpri`/`lsp`, `del`/`rm`, `append`/`app`, `prepend`/`prep`, `pri`/`p`, `depri`/`dp`, `archive`/`arc`) are now recognized. Use `--compat` flag for numbered `{N} {task}` output format.
- **Deferred task support (`t:` threshold dates)** — Tasks with a future `t:YYYY-MM-DD` are hidden from the default list. TUI: press `h` to toggle deferred task visibility (shown with DIM styling and `[+deferred]` status indicator). CLI: use `--all` flag to include deferred tasks.
- **TUI task grouping** — Press `g` in the TUI to group tasks by the active sort key (project, context, priority, etc.). Group headers appear as reversed-style rows; navigation skips headers automatically.
- **TUI filter Esc/restore** — Pressing `Esc` in the filter panel now restores the previously confirmed filter instead of clearing it.
- **Persistent filter presets** — F-key filter presets are saved to TOML config and restored on startup.

### Changed
- **TUI status bar** — Removed the theme label from the status bar for a cleaner display. Status now shows sort order, active filter, grouping state, and deferred toggle.

## [1.1.0] - 2026-04-23

### Added
- TUI interface (`todotxt-tui`) — keyboard-driven terminal UI with add/edit/delete, filtering/sorting, presets, and themes.

## [1.0.0] - 2026-04-16

### Added
- Initial release: `todotxt-core` parser + `todotxt-cli` with 25+ commands, JSON output, TOML config, shell completions, cross-platform support.

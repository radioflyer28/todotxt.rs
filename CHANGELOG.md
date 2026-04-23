# Changelog

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

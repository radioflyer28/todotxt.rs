# Plan 43-02 Summary — Wiring TuiStateFile into startup and quit

## Status: COMPLETE

## Commit
`a2ff4b1` — feat(43): wire TuiStateFile into startup and quit flow (PRSV-01/02/03)

## What Was Built

### main.rs
- After `TuiConfig::load`, derive `state_path` and call `TuiStateFile::load`
- If the state file loads and has non-empty panes, override `config.panes` before `App::new`

### app.rs — Struct field
- `startup_pane_snapshot: Vec<crate::config::PaneConfig>` added to `App` struct
- Initialized in `App::new()` from `config.panes.clone()` before consuming config

### app.rs — save_view_state (replaces persist_panes_on_quit)
- `save_view_state(&self)` computes `current` pane config from live pane state
- Skips write if `current == startup_pane_snapshot` (D-06, compare-on-quit)
- Writes to `tui-state.toml` via `TuiStateFile::save` — `config.toml` is NEVER written (PRSV-03)
- `run()` calls `save_view_state()` on clean quit

### Tests updated (pane_integration_test.rs)
- `test_quit_persists_runtime_panes_into_config` — now reads `TuiStateFile` from state path
- `test_persisted_pane_data_contains_only_config_fields` — now reads state file; uses `tempfile::tempdir()` for isolation
- `test_no_pane_write_occurs_until_quit_persist_path` — now checks state file existence; uses `tempfile::tempdir()` for isolation

## All Tests: PASS
- 212 unit tests + 18 pane integration tests + all other integration test suites: 0 failures

## Deviations from Plan
- Used `&self` (not `&mut self`) on `save_view_state` since it doesn't mutate app state
- Used `tempfile::tempdir()` for 2 tests where parallel execution caused shared-path collisions

## Files Modified
- `crates/todotxt-tui/src/main.rs` (+8 lines)
- `crates/todotxt-tui/src/app.rs` (+27, -15 lines)
- `crates/todotxt-tui/tests/pane_integration_test.rs` (+12, -12 lines)

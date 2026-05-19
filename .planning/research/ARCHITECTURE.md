# Architecture Research

## Recurring tasks integration

- Add recurrence token parsing in `todotxt-core` task model layer first.
- Expose recurrence metadata as data, not a side-effectful operation, so CLI and TUI can share behavior.
- Add completion orchestration in command/app layer where current done/archive semantics already live.

## Filter OR integration

- Keep query grammar in `todotxt-core/src/filter.rs`; avoid duplicating parser logic in the TUI.
- Expose `FilterTerm` evaluation that naturally composes:
  - AND terms at top level
  - OR term with short-circuit `any` semantics.
- Add focused unit tests in `filter.rs` test module and lightweight TUI behavior tests for selected edge cases.

## TUI visual polish integration

- Render behavior changes in list component (`pane_list.rs`) and focus logic in `app.rs`.
- Preserve existing keyboard model; avoid creating new key remapping paths.
- Ensure row-selection math and focus restoration are unchanged except for new rendering constraints.

## done.txt rotation integration

- Place rotation decision after archive append and before returning success.
- Treat rotation failures as non-fatal to archive append when safe; when unsafe, return clear warning output.
- Keep undo/quick task expectations explicit since rotated done logs and snapshots have different scopes.


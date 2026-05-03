# 15-02 Summary — Command Implementations

## Status: COMPLETE

## What was done

### `list.rs` — wired `--all` and `--compat` flags
- `build_filter` returns a mutable filter; after construction, `--all` sets `suppress_future_threshold = false` and `suppress_hidden = false`
- `run` branches on `args.compat`: emits `"{id+1} {raw_task}"` per line when true, otherwise uses renderer (table output)
- 1-based numbering: `id + 1` (filter returns 0-based indices; table display uses `idx + 1`)

### `listpri.rs` (new)
- Parses priority spec: single letter `"A"` → exact match; range `"A-C"` → inclusive range
- Loads TaskList, applies `-DONE` base filter, post-filters by priority range
- Invalid spec → `CliError::Other` (exit 2)

### `listall.rs` (new)
- Loads `todo.txt` and `done.txt` (config `done_file` or `{todo_dir}/done.txt`)
- Uses `Filter { suppress_hidden: false, suppress_future_threshold: false, ..default() }` to show all tasks
- `done.txt` not present → handled gracefully (no error)
- Prints both sections; count = total from both

### `deduplicate.rs` (new)
- Tracks seen raw strings in `HashSet<String>`, collects duplicate indices
- Deletes in reverse order (preserving index validity)
- `TaskList::delete()` auto-saves after each removal
- Prints "No duplicate tasks found." or "Removed N duplicate task(s)."

### `mod.rs`
- Added `pub mod deduplicate;`, `pub mod listall;`, `pub mod listpri;`

### `main.rs`
- Added dispatch arms: `Commands::Listpri`, `Commands::Listall`, `Commands::Deduplicate`

## API discoveries
- `TaskList::filter()` returns `Vec<(usize, &Task)>` with **0-based** indices (enumerate-based)
- `build_task_table` converts to 1-based by doing `id + 1` for display
- `Renderer` has no `print_message` — used `println!` directly
- `Filter` struct fields are public — struct update syntax works

## Deviations
- None

## Verification
- `cargo build` → exit 0 ✅

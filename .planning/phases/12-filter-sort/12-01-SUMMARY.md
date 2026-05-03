# Phase 12, Plan 01 — SUMMARY

## What Was Built

Extended `SortOrder` in `todotxt-core` with three new variants (`FileOrder`, `CompletedDate`, `CreationDate`) and their `compare()` implementations. Added a `display_indices: Vec<usize>` layer to the `App` struct that decouples display row order from canonical task storage order, enabling non-destructive sort and filter. All navigation, write operations, and rendering now route through `display_indices`, with the `o` key cycling through 8 sort orders.

## Tasks Completed

- **Task 1:** Extend SortOrder with FileOrder, CompletedDate, CreationDate variants + compare() arms — `fe0eece`
- **Task 2:** Add display_indices architecture to App struct (FilteringState, AppMode::Filtering, new fields, rebuild_display_indices, rebuild_and_reanchor, canonical_selected, clamp_selection update, cycle_sort, sort_name) — `f6e4962`
- **Task 3:** Wire display_indices through all callers (handle_normal_key, toggle_done, handle_delete_confirm_key, save_and_exit, apply_pending_reload, FileChanged, render_task_list, render_status_bar, render_delete_confirm) and update main.rs App::new() — `34c7ba0`

## Verification

- cargo build -p todotxt-core: ✓ (zero warnings)
- cargo build -p todotxt-tui: ✓ (zero warnings)
- SortOrder has 8 variants: ✓ (Priority, DueDate, Alphabetical, Project, Context, FileOrder, CompletedDate, CreationDate)
- display_indices wired throughout: ✓
- `o` key cycles sort: ✓

## Decisions Made

- Used `#[allow(dead_code)]` on `FilteringState`, `AppMode::Filtering`, `filter_state`, `presets`, and `sort_name()` since they are Plan-02 stubs that would otherwise trigger `#[deny(warnings)]` errors in the binary crate.
- `App::new()` now accepts `presets: Vec<(String, String)>` as third argument; `main.rs` passes `Vec::new()` as placeholder.
- `save_and_exit()` Adding/Editing branches call `rebuild_display_indices()` directly (not `rebuild_and_reanchor()`) to allow precise selection positioning of the newly added/edited task.
- `filter_query` is a `String` (empty = no filter) rather than `Option<String>` as specified in RESEARCH.md; this matches the Plan-01 spec.

## Handoff to Plan 02

Plan 02 can rely on:
- `App::display_indices: Vec<usize>` — populated by `rebuild_display_indices()`
- `App::sort_order: SortOrder` — current sort, defaults to `SortOrder::FileOrder`
- `App::filter_query: String` — set this and call `rebuild_and_reanchor()` to apply a filter live
- `App::filter_state: Option<FilteringState>` — field exists, initialized to `None`; `FilteringState { editor: TextArea<'static>, selected_preset: usize }` struct is defined
- `App::presets: Vec<(String, String)>` — wired from `App::new()` third arg; main.rs passes `Vec::new()` for Plan 02 to replace with real config presets
- `AppMode::Filtering` variant — defined, key dispatch arm is a no-op; Plan 02 adds `handle_filtering_key()`
- `rebuild_and_reanchor()` — call after any filter/sort change to recompute display and preserve selection
- `canonical_selected()` — returns `Option<usize>` into task_list for the selected display row
- `cycle_sort(SortOrder) -> SortOrder` — free function, already wired to `o` key
- `sort_name(SortOrder) -> &'static str` — free function, Plan 02 should add it to the status bar display

## Self-Check

- `fe0eece` exists: ✓
- `f6e4962` exists: ✓
- `34c7ba0` exists: ✓
- sort.rs FileOrder variant: ✓
- display_indices on App: ✓
- cycle_sort called by o key: ✓

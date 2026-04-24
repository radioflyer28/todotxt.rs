# Research Summary — v1.3 Feature/Hotkey Parity with todotxt.net

**Synthesized:** 2026-04-24  
**Sources:** local `Client/` WPF app, `Client/Resource.resx`, current Rust TUI, todo.txt format primer  
**Confidence:** HIGH

## Executive Summary

The v1.3 milestone should focus on one core migration problem: todotxt.net users expect multi-selection and bulk task operations to feel natural, while the current Rust TUI is still fundamentally single-selection. The C# app already exposes extended selection and routes many task commands over all selected items; the Rust TUI has the rendering and mutation foundation to add that, but it needs a proper canonical selection model first.

The second opportunity is deliberate improvement over parity: todotxt.net appends raw text to selected tasks, but the Rust stack can use `todotxt-core::Task` parsing and rebuild logic to normalize recognized metadata so `(A)` stays at the front, dates stay in valid positions, and known tokens are deduplicated or replaced predictably.

## Stack additions

- No new UI framework or parser is needed.
- Reuse `ratatui`, `crossterm`, `tui-textarea`, and `todotxt-core::Task`.
- Prefer adding normalization helpers in `todotxt-core` over implementing token surgery in the TUI.

## Feature table stakes

- Range selection in the TUI
- Disjoint multi-selection mode
- Bulk delete and bulk append over selected tasks
- Selection persistence across regroup/reload/filter changes
- Updated help and status text for new parity hotkeys

## Watch out for

- Selection drift if state is stored by visible row rather than canonical task identity
- Broken bulk deletes if indices are applied in ascending order
- Over-aggressive text rewriting that mutates unknown content
- Silent parity deviations that undermine user trust

## Recommended build order

1. Selection model and row-highlighting semantics
2. Bulk delete / append plumbing
3. Smart normalization helpers and edit-path integration
4. Hotkey/help parity pass and verification
- `rebuild_visible_tasks()` runs after every mutation, reload, filter change, or sort change; re-anchors selection by prior `source_index`.
- `pending_reload: bool` defers file-watch reloads while in Add / Edit / DeleteConfirm mode.

### Overlay rendering order (hard rule)

Delete confirmation -> autocomplete popup -> editor -> base task list. Rendered last = drawn on top. Input routed to top active layer only.

---

## Watch Out For

Top 5 pitfalls — each has a one-line prevention.

| # | Pitfall | Prevention |
|---|---------|------------|
| 1 | **Reload during active edit clobbers user input** | In Add/Edit/DeleteConfirm, set `pending_reload = true`; apply reload only after save or cancel |
| 2 | **Mutating by visible row instead of `source_index`** | Every write call uses `visible_tasks[sel].source_index`; never use `sel` or `display_id` directly |
| 3 | **Panic / early return leaves terminal in raw mode** | `tui.rs` guard type with `Drop` restore; `color-eyre` panic hook calls `ratatui::restore()` first |
| 4 | **TUI's own saves trigger spurious file-watch reloads** | After each local mutation, mark one watcher event as self-originated before coalescing events |
| 5 | **Workspace dependency skew (duplicate `crossterm`)** | Add `ratatui`, `crossterm`, `tui-textarea` to `[workspace.dependencies]`; `cargo tree -d` check in Foundation |

**Honorable mentions:** Filter `KeyEventKind::Press` only (prevents key duplication on Windows). Configure `tui-textarea` in single-line mode explicitly (prevents Enter/newline leaking into add/edit). Always recompute layout from `frame.area()` per draw (prevents stale geometry after resize). Start from ratatui 0.30 docs only, not community blog posts (prevents `Frame::size()` and import breakage).

---

## Build Order Recommendation

Suggested phase sequence for the roadmapper, in dependency order:

| Phase | Name | Delivers | Rationale |
|-------|------|----------|-----------|
| 1 | **Foundation** | New `todotxt-tui` crate, workspace deps pinned, `tui.rs` terminal guard, `color-eyre` panic hook, tokio event loop skeleton, watcher bridge, config loader | Must be airtight before any visible work. Terminal restore bugs are the hardest to retrofit. |
| 2 | **Core TUI** | Task list rendering, navigation (`j`/`k`/`g`/`G`/half-page), mark-done toggle, status bar, sort toggle, visible-list rebuild with `source_index` anchoring | Gets the app usable for read + mark-done. All identity and selection patterns established here. |
| 3 | **Edit Mode** | Add new task (`a`), inline edit (`e`), delete confirmation (`d`), `@`/`+` autocomplete popup, deferred reload during edit | Layered on top of the stable state model. Overlay ordering and single-line textarea config solved here. |
| 4 | **Filter Panel** | Filter sidebar (`f`), text search, context/project toggles, due-date bucket, show-done toggle, live ANDed filtering, `Ctrl+R` reset, narrow-terminal overlay fallback | Depends on stable visible-list rebuild from Phase 2. |
| 5 | **Theming** | `Theme` struct, `default`/`light` built-ins, `[tui] theme` config, custom theme TOML, `NO_COLOR` support, priority/done/overdue/selected color slots | Isolated rendering concern; no state dependencies. Can run in parallel with Phase 4 if needed. |
| 6 | **Polish** | Scrollbar (D2), quick search `/` (D5), help overlay `?` (D6), Unicode width fixes, narrow-terminal edge cases, `TestBackend` render smoke tests, state-machine unit tests | Everything that raises quality without blocking core functionality. |

**Research flags for planning:**
- Phase 1 (Foundation): standard patterns — no deep research needed; ratatui quickstart template covers >80%.
- Phase 3 (Edit Mode): consider a brief research pass on `tui-textarea` single-line configuration to confirm the exact API for disabling multiline defaults in v0.7.
- Phase 4 (Filter Panel): standard patterns — ratatui `List` + `Block` sidebar is well-documented.
- Phase 6 (Polish): `ratatui::backend::TestBackend` usage worth a quick research pass before writing tests.

---

## Sources

- STACK.md: crates.io (ratatui 0.30.0, crossterm 0.29.0, tui-textarea 0.7.0, color-eyre 0.6.5), ratatui.rs event-handling docs
- FEATURES.md: taskwarrior-tui, gitui, lazygit UX patterns; ratatui widget docs; todo.txt format spec
- ARCHITECTURE.md: ratatui component architecture guides; todotxt-core existing API surface
- PITFALLS.md: crossterm 0.29 docs (KeyEventKind, resize); ratatui 0.30 breaking changes; tui-textarea 0.7 docs; todotxt-core watcher and TaskList source
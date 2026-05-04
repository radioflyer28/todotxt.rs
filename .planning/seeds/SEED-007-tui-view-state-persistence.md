---
id: SEED-007
status: dormant
planted: 2026-05-04
planted_during: v1.5 complete / preparing v1.6
trigger_when: next milestone (v1.6)
scope: Medium
---

# SEED-007: TUI view state persistence (sort, group, filter, panes) across restarts

## Why This Matters

Users expect view tweaks to survive restarts. Right now every session starts from the static `config.toml` defaults — any runtime changes to sort order, grouping, active filter, or pane arrangement are silently discarded on exit. Power users who rely on the TUI daily will repeatedly re-apply the same adjustments, which is friction that erodes confidence in the tool.

## When to Surface

**Trigger:** Next milestone (v1.6).

This seed should be presented during `/gsd-new-milestone` when the milestone scope matches any of these conditions:
- TUI pane or view features are being extended
- TUI configuration or settings work is planned
- Quality-of-life / polish work is in scope

## Scope Estimate

**Medium** — Requires decisions about:
- What state to persist (sort order, filter query, grouping toggle, active pane index, pane labels)
- Where to persist it (sidecar `.tui-state.toml` next to `config.toml`? XDG state dir? same `config.toml`?)
- Migration path for users who don't have a state file yet (graceful absence = use config defaults)
- Whether per-pane state is persisted independently or as a single snapshot

The `Pane` struct already carries `sort_order`, `filter_query`, `grouping`, and `label` fields — they're in place but runtime changes aren't saved anywhere.

## Breadcrumbs

Relevant code in the current codebase:

| File | Notes |
|------|-------|
| `crates/todotxt-tui/src/state.rs` line 26–56 | `Pane` struct with `sort_order`, `filter_query`, `grouping`, `label` — the fields to persist |
| `crates/todotxt-tui/src/config.rs` line 37–65 | `PaneSort` enum + `to_sort_order`/`from_sort_order` helpers — serialisation foundation already exists |
| `crates/todotxt-tui/src/config.rs` line 31–34 | `filter` field in named presets — related concept |
| `crates/todotxt-tui/src/app.rs` line 184–221 | `App::new()` — startup init where state would be loaded |
| `crates/todotxt-tui/src/config.rs` line 126–169 | `resolve_paths()` — already resolves `archive_path`; same pattern could derive a state file path |

## Notes

The C# todotxt.net app persists view state via `User.settings` (see `Client/User.settings`, `Client/User.cs`). The Rust TUI should offer equivalent persistence without being tied to the Windows registry.

A lightweight approach: write a `tui-state.toml` (or `.tui-state.toml`) next to `todo.txt` on clean exit, and load it at startup before applying any `config.toml` pane defaults. If the file is absent or malformed, fall back silently to `config.toml` values.

Also consider: only persist state for panes that were interactively modified, not for config-file-defined panes whose source of truth should remain the config.

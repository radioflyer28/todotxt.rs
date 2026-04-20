# Plan 13-01 Summary: Theme Module + Startup Wiring

**Status:** Complete
**Commit:** a049036
**Date:** 2026-04-20

## What Was Built

Created `crates/todotxt-tui/src/theme.rs` — new module with:
- `Theme` enum (`Default`, `Light`) with `from_str()` (unknown names → `Theme::Default`, never panics)
- `StyleSheet` struct with `priority_a`, `priority_b`, `priority_c`, `overdue` fields
- `StyleSheet::from_theme(theme, no_color)` — precomputes styles once at startup

Wired through three existing files:
- `config.rs`: Added `TuiSection { theme: String }` struct + `tui: TuiSection` field on `TuiConfig` with `#[serde(default)]`
- `app.rs`: Added `styles: StyleSheet` field on `App`; updated `App::new()` to accept `theme: Theme, no_color: bool`
- `main.rs`: Added `mod theme;`; checks `NO_COLOR` env var once at startup; parses `config.tui.theme`; passes both to `App::new()`

## Key Decisions Honored

- D-03: `Theme::from_str()` uses `_ => Theme::Default` — unrecognized names never panic
- D-04/D-05: `[tui]` block has only `theme`; existing root-level fields unchanged; backward-compatible
- D-06/D-07: `NO_COLOR` checked once at startup via `std::env::var("NO_COLOR").is_ok()`; modifiers preserved
- D-08: `StyleSheet::from_theme()` builds palette; `App` stores `styles: StyleSheet`

## Deviation

The CONTEXT.md specified storing `no_color: bool` on `App` (D-07). This was omitted from the struct because render functions read `self.styles.*` (pre-computed) rather than branching on `self.no_color` directly. The `no_color` parameter is passed to `StyleSheet::from_theme()` in `App::new()` and then discarded. `#![deny(warnings)]` would have flagged a dead_code error otherwise.

## Acceptance Criteria

- [x] `theme.rs` exports `Theme`, `StyleSheet`, `StyleSheet::from_theme()`
- [x] `TuiConfig` has `tui: TuiSection` with `#[serde(default)]`
- [x] `App::new()` accepts `theme: Theme, no_color: bool`
- [x] `main.rs` checks `NO_COLOR` once and parses theme from config
- [x] `cargo check --package todotxt-tui` exits 0 with zero warnings

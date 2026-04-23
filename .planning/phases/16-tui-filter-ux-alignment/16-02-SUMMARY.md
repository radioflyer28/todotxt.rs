---
plan: 16-02
phase: 16-tui-filter-ux-alignment
status: complete
completed: "2026-04-23"
---

# Plan 16-02: TuiConfig Serialization + save() — SUMMARY

## What Was Built

Added TOML serialization capability and an atomic `save()` method to `TuiConfig` so that
preset definitions written in the definition panel (Plan 16-03) can be persisted to disk.

## Changes Made

**`crates/todotxt-tui/src/config.rs`**
- `use serde::{Deserialize, Serialize}` — added `Serialize` to import
- `TuiSection`, `TuiPreset`, `TuiConfig` all derive `Serialize` (in addition to `Deserialize`)
- Added `#[serde(skip_serializing_if = "Option::is_none")]` to `TuiConfig::todo_file` and `done_file` — prevents inserting null TOML fields where they were absent
- Added `TuiConfig::save(path: &Path) -> color_eyre::Result<()>` with atomic write: serialize to `.toml.tmp`, then rename to final path (T-16-02-01)
- Annotated `save()` with `#[allow(dead_code)]` until Plan 16-03 adds call sites

## Self-Check: PASSED

- `cargo check -p todotxt-tui` exits 0 with no errors
- `TuiConfig`, `TuiPreset`, `TuiSection` all derive `Serialize`
- `TuiConfig::save(path: &Path)` exists and uses atomic write via temp file + rename
- `skip_serializing_if` prevents spurious null fields in output TOML

## Key Files

- `crates/todotxt-tui/src/config.rs` — modified

## Commits

- `762a2f1` feat(16-02): add Serialize derives + atomic save() to TuiConfig

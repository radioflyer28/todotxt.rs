# Phase 49 Research: Archive Hygiene

**Phase:** 49-archive-hygiene
**Date:** 2026-05-19
**Status:** Planned

## Summary

Phase 49 should extend the existing archive flow rather than replace it. The current CLI and
TUI already append completed tasks into `done.txt` and rewrite `todo.txt` atomically. This
phase adds time-based archive rotation on top of that path, with monthly rotation as the
initial shipped policy and explicit user messaging when rotation occurs.

## Existing Shape

- CLI archive behavior is centralized in `crates/todotxt-cli/src/commands/archive.rs`.
- TUI archive behavior is centralized in `crates/todotxt-tui/src/app.rs`.
- Both archive paths already resolve `done.txt`, append completed tasks, and rewrite files in
  a predictable order.
- Neither config surface currently exposes archive-rotation settings.
- Existing tests already cover archive append semantics and TUI archive confirmation flows.

## Recommended Implementation

Introduce a shared archive-rotation contract that CLI and TUI both call:

- add a rotation cadence concept with monthly as the default shipped value.
- determine the current active archive period from the wall-clock date at archive-write time.
- before appending new completed tasks, detect whether the existing `done.txt` belongs to an
  earlier period.
- when rotation is needed, rename the current `done.txt` into a deterministic period file such
  as `done-2026-05.txt`, then start a fresh active `done.txt`.
- keep all rotated files for now; retention cleanup stays out of scope.

The rotation decision should happen only when an archive write is already occurring. This
matches the phase context and avoids surprising background rotation on startup or open.

## Config Direction

Phase 49 should replace the old threshold-oriented planning language with cadence-based
configuration:

- monthly rotation ships now.
- configuration shape should leave room for future cadences such as weekly without requiring a
  breaking redesign.
- retention and cleanup policy should not be implemented in this phase.

## Plan Split

1. `49-01` - Shared archive cadence/config contract and rotation helper.
2. `49-02` - CLI archive rotation integration and messaging.
3. `49-03` - TUI archive rotation integration and messaging.

## Documentation Reconciliation

Current milestone docs still describe threshold-based rotation and retention cleanup. Phase 49
context supersedes that. Planning should update requirements, roadmap, and project wording to
describe time-based monthly rotation with configurable cadence and no cleanup yet.

## Testing Targets

Core and shared tests should cover:

- default monthly cadence parsing or resolution.
- deterministic period naming such as `done-2026-05.txt`.
- rotation decision logic when the existing archive period differs from the current archive
  write date.

CLI tests should cover:

- archive with completed tasks rotates old `done.txt` into the prior period bucket when needed.
- archive in the same period appends without rotation.
- rotation emits explicit user-facing messaging.

TUI tests should cover:

- archive confirmation flow still works when rotation occurs.
- rotated archives preserve prior done entries and new completed tasks land in the new active
  `done.txt`.
- undo and existing archive messaging boundaries remain clear after rotation-aware writes.

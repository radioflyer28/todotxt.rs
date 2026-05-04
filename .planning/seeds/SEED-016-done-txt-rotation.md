---
id: SEED-016
status: dormant
planted: 2026-05-04
planted_during: v1.6 Phase 39 discuss
trigger_when: when archive workflow is mature and done.txt grows large enough to be a concern
scope: Small
---

# SEED-016: Rotate done.txt like Linux log files

## Why This Matters

As users archive completed tasks over time, `done.txt` grows unbounded. During the v1.6
Phase 39 discuss, we noted that snapshotting done.txt for undo purposes becomes
increasingly inefficient as the file grows. More broadly, a large done.txt slows reads,
makes searching unwieldy, and has no native cap.

Log rotation (like `logrotate` on Linux) offers a proven pattern: when `done.txt` exceeds
a threshold (size or line count), rename it to `done.txt.1`, start a fresh `done.txt`,
and optionally compress older rotations. This keeps the active done.txt small while
preserving full history.

## When to Surface

**Trigger:** When archive workflow (`ARCH-*` features from v1.6) is shipped and in use,
and/or when a user-facing "manage done.txt" feature is being planned.

This seed should be presented during `/gsd-new-milestone` when the milestone scope
matches any of these conditions:
- Milestone includes done.txt / archive management features
- Milestone includes performance improvements to file I/O
- User reports done.txt growing too large

## Scope Estimate

**Small** — Configuration option (e.g., `done_rotate_kb = 1024`) plus rotation logic
in the archive path. Likely 1 phase: add threshold config, implement rename+create in
the archive function, add undo awareness (undo would only need to snapshot the active
`done.txt`, not the rotated files).

## Breadcrumbs

- `crates/todotxt-cli/src/commands/archive.rs` — `run_archive()` is where rotation
  would be triggered (after appending completed tasks, before returning)
- `crates/todotxt-tui/src/config.rs:92` — `done_file: Option<PathBuf>` — rotation
  config option would live nearby (e.g., `done_rotate_kb: Option<u64>`)
- `crates/todotxt-tui/src/config.rs:133` — `archive_path: PathBuf` resolved at startup
- `crates/todotxt-core/` — no changes needed; rotation is a file-management concern

## Notes

Came up during v1.6 Phase 39 ARCH-B discussion: undo for archive was proposed to
snapshot both `todo.txt` and `done.txt`. We opted to snapshot todo.txt only (partial
undo) because snapshotting a large done.txt is memory-inefficient. If done.txt rotation
were in place, the active done.txt would always be small and full snapshot undo would
become practical again — so this seed and the undo strategy are linked.

Possible rotation config options to consider:
- `done_rotate_kb` — rotate when done.txt exceeds N kilobytes
- `done_rotate_lines` — rotate when done.txt exceeds N lines
- `done_keep_rotations` — how many rotations to keep (0 = no limit)

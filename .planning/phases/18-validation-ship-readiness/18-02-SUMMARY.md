---
phase: 18-validation-ship-readiness
plan: "02"
status: complete
completed: 2026-04-23
wave: 2
---

# Plan 18-02 Summary: Requirements Close-out + CHANGELOG + README

## What Was Built

Closed out all v1.2 requirement tracking and created/updated user-facing documentation for the v1.2 release.

### Task 1: REQUIREMENTS.md — All 9 v1.2 requirements marked complete

All 9 v1.2 requirements changed from `[ ]` to `[x]`:
- V12-COMPAT-01, V12-COMPAT-02 (todo.sh compat layer — Phase 15)
- V12-TUI-FILTER-01, V12-TUI-FILTER-02, V12-TUI-FILTER-03 (filter UX — Phase 16)
- V12-TUI-STATUS-01, V12-TUI-GROUP-01 (status polish, grouping — Phase 17)
- V12-TUI-DEFER-01, V12-TUI-DEFER-02 (deferred task parity — Phases 14/17)

Traceability table updated: all rows `Planned` → `Complete`.

### Task 2: CHANGELOG.md created

New file at repo root documenting v1.0.0, v1.1.0, and v1.2.0 releases. v1.2.0 section covers: todo.sh aliases, deferred task support, TUI grouping, filter Esc/restore, persistent filter presets, and status bar cleanup.

### Task 3: README.md updated

Added a **Features** section between the intro paragraph and the Table of Contents, listing: CLI commands, todo.sh compat aliases, TUI features (grouping, deferred toggle, persistent presets), deferred task semantics, format support, and cross-platform support.

## Key Files

### Modified
- `.planning/REQUIREMENTS.md` — all 9 V12-* requirements now `[x]`, traceability table shows Complete
- `README.md` — added Features section with v1.2 highlights

### Created
- `CHANGELOG.md` — v1.0.0, v1.1.0, v1.2.0 release notes

## Self-Check: PASSED

- [x] All 9 `V12-*` requirements show `[x]` in REQUIREMENTS.md
- [x] No `[ ] V12-` entries remain
- [x] CHANGELOG.md exists at repo root with a `[1.2.0]` section
- [x] README.md has a Features section mentioning todo.sh compatibility and TUI grouping/deferred toggle

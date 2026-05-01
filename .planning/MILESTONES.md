# Milestones

## v1.5 Capture Flow + Bulk Safety + Clipboard + Undo (Shipped: 2026-05-01)

**Phases:** 6 (Phases 33-38) | **Plans:** 14 | **Requirements:** 27/27 complete (5 adjusted)

**Key accomplishments:**

1. Delivered fast capture/edit flows with due and priority pickers plus quick context/project setters.
2. Added month-aware date autocomplete with weekday labels and keyboard navigation parity.
3. Added safer bulk-action confirmation UX and metadata-preserving mutation paths.
4. Added clipboard copy/paste workflows including Adding-mode paste support.
5. Added short-horizon undo infrastructure and mutation-site wiring across high-impact operations.
6. Backfilled verification artifacts for phases 33/34/37 and resolved META-01 orphan evidence.

**Known deferred items at close:** 5 requirement-level debt notes (CLIP-01..04 runtime human checks, UNDO-03 override) and Nyquist coverage gaps for phases 33/36/37/38.

**Archive:** .planning/milestones/v1.5-ROADMAP.md | .planning/milestones/v1.5-REQUIREMENTS.md | .planning/v1.5-MILESTONE-AUDIT.md

---

## v1.1 TUI Interface (Shipped: 2026-04-23)

**Phases:** 5 (Phases 9–13) | **Plans:** 14 | **Requirements:** 25/25 complete

**Key accomplishments:**

1. Delivered `todotxt-tui` crate and full terminal lifecycle/event loop integration with `todotxt-core`.
2. Implemented full keyboard-driven task operations: navigation, toggle done, add/edit/delete with confirmation.
3. Added query filtering, sort cycling, preset integration, and status-bar context visibility.
4. Added `@`/`+` autocomplete and queued reload guard during edit-mode flows.
5. Added theme system (default/light), TOML theme config, and `NO_COLOR` behavior.
6. Closed milestone blocker by updating file-watch debounce to 500ms and archived audit as tech debt only.

**Archive:** .planning/milestones/v1.1-ROADMAP.md | .planning/milestones/v1.1-REQUIREMENTS.md | .planning/milestones/v1.1-MILESTONE-AUDIT.md

---

## v1.0 Core Library + CLI (Shipped: 2026-04-16)

**Phases:** 8 (Phases 1–8) | **Plans:** 30 | **Tests:** 207 passing

**Key accomplishments:**

1. Built `todotxt-core` Rust library: single-pass winnow parser, immutable Task model, atomic file writes
2. Implemented full filter + sort engines with BOM/CRLF handling and portable mode (CORE-01..08)
3. Built `todotxt-cli` with 25+ commands: list, show, stats, projects, contexts, add, do, undo, del, edit, append, prepend, pri, depri, due, postpone, archive, del-done, completions
4. Structured JSON output (`--json`), exit code contract (0/1/2), `--no-color`/`--quiet` flags, TOML config + named filter presets
5. Cross-platform validation (Windows/Linux/macOS), `#![deny(warnings)]` enforced, 5 E2E integration tests
6. 207 tests passing, 0 clippy warnings, CI workflow in place

**Archive:** `.planning/milestones/v1.0-ROADMAP.md` | `.planning/milestones/v1.0-REQUIREMENTS.md`

---

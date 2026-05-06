# Retrospective

## Milestone: v1.0 — Core Library + CLI

**Shipped:** 2026-04-16
**Phases:** 8 | **Plans:** 30 | **Tests:** 207

### What Was Built

- `todotxt-core`: single-pass winnow parser, immutable Task model, atomic file writes, filter/sort engines, file watching, BOM/CRLF handling, portable mode
- `todotxt-cli`: 25+ commands — read (list, show, stats, projects, contexts), write (add, do, undo, del, edit, append, prepend), enrichment (pri, depri, due, postpone), bulk (archive, del-done)
- CLI conventions: structured JSON output, exit code contract (0/1/2), --no-color/--quiet, TOML config, named presets, shell completions
- Cross-platform validation (Windows/Linux/macOS), `#![deny(warnings)]`, 5 E2E integration tests, CI workflow

### What Worked

- **Cargo workspace**: clean separation of core library and CLI from day 1 — no circular dependency issues
- **winnow parser**: single-pass, zero-copy; composable combinators made the todo.txt grammar easy to express
- **`#![deny(warnings)]` from day 1**: caught regressions immediately during development
- **Phase-by-phase planning**: small focused phases (2-6 plans each) kept context manageable
- **JSON output mode**: enabled integration tests to be assertion-exact without parsing human text

### What Was Inefficient

- **Missing VERIFICATION.md artifacts** in Phases 04, 05, 06: required two retroactive phases (07 + 08) to close gaps — 5 extra plans of documentation work
- **Traceability table not updated at phase completion**: REQUIREMENTS.md still showed "Pending" for CORE-01..08 and WRITE-01..07 at audit time — required manual fix
- **`audit-open` gsd-tools bug**: ReferenceError at milestone close — skipped without impact but indicates tooling needs a fix

### Patterns Established

- Phase VERIFICATION.md must be produced at phase execution time, not retroactively
- REQUIREMENTS.md traceability table should be updated in the same commit as phase SUMMARY.md
- Integration tests in a separate `tests/` directory with fixture helpers enable clean E2E coverage
- Both `#![deny(warnings)]` at crate root AND `cargo clippy -- -D warnings` in CI catch different classes of issues

---

## Milestone: v1.6 — TUI Fixes and Power User Improvements

**Shipped:** 2026-05-06
**Phases:** 7 | **Plans:** 18 | **Tests:** 215

### What Was Built

- Archive workflow: `AppMode::ArchiveConfirm`, atomic `done.txt` append via `tempfile`, `archive_tasks()` with RAII undo entry
- Bulk mark-done (`bulk_mark_done()`), external editor (`RawModeGuard`, `resolve_editor()`, `Ctrl+E`), `+` autocomplete verified correct
- `GroupByCategory` enum + per-pane `group_by` field — fully decoupled from sort order; automated all 11 Phase 22 manual test gaps
- Multi-dimensional view presets (`[presets.filter.*]` / `[presets.panes.*]` TOML type system), session filter history ring with `Ctrl+R`, pane task movement via tag mutation
- `compute_filter_autocomplete` free function (cursor-aware, incremental narrowing); `accept_filter_completion` with borrow-safe `AcceptResult` enum
- `TuiStateFile` struct — `tui-state.toml` sidecar for per-pane view state persistence (sort, group_by, grouping, filter)
- BUG-41-01 TDD fix: `KeyModifiers::NONE` guard on Left/Right pane-nav arms — Ctrl+Left/Right now reach `pane_move_task()`

### What Worked

- **TDD RED→GREEN for BUG-41-01 (Phase 44)**: writing failing regression tests before the fix made the root cause immediately obvious and produced a 2-line fix
- **Phase 45 verification backfill pattern**: writing VERIFICATION.md as a dedicated phase (not ad hoc) forced systematic review of each phase's evidence and caught the Phase 39 ROADMAP tracking gap
- **3-source cross-reference in audit**: checking VERIFICATION.md × SUMMARY.md × REQUIREMENTS.md simultaneously ensured nothing slipped through
- **`LocalResult` / `AcceptResult` enum pattern**: Rust borrow checker demanded the enum approach for `accept_filter_completion`; the pattern is now established for future autocomplete wiring

### What Was Inefficient

- **REQUIREMENTS.md traceability table not updated during execution** (again): same issue as v1.0 — all 29 checkboxes were still `[ ]` at audit time and required a bulk update in the audit step; should be updated per-phase at plan execution time
- **Verification backfill required a full dedicated phase (Phase 45)**: 4 VERIFICATION.md files were missing at audit time (phases 39/40/41/43) — this is the same root cause as v1.0 and means the "produce VERIFICATION.md at execution time" pattern hasn't stuck yet
- **Seed registry stale**: 10 of 12 seeds flagged by `audit-open` were actually addressed by v1.6 phases but still showed `dormant` — seed status should be updated when work is completed
- **BUG-41-01 was shipped in Phase 41 incomplete**: the pane_move_task method was correct but unreachable via keyboard until Phase 44 added the modifier guard — the key dispatch path should be tested end-to-end at time of implementation, not after audit

### Patterns Established

- Borrow-safe autocomplete wiring: extract action into local `XxxResult` enum before dropping `&mut self.autocomplete` — apply to any future autocomplete accept path
- `KeyModifiers::NONE` guard on navigation keys: whenever adding a new Ctrl+Key binding, verify the unmodified variant won't catch it first — add regression test at implementation time
- `TuiStateFile` sidecar pattern: config.toml = startup defaults, tui-state.toml = session overrides — apply to any new per-session state (e.g., future filter history persistence)

### Key Lessons

- Write VERIFICATION.md in the same commit as SUMMARY.md — never defer to a backfill phase
- Update REQUIREMENTS.md traceability and checkboxes in the same commit as plan execution
- Test Ctrl+Key bindings with both the modified and unmodified key variant in the same test run to catch dispatch shadowing immediately
- Mark seeds as `shipped` in the same commit that closes the corresponding phase

---

## Cross-Milestone Trends

(To be populated after v1.1+)

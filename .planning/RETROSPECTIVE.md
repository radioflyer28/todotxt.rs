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

## Cross-Milestone Trends

(To be populated after v1.1+)

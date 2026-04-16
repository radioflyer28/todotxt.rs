# Milestones

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

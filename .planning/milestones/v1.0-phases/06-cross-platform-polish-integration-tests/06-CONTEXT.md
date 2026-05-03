---
phase: 06
created: 2026-04-15
status: locked
---

# Phase 06 Context — Cross-Platform Polish + Integration Tests

## Scope

Phase 6 delivers the "ship it" hardening layer: end-to-end integration tests, compiler hardening (`#![deny(warnings)]`), README documentation, and a CI placeholder. All Phases 1–5 requirements are validated end-to-end.

## Canonical Refs

- `.planning/ROADMAP.md` — Phase 6 deliverables and UAT criteria
- `.planning/REQUIREMENTS.md` — CORE-08 (portable mode), CFG-01 (platform paths), READ-08 (exit codes)
- `crates/todotxt-core/src/lib.rs` — where `#![deny(warnings)]` is added
- `crates/todotxt-cli/src/main.rs` — where `#![deny(warnings)]` is added
- `crates/todotxt-cli/tests/` — existing test pattern for `assert_cmd` + `tempdir`
- `crates/todotxt-core/tests/` — platform test target directory

## Decisions

### D-01: End-to-end integration test scope

**Decision:** Scenario tests only — 3–5 real-world workflow scenarios covering the most important user journeys. No per-command regression suite.

**Rationale:** Per-command tests already exist in `write_tests.rs` and `enrich_bulk_tests.rs`. The integration tests in `tests/integration/` add scenario-level coverage, not duplication.

**Scenarios to cover:**
1. Full workflow: `add` → `list` → `do` → `stats` → `archive` (smoke test)
2. Filter + sort: `list +project`, `list @context`, `list --sort priority`
3. JSON output round-trip: every command with `--json`, validate `schema_version: 1`
4. Exit code contract: invalid ID → exit 1, invalid args → exit 2, success → exit 0
5. Cross-command state: add → due → postpone → archive (enrichment + bulk pipeline)

### D-02: JSON contract tests

**Decision:** Yes — all integration `--json` tests must explicitly parse and assert `schema_version: 1` present in output. Use `serde_json::from_str` in test assertions.

**Rationale:** The `schema_version` field is a contract for AI agent consumers. Silently breaking it is a high-severity regression.

### D-03: README audience and structure

**Decision:** Human + AI agent as co-equal first-class audiences.

**Sections to write (all 7):**
1. Installation (cargo install + pre-built binary placeholder)
2. Quick Start (5-command walkthrough)
3. Full command reference table
4. JSON schema documentation (fields, schema_version, error envelope format)
5. Config file format + preset example
6. Shell completion instructions (bash/zsh/fish/powershell)
7. **todo.txt format primer** — explain the format for new users who may not know it

**Agent-friendly sections:** JSON schema section must be written to be machine-parseable — structured table with field names, types, and examples. Not prose.

### D-04: Compiler hardening scope

**Decision — #![deny(warnings)]:** Both crates: `todotxt-core/src/lib.rs` AND `todotxt-cli/src/main.rs`.

**Decision — todo!()/unimplemented!() removal:** Full implementation — no placeholders remaining. Every `todo!()` must be either implemented or converted to a specific `CliError` variant (no panics in production paths).

**Decision — .unwrap() audit:** Core library only (`crates/todotxt-core/src/`). Zero `.unwrap()` in library code outside test files. CLI code may retain `.expect("message")` with descriptive messages.

### D-05: CI yml placeholder

**Decision:** Multi-OS matrix template (ubuntu/macos/windows) — but initially with macOS and Windows jobs commented out. Structure anticipates SEED-004 expansion. Layout:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest]
    # os: [ubuntu-latest, macos-latest, windows-latest]  # Uncomment for SEED-004
```

**Decision — Triggers:** `on: push + pull_request` to `main` only.

**Jobs in ci.yml:**
- `test`: checkout + setup-rust stable + `cargo test --workspace`
- (commented) `clippy`: `cargo clippy --workspace -- -D warnings`
- (commented) `doc`: `cargo doc --workspace --no-deps`

## Patterns (from prior phases)

- `assert_cmd` + `assert_fs`/`tempdir` for CLI integration tests
- `#[cfg(target_os = "...")]` or `#[ignore]` guards for platform-specific tests
- JSON envelope: `{"schema_version":1,"data":{...}}` (established in Phase 3)
- Exit codes: 0 = success, 1 = ID not found, 2 = validation error (locked Phase 3)
- Atomic saves via `NamedTempFile::persist()` (locked Phase 1)
- All new public-facing tests follow `test_{verb}_{scenario}` naming (Phase 4 pattern)

## Phase Structure (Expected Plans)

- **Wave 1:** Compiler hardening (deny warnings, todo! removal, unwrap audit) — foundational
- **Wave 2:** Platform tests (`crates/todotxt-core/tests/platform.rs`) — parallel with Wave 2b
- **Wave 2b:** E2E integration tests (`tests/integration/`) — parallel with Wave 2
- **Wave 3:** CI yml + README — final deliverables

## Deferred Ideas

- Full multi-OS CI matrix (SEED-004 — not in this milestone)
- Pre-built binary release workflow (SEED-004)
- Benchmark suite for parser performance
- Man page generation (`cargo man`)

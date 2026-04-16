---
plan: 06-03
phase: 06
status: complete
commit: 696a2b9
---

# 06-03 Summary — E2E Integration Tests

## One-liner
Created 5 end-to-end scenario tests in `integration_tests.rs` covering full CLI lifecycle, filters, JSON `schema_version` contract for 5 commands, exit code contract, and enrichment pipeline.

## What was done
- Created `crates/todotxt-cli/tests/integration_tests.rs` (322 lines, 5 test functions)
- **Scenario 1** — Full workflow smoke: add → list → do → stats → archive; asserts done.txt populated
- **Scenario 2** — Filter tests: `+project`, `@context`, and AND-combined filters
- **Scenario 3** — JSON schema_version: list, stats, add, show, do all assert `schema_version == 1` (D-02)
- **Scenario 4** — Exit codes: exit 0 (success), exit 1 (do/show with ID 9999), exit 2 (bad subcommand)
- **Scenario 5** — Enrichment pipeline: add → due → postpone (7 days) → do → archive; verifies `due:2026-12-08` after postpone

## Verification
- `cargo test -p todotxt-cli --test integration_tests` → 5 passed in 3.33s
- `cargo build --workspace` clean after test compilation

## Files changed
- `crates/todotxt-cli/tests/integration_tests.rs` — created (new file)

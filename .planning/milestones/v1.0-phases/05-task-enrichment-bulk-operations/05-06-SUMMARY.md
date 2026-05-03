---
phase: "05"
plan: "06"
subsystem: "todotxt-cli/tests"
tags: ["integration-tests", "pri", "depri", "due", "postpone", "archive", "del-done"]
dependency_graph:
  requires: ["05-01", "05-02", "05-03", "05-04", "05-05"]
  provides: ["integration test coverage for all Phase 5 commands"]
  affects: ["test suite"]
tech_stack:
  added: []
  patterns: ["assert_cmd", "assert_fs", "TestFixture pattern"]
key_files:
  created:
    - crates/todotxt-cli/tests/enrich_bulk_tests.rs
  modified: []
decisions:
  - "Used ISO date literals (e.g., 2026-04-15) for postpone tests to avoid test flakiness from date-sensitive resolution"
  - "JSON assertions check val[data][count] and val[schema_version]; status field does not exist in json_success envelope (uses data key)"
  - "fixture_with_done_file() helper creates config.toml with explicit done_file path for archive tests"
metrics:
  duration: "~15 minutes"
  completed: "2026-04-15"
  tasks: 1
  files_created: 1
---

# Phase 05 Plan 06: Integration Tests for Enrichment/Bulk Commands Summary

**One-liner:** 33-test integration suite covering pri/depri/due/postpone/archive/del-done commands with exit code, JSON, idempotency, and atomicity verification.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create enrich_bulk_tests.rs with 33 integration tests | 1aa7c81 | crates/todotxt-cli/tests/enrich_bulk_tests.rs |

## Test Coverage Summary

**Priority tests (7):** pri sets/replaces priority, multi-ID, invalid letter exits 2, depri removes priority, depri multi-ID, depri idempotent

**Due/Postpone tests (12):** ISO date, today, tomorrow, weekday, invalid format exits 2, nonexistent ID exits 1, JSON output, postpone adds days, no-due-date exits 2, cross-month boundary, JSON output, cross-year boundary

**Archive/Del-done tests (8):** moves completed tasks, creates done.txt, empty list exits 0, idempotent, atomicity (no task in both files), JSON output with count, del-done removes completed, del-done empty exits 0, del-done idempotent, del-done JSON output

**Exit code tests (4):** invalid ID exits 1, validation error exits 2, success exits 0, due success exits 0

**Total: 33 tests** (exceeds 29+ requirement)

## Test Infrastructure Notes

- `fixture_with_done_file()` helper extends `TestFixture` with explicit `done_file` TOML config entry for archive tests
- JSON envelope structure: `{"schema_version":1,"data":{...}}` — assertions use `val["data"]["count"]` for bulk commands
- `CliError::NotFound` → exit 1; `CliError::Other` → exit 2

## Deviations from Plan

**1. [Rule 1 - Bug] JSON schema uses "data" not "status"**
- **Found during:** Initial test run (4 failures)
- **Issue:** Plan described checking `val["status"] == "ok"` but actual JSON envelope is `{"schema_version":1,"data":{...}}` with no status field
- **Fix:** Updated assertions to `val["data"]` and `val["data"]["count"]`
- **Files modified:** enrich_bulk_tests.rs (no implementation change)

**2. [Observation] cross-year postpone test had intermittent Windows file-locking failure**
- Reproduced once, then passed consistently. Windows TempDir race condition under parallel test execution. Test retained as-is (no logic issue).

## Self-Check

- [x] enrich_bulk_tests.rs exists and has 33 tests
- [x] cargo test -p todotxt-cli: 94 total tests, 0 failed
- [x] cargo clippy -p todotxt-cli -- -D warnings: clean
- [x] Commit 1aa7c81 exists

## Self-Check: PASSED

---
phase: 05
plan: 02
status: COMPLETE
commit: c0d9008
---

# Phase 05 Plan 02: Date Parsing Utility — Summary

**One-liner:** Shared `parse_date_input(s, today)` utility in `date.rs` handling today/tomorrow/weekdays/ISO formats per D-03 strict whitelist.

## Tasks Completed

1. **Implement parse_date_input utility** — Created `crates/todotxt-cli/src/date.rs` with `parse_date_input(s: &str, today: NaiveDate) -> Result<NaiveDate, String>`, `next_weekday` helper, and 5 unit tests.
2. **Export from main.rs** — Added `pub mod date;` to `main.rs` so `due.rs` and `postpone.rs` can import via `crate::date::parse_date_input`.

## Verification

- `cargo build -p todotxt-cli` — clean
- `cargo clippy -p todotxt-cli -- -D warnings` — clean
- `cargo test -p todotxt-cli date::tests` — 5/5 passed (today, tomorrow, weekday_next_occurrence, iso_date, invalid_format)

## Deviations from Plan

None — plan executed exactly as written.

## Self-Check: PASSED

- `crates/todotxt-cli/src/date.rs` — FOUND
- Commit `c0d9008` — FOUND

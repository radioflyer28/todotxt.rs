---
phase: 40
slug: group-by-decoupling-test-coverage
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2025-07-17
---

# Phase 40 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust / cargo test |
| **Config file** | `crates/todotxt-tui/Cargo.toml` |
| **Quick run command** | `cargo test -p todotxt-tui 2>&1 \| tail -3` |
| **Full suite command** | `cd crates/todotxt-tui && cargo test` |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p todotxt-tui 2>&1 | tail -3`
- **After every plan wave:** Run `cd crates/todotxt-tui && cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~1 second

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 40-01-01 | 01 | 1 | GRP-01 | — | GroupByCategory enum with all 4 variants | unit | `cargo test -p todotxt-tui group_by_category_default_is_priority` | ✅ | ✅ green |
| 40-01-02 | 01 | 1 | GRP-01 | — | group_key_for returns correct key per variant | unit | `cargo test -p todotxt-tui group_key_for_groups_by_correct_field_per_variant` | ✅ | ✅ green |
| 40-01-03 | 01 | 1 | GRP-01 | — | Pane initializes group_by to Priority | unit | `cargo test -p todotxt-tui pane_initializes_group_by_to_priority` | ✅ | ✅ green |
| 40-02-01 | 02 | 2 | GRP-02 | — | cycle_group_by wraps through all 4 variants | unit | `cargo test -p todotxt-tui cycle_group_by_wraps_through_all_four_variants` | ✅ | ✅ green |
| 40-02-02 | 02 | 2 | GRP-02 | — | 'g' key cycles group_by independently of sort | unit | `cargo test -p todotxt-tui g_key_cycles_group_by_independently_of_sort_order` | ✅ | ✅ green |
| 40-03-01 | 02 | 2 | GRP-03 | — | status bar grp: indicator matches active group_by | unit | `cargo test -p todotxt-tui status_bar_grp_indicator_text_matches_active_group_by` | ✅ | ✅ green |
| 40-04-01 | 01 | 1 | GRP-04 | — | PaneConfig without group_by deserializes to None | unit | `cargo test -p todotxt-tui pane_config_without_group_by_deserializes_to_none` | ✅ | ✅ green |
| 40-03-02 | 03 | 3 | TST-01 | — | 11 Phase 22 gap tests (22-01-G01 through 22-03-G05) | unit | `cargo test -p todotxt-tui phase_22` | ✅ | ✅ green |
| 40-03-03 | 03 | 3 | TST-02 | — | make_app_with_config helper for config-driven tests | unit | `cargo test -p todotxt-tui make_app_with_config` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. The `crates/todotxt-tui` crate had a working `cargo test` setup with 161 tests before Phase 40 began. No new test scaffolding or fixtures were needed beyond helper function `make_app_with_config` added in Plan 40-03.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [x] All tasks have automated verify commands
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references (N/A — infrastructure pre-existed)
- [x] No watch-mode flags
- [x] Feedback latency < 1s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2025-07-17

---

## Notes

- `GroupByCategory::DueDate` serializes as `"due_date"` (snake_case via `#[serde(rename_all = "snake_case")]`), **not** `"duedate"`.
- Single-pane `rebuild_and_reanchor()` syncs `pane.grouping → app.grouping` but does **not** sync `pane.group_by → app.group_by`. Tests that drive grouping via `rebuild_and_reanchor()` must set `app.active_pane_mut().grouping = true` (not `app.grouping = true`) and set `app.group_by` directly on the app struct.
- Final test count: 168 passed, 0 failed (was 161 before Phase 40 validation gap fill).

---
phase: 43
slug: view-state-persistence
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-07
audited: 2026-05-07
---

# Phase 43 — Validation Strategy

> Per-phase validation contract for PRSV-01, PRSV-02, PRSV-03.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) |
| **Config file** | `Cargo.toml` (workspace) |
| **Quick run command** | `cargo test -p todotxt-tui state_file_tests` |
| **Full suite command** | `cargo test -p todotxt-tui` |
| **Estimated runtime** | ~3 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p todotxt-tui state_file_tests`
- **After every plan wave:** Run `cargo test -p todotxt-tui`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File | Status |
|---------|------|------|-------------|-----------|-------------------|------|--------|
| 43-01-T01 | 01 | 1 | PRSV-02 | unit | `cargo test -p todotxt-tui tuistatefile_load_missing_returns_none` | `config.rs` | ✅ green |
| 43-01-T02 | 01 | 1 | PRSV-02 | unit | `cargo test -p todotxt-tui tuistatefile_load_malformed_returns_none` | `config.rs` | ✅ green |
| 43-01-T03 | 01 | 1 | PRSV-01/02 | unit | `cargo test -p todotxt-tui tuistatefile_load_valid_parses_panes` | `config.rs` | ✅ green |
| 43-01-T04 | 01 | 1 | PRSV-01 | unit | `cargo test -p todotxt-tui tuistatefile_save_load_roundtrip` | `config.rs` | ✅ green |
| 43-01-T05 | 01 | 1 | PRSV-02 | unit | `cargo test -p todotxt-tui tuistatefile_load_unknown_fields_ignored` | `config.rs` | ✅ green |
| 43-01-T06 | 01 | 1 | D-04 | unit | `cargo test -p todotxt-tui state_file_path_sibling_of_config` | `config.rs` | ✅ green |
| 43-02-T01 | 02 | 2 | PRSV-01 | integration | `cargo test -p todotxt-tui test_quit_persists_runtime_panes_into_config` | `pane_integration_test.rs` | ✅ green |
| 43-02-T02 | 02 | 2 | PRSV-01 | integration | `cargo test -p todotxt-tui test_persisted_pane_data_contains_only_config_fields` | `pane_integration_test.rs` | ✅ green |
| 43-02-T03 | 02 | 2 | PRSV-01 | integration | `cargo test -p todotxt-tui test_no_pane_write_occurs_until_quit_persist_path` | `pane_integration_test.rs` | ✅ green |
| 43-02-T04 | 02 | 2 | PRSV-02 | integration | `cargo test -p todotxt-tui test_startup_state_file_overrides_config_panes` | `pane_integration_test.rs` | ✅ green |
| 43-02-T05 | 02 | 2 | PRSV-02 | integration | `cargo test -p todotxt-tui test_startup_absent_state_file_uses_config_panes` | `pane_integration_test.rs` | ✅ green |
| 43-02-T06 | 02 | 2 | PRSV-03/D-06 | integration | `cargo test -p todotxt-tui test_save_view_state_no_write_when_unchanged` | `pane_integration_test.rs` | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No additional framework setup needed — Rust `cargo test` + `tempfile` crate (already a dev-dep) is sufficient.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `tui-state.toml` written to correct portable-mode dir when `config.toml` is beside binary | D-04 | Requires running the compiled binary with a portable-mode directory layout | Launch `todotxt-tui` from a directory containing `config.toml`, quit cleanly, verify `tui-state.toml` appears in the same directory |
| No error displayed to user when `tui-state.toml` is unreadable (permissions) | PRSV-02 | Requires OS-level permission manipulation | Create `tui-state.toml` with no read permissions, launch TUI, verify no error in status bar and config.toml panes are used |

---

## Validation Audit 2026-05-07

| Metric | Count |
|--------|-------|
| Gaps found | 3 |
| Resolved (automated) | 3 |
| Escalated (manual-only) | 0 |
| Bug fix triggered | 1 (startup_pane_snapshot normalization — `group_by: None` vs `Some(Priority)` mismatch) |

---

## Validation Sign-Off

- [x] All tasks have automated verify
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0: no stubs needed — existing infrastructure sufficient
- [x] No watch-mode flags
- [x] Feedback latency < 5s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-05-07

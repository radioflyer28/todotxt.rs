---
phase: 27
slug: config-defined-panes-validation-ship-readiness
status: complete
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-29
---

# Phase 27 — Validation Strategy

> Per-phase validation contract for Phase 27: config-defined-panes-validation-ship-readiness.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust / cargo test |
| **Config file** | `crates/todotxt-tui/Cargo.toml` |
| **Quick run command** | `cargo test -p todotxt-tui config_panes_test path_resolution_test` |
| **Full suite command** | `cargo test -p todotxt-tui` |
| **Estimated runtime** | ~2 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p todotxt-tui`
- **After every plan wave:** Run `cargo test -p todotxt-tui`
- **Before `/gsd-verify-work`:** Full suite must be green (13/13 automated tests pass)
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 27-01-T1 | 01 | 1 | CFG-01 | — | `parse_pane_config` loads valid pane entries with correct label/filter/sort | integration | `cargo test -p todotxt-tui test_config_panes_valid_entries` | ✅ | ✅ green |
| 27-01-T2 | 01 | 1 | CFG-01 | — | Invalid `sort` value is skipped — pane still loads with default sort | integration | `cargo test -p todotxt-tui test_config_panes_invalid_sort_skipped` | ✅ | ✅ green |
| 27-01-T3 | 01 | 1 | CFG-01 | — | Missing/empty `[panes]` section produces single default pane | integration | `cargo test -p todotxt-tui test_config_panes_missing_or_empty` | ✅ | ✅ green |
| 27-02-T1 | 02 | 2 | CFG-02 | — | `startup_bootstrap` loads config panes into `App.panes` on init | integration | `cargo test -p todotxt-tui test_startup_bootstrap_from_config` | ✅ | ✅ green |
| 27-02-T2 | 02 | 2 | CFG-01 | — | Config pane with invalid `sort` is skipped; remaining panes load | integration | `cargo test -p todotxt-tui test_startup_bootstrap_invalid_sort_skip` | ✅ | ✅ green |
| 27-02-T3 | 02 | 2 | CFG-01 | — | Config with only invalid pane entries loads a single safe default pane | integration | `cargo test -p todotxt-tui test_startup_bootstrap_invalid_only_produces_safe_default` | ✅ | ✅ green |
| 27-03-T1 | 03 | 3 | CFG-03 | — | On quit, pane state (filter/sort/grouping) is persisted to config file | integration | `cargo test -p todotxt-tui test_quit_persists_pane_state` | ✅ | ✅ green |
| 27-03-T2 | 03 | 3 | CFG-03 | — | Persisted pane fields include label, filter, sort, grouping (round-trip) | integration | `cargo test -p todotxt-tui test_quit_persists_pane_fields` | ✅ | ✅ green |
| 27-03-T3 | 03 | 3 | CFG-03 | — | Config file is NOT written until quit (no intermediate writes) | integration | `cargo test -p todotxt-tui test_no_write_until_quit` | ✅ | ✅ green |
| 27-03-T4 | 03 | 1 | PATH-01 | — | `resolve_paths` with no flags uses sibling convention from todo.txt | integration | `cargo test -p todotxt-tui test_path_resolution_no_flags` | ✅ | ✅ green |
| 27-03-T5 | 03 | 1 | PATH-02 | — | `--todo` CLI flag overrides sibling convention for todo path | integration | `cargo test -p todotxt-tui test_path_resolution_cli_todo_override` | ✅ | ✅ green |
| 27-03-T6 | 03 | 1 | PATH-01 | — | Sibling done.txt is resolved as `<todo-dir>/done.txt` by default | integration | `cargo test -p todotxt-tui test_path_resolution_sibling_done` | ✅ | ✅ green |
| 27-03-T7 | 03 | 1 | PATH-03 | — | `--archive` CLI flag overrides archive path | integration | `cargo test -p todotxt-tui test_path_resolution_cli_archive_override` | ✅ | ✅ green |
| 27-03-T8 | 03 | 1 | PATH-02 | — | `--todo` and `--archive` both set resolves both overrides independently | integration | `cargo test -p todotxt-tui test_path_resolution_both_overrides` | ✅ | ✅ green |
| 27-G01 | 03 | — | PATH-01 | — | CLI argument parsing accepts `--todo` and `--archive` flags (end-to-end) | manual | — | ❌ | ⚠️ manual-only |
| 27-G02 | 03 | — | CFG-01 | — | README documents `[panes]` TOML format with label, filter, sort, grouping fields | manual | — | ❌ | ⚠️ manual-only |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ manual-only*

---

## Wave 0 Requirements

Existing test files (`config_panes_test.rs`, `path_resolution_test.rs`, `pane_integration_test.rs`) provide comprehensive automated coverage. No new infrastructure needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CLI `--todo` and `--archive` flags accepted and parsed | PATH-01/PATH-02/PATH-03 | Argument parsing via `clap` requires a spawned binary process; unit tests cover `resolve_paths` logic but not CLI invocation | Run `todotxt-tui --todo ~/todo.txt --archive ~/done.txt`; verify app starts with the specified files |
| README documents `[panes]` TOML format | CFG-01 | Documentation review; cannot be automated | Open README.md; verify `[panes]` section with `[[panes]]` entry syntax, `label`, `filter`, `sort`, `grouping` fields, and TOML formatting example |

---

## Validation Sign-Off

- [x] All tasks have automated verify or are marked manual-only
- [x] Sampling continuity: 13/15 tasks automated (87% automation density) — well above threshold
- [x] Wave 0 not needed — `config_panes_test.rs` (3 tests) + `path_resolution_test.rs` (5 tests) + pane_integration_test.rs subset (6 tests) cover all requirements
- [x] No watch-mode flags

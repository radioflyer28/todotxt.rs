---
phase: 25
slug: per-pane-query-behavior
status: complete
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-29
---

# Phase 25 — Validation Strategy

> Per-phase validation contract for Phase 25: per-pane-query-behavior.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust / cargo test |
| **Config file** | `crates/todotxt-tui/Cargo.toml` |
| **Quick run command** | `cargo test -p todotxt-tui pane_integration_test` |
| **Full suite command** | `cargo test -p todotxt-tui` |
| **Estimated runtime** | ~2 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p todotxt-tui pane_integration_test`
- **After every plan wave:** Run `cargo test -p todotxt-tui pane_integration_test`
- **Before `/gsd-verify-work`:** Full suite must be green (18/18 integration tests pass)
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 25-01-T1 | 01 | 1 | PANE-03 | — | Per-pane filter state is preserved when switching pane focus | integration | `cargo test -p todotxt-tui test_pane_filter_state_preserved_on_navigation` | ✅ | ✅ green |
| 25-01-T2 | 01 | 1 | PANE-03 | — | Navigation wraps around and filter routes to active pane | integration | `cargo test -p todotxt-tui test_pane_navigation_wraps_around` | ✅ | ✅ green |
| 25-01-G01 | 01 | 1 | PANE-03 | — | `filter_toggle` (f key) targets `active_pane_mut().filter_query` | manual | — | ❌ | ⚠️ manual-only |
| 25-01-G02 | 01 | 1 | PANE-03 | — | `clear_filter` ('0' key) clears only active pane's filter | manual | — | ❌ | ⚠️ manual-only |
| 25-01-G03 | 01 | 1 | PANE-03 | — | Preset key (1-9) applies filter to active pane only | manual | — | ❌ | ⚠️ manual-only |
| 25-01-G04 | 01 | 1 | PANE-03 | — | `filter_open` (F key) pre-fills editor from active pane's `filter_query` | manual | — | ❌ | ⚠️ manual-only |
| 25-02-T1 | 02 | 2 | PANE-04 | — | Per-pane sort order is preserved when switching pane focus | integration | `cargo test -p todotxt-tui test_pane_sort_state_preserved_on_navigation` | ✅ | ✅ green |
| 25-02-T2 | 02 | 2 | PANE-04 | — | Per-pane grouping state is preserved when switching pane focus | integration | `cargo test -p todotxt-tui test_pane_grouping_state_preserved_on_navigation` | ✅ | ✅ green |
| 25-02-G01 | 02 | 2 | PANE-04 | — | `sort_cycle` hotkey routes to `active_pane_mut().sort_order` only | manual | — | ❌ | ⚠️ manual-only |
| 25-02-G02 | 02 | 2 | PANE-04 | — | `group_toggle` hotkey routes to `active_pane_mut().grouping` only | manual | — | ❌ | ⚠️ manual-only |
| 25-02-G03 | 02 | 2 | VIEW-02 | — | Status bar shows active pane's filter/sort/group state in multi-pane mode | manual | — | ❌ | ⚠️ manual-only |
| 25-02-G04 | 02 | 2 | VIEW-02 | — | Status bar shows global state (not per-pane) when panes are hidden | manual | — | ❌ | ⚠️ manual-only |
| 25-03-T1 | 03 | 3 | PANE-03 | — | `reconcile_active_pane()` clamps to valid bounds | integration | `cargo test -p todotxt-tui test_reconcile_active_pane_ensures_bounds` | ✅ | ✅ green |
| 25-03-T2 | 03 | 3 | PANE-03 | — | `reconcile_active_pane()` creates default pane when list is empty | integration | `cargo test -p todotxt-tui test_reconcile_active_pane_creates_default_pane_when_empty` | ✅ | ✅ green |
| 25-03-T3 | 03 | 3 | PANE-03 | — | `active_pane_mut()` auto-reconciles invalid `active_pane` index | integration | `cargo test -p todotxt-tui test_active_pane_mut_reconciles_bounds` | ✅ | ✅ green |
| 25-SUP-T1 | SUP | — | PANE-03 | — | `handle_filter_defining_key` Enter arm writes to `active_pane_mut().filter_query` (Phase 28 FAIL-1 fix, commit e5d25eb) | manual | — | ❌ | ⚠️ manual-only |
| 25-SUP-T2 | SUP | — | PANE-03 | — | Live preview in FilterDefining panel updates active pane only (Phase 28 fix) | manual | — | ❌ | ⚠️ manual-only |
| 25-SUP-T3 | SUP | — | VIEW-02 | — | `render_status_bar` `panes_hidden` guard prevents stale multi-pane display when hidden (Phase 28 fix) | manual | — | ❌ | ⚠️ manual-only |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ manual-only*

---

## Wave 0 Requirements

Existing infrastructure (`pane_integration_test.rs`, 18 tests) covers all automated verifications. No new test files or framework installation needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `filter_toggle` (f) targets active pane `filter_query` | PANE-03 | Dispatched through `handle_normal_key` which requires a full `KeyEvent` loop; no key simulation helper | Run TUI with 2 panes; set filter on pane 0 (press f, enter filter text); navigate to pane 1; press f for different filter; navigate back; verify each pane retains its own filter |
| `clear_filter` ('0') clears only active pane | PANE-03 | Same as above | Set filter on pane 0; navigate back; press 0; verify only pane 0 filter is cleared; pane 1 filter intact |
| Preset key (1-9) applies to active pane only | PANE-03 | Same as above | Define presets in config; run TUI; focus pane 0; press 1; verify only active pane shows preset filter |
| `filter_open` (F) pre-fills from active pane filter | PANE-03 | Dialog pre-fill happens in event loop via editor construction | Run TUI with existing pane filter; press F; verify the input field shows current pane's filter text |
| `sort_cycle` routes to active pane only | PANE-04 | Same hotkey dispatch gap | Set different sorts on each pane; navigate between panes; verify each pane retains its sort independently |
| `group_toggle` routes to active pane only | PANE-04 | Same as above | Toggle grouping on pane 0; navigate to pane 1; verify grouping state is independent |
| Status bar shows active pane filter/sort/group | VIEW-02 | `render_status_bar` requires `Frame` context | Run TUI; focus pane 0 with filter "project:work"; verify status bar shows "project:work" and pane 0's sort |
| Status bar hides per-pane section when panes hidden | VIEW-02 | Same as above | Press Ctrl+P to hide panes; verify status bar switches to global state display |
| FilterDefining Enter writes to active pane (Phase 28 fix) | PANE-03 | Full F-key dialog + Enter flow through event loop | Press F; type new filter; press Enter; verify active pane filter updated (not global `self.filter_query`) |
| Live preview updates active pane only (Phase 28 fix) | PANE-03 | Same as above | Press F; type characters one by one; verify task list updates live showing only active pane results |

---

## Validation Sign-Off

- [x] All tasks have automated verify or are marked manual-only
- [ ] Sampling continuity: 8 manual-only items (key dispatch + status bar render) reduce automated density — acceptable; core state preservation is fully automated via 5 integration tests
- [x] Wave 0 not needed — `pane_integration_test.rs` (18 tests) covers all automated verifications
- [x] No watch-mode flags

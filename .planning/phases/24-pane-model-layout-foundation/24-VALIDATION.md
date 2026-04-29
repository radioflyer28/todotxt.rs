---
phase: 24
slug: pane-model-layout-foundation
status: complete
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-29
---

# Phase 24 — Validation Strategy

> Per-phase validation contract for Phase 24: pane-model-layout-foundation.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust / cargo test |
| **Config file** | `crates/todotxt-tui/Cargo.toml` |
| **Quick run command** | `cargo test -p todotxt-tui` |
| **Full suite command** | `cargo test -p todotxt-tui` |
| **Estimated runtime** | ~2 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p todotxt-tui`
- **After every plan wave:** Run `cargo test -p todotxt-tui`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 24-01-T1 | 01 | 1 | PANE-01 | — | `Pane` struct with `id`, `label`, `display_rows`, `selected`, `filter_query`, `sort_order` fields | unit | `cargo test -p todotxt-tui test_app_initializes_with_one_pane` | ✅ | ✅ green |
| 24-01-T2 | 01 | 1 | PANE-01 | — | `focus_next_pane()` wraps around at boundary | unit | `cargo test -p todotxt-tui test_focus_navigation_multiple_panes` | ✅ | ✅ green |
| 24-01-T3 | 01 | 1 | PANE-01 | — | `focus_prev_pane()` wraps around at boundary | unit | `cargo test -p todotxt-tui test_focus_navigation_multiple_panes` | ✅ | ✅ green |
| 24-01-T4 | 01 | 1 | PANE-01 | — | `focus_next_pane()` is no-op with single pane | unit | `cargo test -p todotxt-tui test_focus_next_pane_single_pane_noop` | ✅ | ✅ green |
| 24-01-T5 | 01 | 1 | PANE-02 | — | Per-pane `selected` index is independent across panes | unit | `cargo test -p todotxt-tui test_pane_selection_independence` | ✅ | ✅ green |
| 24-01-G01 | 01 | 1 | PANE-01 | — | Left/Right arrow keys dispatch to `focus_prev_pane()` / `focus_next_pane()` | manual | — | ❌ | ⚠️ manual-only |
| 24-02-G01 | 02 | 1 | VIEW-01 | — | `PaneList` widget renders N panes side-by-side using `Layout::horizontal()` | manual | — | ❌ | ⚠️ manual-only |
| 24-02-G02 | 02 | 1 | VIEW-01 | — | Active pane has Cyan bold border + ▶ indicator in title | manual | — | ❌ | ⚠️ manual-only |
| 24-02-G03 | 02 | 1 | VIEW-01 | — | Inactive panes rendered with DarkGray border | manual | — | ❌ | ⚠️ manual-only |
| 24-02-T1 | 02 | 1 | PANE-01 | — | `render_panes()` routes to fallback when `should_show_single_pane()` is true | unit | `cargo test -p todotxt-tui test_single_pane_mode_with_empty_panes` | ✅ | ✅ green |
| 24-03-T1 | 03 | 1 | PANE-01 | — | `reconcile_active_pane()` clamps `active_pane` to `[0, panes.len()-1]` | unit | `cargo test -p todotxt-tui test_reconcile_out_of_bounds_active_pane` | ✅ | ✅ green |
| 24-03-T2 | 03 | 1 | PANE-01 | — | `reconcile_active_pane()` creates default pane when `panes` is empty | unit | `cargo test -p todotxt-tui test_reconcile_empty_panes` | ✅ | ✅ green |
| 24-03-T3 | 03 | 1 | PANE-02 | — | `display_rows` fallback returns active pane rows correctly | unit | `cargo test -p todotxt-tui test_display_rows_fallback` | ✅ | ✅ green |
| 24-03-T4 | 03 | 1 | PANE-02 | — | `display_rows` multi-pane returns correct per-pane rows | unit | `cargo test -p todotxt-tui test_display_rows_multi_pane` | ✅ | ✅ green |
| 24-03-T5 | 03 | 1 | PANE-01 | — | `should_show_single_pane()` returns true with exactly one pane | unit | `cargo test -p todotxt-tui test_single_pane_mode_with_one_pane` | ✅ | ✅ green |
| 24-03-T6 | 03 | 1 | PANE-01 | — | `should_show_single_pane()` returns true when all panes are empty | unit | `cargo test -p todotxt-tui test_single_pane_mode_with_all_empty` | ✅ | ✅ green |
| 24-03-T7 | 03 | 1 | VIEW-01 | — | Multi-pane mode active with populated panes | unit | `cargo test -p todotxt-tui test_multi_pane_mode_with_populated_panes` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ manual-only*

---

## Wave 0 Requirements

Existing infrastructure (cargo test) covers all automated verifications. No new test files or framework installation needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Left/Right arrow keys dispatch to `focus_prev/next_pane()` | PANE-01 | Key dispatch requires full ratatui event loop + `KeyEvent`; no headless key simulation helper | Run TUI; with 2+ panes, press Left/Right arrow keys; verify active pane focus changes |
| `PaneList` renders N panes side-by-side using horizontal layout | VIEW-01 | `ratatui` rendering requires a `Frame` context; no headless render helper exists | Run TUI with 2+ panes configured; verify multiple panes appear side-by-side with visible borders |
| Active pane has Cyan bold border + ▶ title indicator | VIEW-01 | Same as above — visual property requires Frame render | Run TUI with 2+ panes; navigate panes; verify active pane shows Cyan border and ▶ symbol in title |
| Inactive panes show DarkGray border | VIEW-01 | Same as above | Run TUI with 2+ panes; verify non-active panes display with DarkGray border styling |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or are marked manual-only
- [ ] Sampling continuity: 4 manual-only items (visual rendering) reduce automated coverage density — acceptable given no headless render helper available in current test infrastructure
- [x] Wave 0 not needed — existing infrastructure (`fallback_test.rs`, `app::tests`) covers all automated tests
- [x] No watch-mode flags

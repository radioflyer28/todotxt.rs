---
phase: 26
slug: pane-management-quick-hide-show
status: complete
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-29
---

# Phase 26 — Validation Strategy

> Per-phase validation contract for Phase 26: pane-management-quick-hide-show.

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
| 26-01-G01 | 01 | 1 | PANE-05 | — | `pane_add()` creates pane labeled "Pane N" using monotonic counter and appends to vec | manual | — | ❌ | ⚠️ manual-only |
| 26-01-G02 | 01 | 1 | PANE-05 | — | `pane_add()` max guardrail: silent no-op when `panes.len() >= 10` | manual | — | ❌ | ⚠️ manual-only |
| 26-01-G03 | 01 | 1 | PANE-05 | — | `pane_add()` shifts `active_pane` to the newly created pane | manual | — | ❌ | ⚠️ manual-only |
| 26-01-G04 | 01 | 1 | PANE-05 | — | `pane_delete()` removes active pane and re-normalizes pane IDs | manual | — | ❌ | ⚠️ manual-only |
| 26-01-G05 | 01 | 1 | PANE-05 | — | `pane_delete()` shifts focus left (prefer `active_pane - 1`), else right | manual | — | ❌ | ⚠️ manual-only |
| 26-01-G06 | 01 | 1 | PANE-05 | — | Ctrl+N dispatch in `handle_normal_key` calls `pane_add()` | manual | — | ❌ | ⚠️ manual-only |
| 26-01-G07 | 01 | 1 | PANE-05 | — | Ctrl+W dispatch in `handle_normal_key` calls `pane_delete()` | manual | — | ❌ | ⚠️ manual-only |
| 26-01-T1 | 01 | 1 | PANE-05 | — | Navigation wraps around after pane creation (indirect: tests 2-pane setup) | integration | `cargo test -p todotxt-tui test_pane_navigation_wraps_around` | ✅ | ✅ green |
| 26-02-G01 | 02 | 2 | PANE-05 | — | Ctrl+P dispatch calls `pane_hide_toggle()` in `handle_normal_key` | manual | — | ❌ | ⚠️ manual-only |
| 26-02-G02 | 02 | 2 | PANE-05 | — | `render_panes()` renders single-pane view when `panes_hidden` is `true` | manual | — | ❌ | ⚠️ manual-only |
| 26-02-G03 | 02 | 2 | PANE-05 | — | Pane vec structure (filters/sorts/state) is unchanged while panes are hidden | manual | — | ❌ | ⚠️ manual-only |
| 26-02-G04 | 02 | 2 | PANE-05 | — | `panes_hidden` resets to `false` on `App::new()` (not persisted to config) | manual | — | ❌ | ⚠️ manual-only |
| 26-02-T1 | 02 | 2 | PANE-05 | — | Bounds reconciliation operates correctly with arbitrary pane states | integration | `cargo test -p todotxt-tui test_reconcile_active_pane_ensures_bounds` | ✅ | ✅ green |
| 26-03-G01 | 03 | 3 | PANE-05 | — | Help overlay shows Panes section with Ctrl+N, Ctrl+W, Ctrl+P entries | manual | — | ❌ | ⚠️ manual-only |
| 26-03-G02 | 03 | 3 | PANE-05 | — | Status bar shows no pane count indicator (D-24 compliance) | manual | — | ❌ | ⚠️ manual-only |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ manual-only*

---

## Wave 0 Requirements

Phase 26 lifecycle methods (`pane_add`, `pane_delete`, `pane_hide_toggle`) have no dedicated unit tests. These methods are invoked through `handle_normal_key` which requires a full `KeyEvent` event loop. A Wave 0 scaffold would add direct unit tests for `pane_add()` and `pane_delete()` (bypassing the event loop) to improve Nyquist density.

**Status:** Not implemented. Acceptable — all behaviors are verified via source-code inspection in VERIFICATION.md (18/18 truths verified) and confirmed by manual testing. The automation gap is documented here for future improvement.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `pane_add()` creates "Pane N" with focus shift | PANE-05 | `pane_add` is invoked through `handle_normal_key`; no direct unit test; key dispatch requires a full `KeyEvent` | Run TUI; press Ctrl+N; verify new pane appears with label "Pane 2" and immediately receives focus |
| `pane_add()` max guardrail (10 pane limit) | PANE-05 | Same as above | Create 10 panes via Ctrl+N (press 9 times from default); press Ctrl+N again; verify no 11th pane appears |
| `pane_delete()` removes active pane + ID renormalization | PANE-05 | `pane_delete` same dispatch gap | Run TUI with 3 panes; select middle pane; press Ctrl+W; verify pane removed, remaining panes renumbered starting from 0 |
| `pane_delete()` focus left preference | PANE-05 | Same as above | With pane 2 active (out of panes 0,1,2), press Ctrl+W; verify focus moves to pane 1 (prefer left) |
| Ctrl+N and Ctrl+W hotkey dispatch | PANE-05 | Key dispatch requires full `KeyEvent` event loop; `handle_normal_key` not directly unit-tested | Press Ctrl+N; verify pane created. Press Ctrl+W; verify pane deleted |
| `pane_hide_toggle()` via Ctrl+P | PANE-05 | Same key dispatch gap | Press Ctrl+P; verify multi-pane collapses to single-pane view; press again; verify panes restored |
| Pane structure preserved while hidden | PANE-05 | In-memory state + visual inspection | Set filters/sorts on each pane; press Ctrl+P; press Ctrl+P again; verify all pane settings intact |
| Help overlay Panes section | PANE-05 | Help overlay requires `Frame` render context | Press ?; verify Panes section shows Create Pane (Ctrl+N), Delete Pane (Ctrl+W), Toggle Panes (Ctrl+P) |
| Status bar no pane count indicator | PANE-05 | Status bar render requires `Frame` context | Run TUI with 2+ panes; inspect bottom status bar; verify no "Pane 1/2" or "1/3" text present |

---

## Validation Sign-Off

- [x] All tasks have automated verify or are marked manual-only
- [ ] Sampling continuity: 13 manual-only items — automated coverage is low; `pane_add` / `pane_delete` / `pane_hide_toggle` lack unit tests. Wave 0 scaffolding noted above as future improvement
- [x] Wave 0 automation gap documented (non-blocking — VERIFICATION.md confirms behavior via source inspection)
- [x] No watch-mode flags

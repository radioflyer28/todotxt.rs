---
phase: 20
slug: bulk-actions-selection-ux
status: complete
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-28
note: "Retroactive — written after phase completion (phase predates Nyquist workflow adoption)"
---

# Phase 20 — Validation Strategy (Retroactive)

> Retroactive per-phase validation contract. Phase 20 was completed before the Nyquist validation
> workflow was adopted. This file documents the test coverage that exists and confirms Nyquist compliance.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` |
| **Config file** | `Cargo.toml` (already configured) |
| **Quick run command** | `cargo test -p todotxt-tui bulk` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **Retroactive assessment:** All tests present and green at phase completion
- **Full suite command:** `cargo test --workspace` — 0 failures verified 2026-04-28

---

## Wave 0 Status

**NOT COMPLETED** — Phase 20 predates wave-0 discipline. 9 bulk-action tests were written during task execution (inline with implementation), not in a dedicated Wave 0 scaffold pass. This is a process gap, not a coverage gap: all BULK-01, BULK-02, BULK-03 truths have automated test coverage.

---

## Observable Truths Coverage

| Truth | Requirement | Test Name(s) | Status |
|-------|-------------|-------------|--------|
| Bulk delete with D hotkey enters confirmation flow | BULK-01 | `bulk_delete_descending_order`, `bulk_delete_multiple_tasks_shows_count` | ✓ COVERED |
| Deletion in descending index order prevents corruption | BULK-01 | `bulk_delete_descending_order` | ✓ COVERED |
| Confirmation shows "Delete N tasks?" count for N>1 | BULK-01 | `bulk_delete_multiple_tasks_shows_count` | ✓ COVERED |
| Cancel clears selection state (Esc/non-y) | BULK-01 | `bulk_delete_cancel_clears_selection` | ✓ COVERED |
| Bulk append with T hotkey appends to all selected | BULK-02 | `bulk_append_commits_to_all_selected` | ✓ COVERED |
| Esc in bulk append cancels without mutation | BULK-02 | `bulk_append_esc_cancels` | ✓ COVERED |
| Empty Enter in bulk append cancels without mutation | BULK-02 | `bulk_append_empty_enter_cancels` | ✓ COVERED |
| Status bar shows `\| N selected` when tasks selected | BULK-03 | `status_bar_shows_selected_count` | ✓ COVERED |
| Status bar shows no count when selection empty | BULK-03 | `status_bar_no_count_when_empty` | ✓ COVERED |
| Status bar hint includes bulk action keys | BULK-03 | `status_bar_hint_includes_bulk_keys` | ✓ COVERED |
| PAR-01: Hotkeys aligned with todotxt.net | PAR-01 | Code inspection of `default_keymap` entries — D/T/v defaults match todotxt.net orientation | ✓ COVERED (code) |

**Note on PAR-01:** PAR-01 (hotkeys aligned with todotxt.net) is verified by code inspection of `default_keymap()` entries — not directly testable as a unit test without a specification comparison fixture. The D/T/v defaults are present and documented in DEVIATION.md (Phase 22).

**Total named tests:** 9 across 3 plans (42 total TUI tests at end of Phase 20)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Bulk delete confirmation overlay renders "Delete N tasks?" visually | BULK-01 | Overlay layout requires live TUI | Run TUI, select 3 tasks, press D — verify overlay shows count |
| Bulk append footer renders "Append: " + textarea widget | BULK-02 | Two-part Layout requires live TUI | Select tasks, press T — verify footer split with 9-char label |
| Status bar shows `\| 2 selected` in live TUI | BULK-03 | `render_status_bar` requires live TUI | Select 2 tasks — verify left segment shows count |

---

## Gaps Summary

**Wave-0 discipline gap only.** All BULK-01, BULK-02, BULK-03 truths have automated test coverage via 9 named tests. Tests were written during task execution rather than in a pre-implementation Wave 0 pass. No coverage gaps exist.

---

## Validation Sign-Off

- [x] All observable truths have automated test or code-inspection coverage
- [x] `cargo test -p todotxt-tui` — 58 passed, 0 failed (as of 2026-04-28)
- [x] `cargo test --workspace` — 0 failures
- [ ] Wave 0 test-first discipline — NOT COMPLETED (retroactive phase)

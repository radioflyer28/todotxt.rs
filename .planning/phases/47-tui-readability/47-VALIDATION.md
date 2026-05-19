---
phase: 47
slug: tui-readability
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-18
---

# Phase 47 - Validation Strategy

> Per-phase validation contract reconstructed from execution artifacts and verification results.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | `crates/todotxt-tui/Cargo.toml`, workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p todotxt-tui pane_list` |
| **Full suite command** | `cargo test -p todotxt-tui` |
| **Estimated runtime** | ~20 seconds |

## Sampling Rate

- **After every task commit:** Run the focused `todotxt-tui` filter for the touched behavior.
- **After every plan wave:** Run the relevant grouped/navigation coverage commands.
- **Before `$gsd-verify-work`:** The full `todotxt-tui` crate suite must be green.
- **Max feedback latency:** <120 seconds.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 47-01-01 | 01 | 1 | TUI-01 | N/A | Inactive panes preserve `pane.selected` while rendering no selected row; active pane remains the only visible cursor highlight | unit | `cargo test -p todotxt-tui inactive_pane_has_no_render_selected_row` | ✅ | ✅ green |
| 47-01-02 | 01 | 1 | D-01, D-02 | N/A | `PaneList::selected_row_for_render` gates `ListState` selection on active focus and label-selection state | unit | `cargo test -p todotxt-tui pane_list` | ✅ | ✅ green |
| 47-02-01 | 02 | 2 | D-03, D-04, D-07 | N/A | Grouped views insert exactly one blank spacer row before each non-first header, with no leading spacer and parity across single-pane and multi-pane modes | unit | `cargo test -p todotxt-tui group_spacer` | ✅ | ✅ green |
| 47-02-02 | 02 | 2 | TUI-02 | N/A | Grouped row builders emit `DisplayRow::GroupSpacer` before non-first headers only | unit | `cargo test -p todotxt-tui grouped_rows` | ✅ | ✅ green |
| 47-02-03 | 02 | 2 | D-03 | N/A | Spacer rows render as true blank structure and are ignored by non-task row helpers | unit | `cargo test -p todotxt-tui group_spacer` | ✅ | ✅ green |
| 47-02-04 | 02 | 2 | D-05, D-06 | N/A | Navigation and reanchor logic skip spacer and header rows so selection rests on task rows whenever tasks exist | unit | `cargo test -p todotxt-tui pane_move` | ✅ | ✅ green |
| 47-02-04 | 02 | 2 | TUI-02 | N/A | Full grouped readability behavior remains green across the crate test suite after structural-row changes | crate | `cargo test -p todotxt-tui` | ✅ | ✅ green |

## Wave 0 Requirements

- Existing Rust test infrastructure covers all Phase 47 requirements.
- `cargo test -p todotxt-tui tui_readability` matched zero tests during execution, so it is not relied on as substantive coverage.

## Manual-Only Verifications

All phase behaviors have automated verification.

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-05-18

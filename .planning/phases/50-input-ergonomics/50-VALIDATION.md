---
phase: 50
slug: input-ergonomics
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-19
---

# Phase 50 - Validation Strategy

> Per-phase validation contract reconstructed from execution artifacts and verification results.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | `crates/todotxt-core/Cargo.toml`, `crates/todotxt-tui/Cargo.toml`, workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p todotxt-tui date_picker` |
| **Full suite command** | `cargo test -p todotxt-core && cargo test -p todotxt-tui` |
| **Estimated runtime** | ~25 seconds |

## Sampling Rate

- **After every task commit:** Run focused date-picker or quick-setter tests in the touched crate.
- **After every plan wave:** Run the relevant crate-level suites for `todotxt-core` and `todotxt-tui`.
- **Before `$gsd-verify-work`:** Both `todotxt-core` and `todotxt-tui` suites must be green.
- **Max feedback latency:** <120 seconds.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 50-01-01 | 01 | 1 | DATE-UX-01 | N/A | TUI date picker can target due, threshold, and completed dates from the shared `s` workflow instead of only due dates | unit | `cargo test -p todotxt-tui date_picker_enter_applies` | ✅ | ✅ green |
| 50-01-02 | 01 | 1 | DATE-UX-01 | N/A | Explicit completed-date mutation writes the selected completion date instead of relying on "today" semantics | unit | `cargo test -p todotxt-core with_completion_date_sets_explicit_completion_date` | ✅ | ✅ green |
| 50-01-03 | 01 | 1 | DATE-UX-02 | N/A | Left/right navigation in the TUI picker jumps by 7 days while preserving normal date-entry acceptance flow | unit | `cargo test -p todotxt-tui date_picker_right_arrow_jumps_forward_one_week` | ✅ | ✅ green |
| 50-01-04 | 01 | 1 | DATE-UX-01, DATE-UX-02 | N/A | Cycling picker targets with `s` preserves a single date-entry workflow and keeps target intent visible in the picker state | unit + crate | `cargo test -p todotxt-tui s_cycles_date_picker_target_due_threshold_completed && cargo test -p todotxt-tui` | ✅ | ✅ green |
| 50-02-01 | 02 | 1 | AUTO-SEL-01 | N/A | Quick context/project setters open with a meaningful selected suggestion rather than an arbitrary reset | unit | `cargo test -p todotxt-tui quick_context_prefers_existing_token_when_popup_opens` | ✅ | ✅ green |
| 50-02-02 | 02 | 1 | AUTO-SEL-02 | N/A | Quick-setter selection preserves the current token while narrowing as long as it remains a valid candidate | unit + crate | `cargo test -p todotxt-tui quick_context_keeps_existing_token_selected_while_filtering && cargo test -p todotxt-tui` | ✅ | ✅ green |

## Wave 0 Requirements

- Existing Rust test infrastructure covers all Phase 50 requirements.
- Full crate suites passed for `todotxt-core` and `todotxt-tui` after the Phase 50 changes.
- No additional manual-only gaps were identified after reconstructing coverage from execution artifacts.

## Manual-Only Verifications

All phase behaviors have automated verification.

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-05-19

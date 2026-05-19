---
phase: 48
slug: recurring-workflow-core
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-18
---

# Phase 48 - Validation Strategy

> Per-phase validation contract reconstructed from execution artifacts and verification results.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | `crates/todotxt-core/Cargo.toml`, `crates/todotxt-cli/Cargo.toml`, `crates/todotxt-tui/Cargo.toml`, workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p todotxt-core recurrence` |
| **Full suite command** | `cargo test -p todotxt-core && cargo test -p todotxt-cli && cargo test -p todotxt-tui` |
| **Estimated runtime** | ~60 seconds |

## Sampling Rate

- **After every task commit:** Run the focused recurrence filter for the touched subsystem.
- **After every plan wave:** Run the relevant crate-level recurring coverage commands.
- **Before `$gsd-verify-work`:** Core, CLI, and TUI crate suites must all be green.
- **Max feedback latency:** <120 seconds.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 48-01-01 | 01 | 1 | REC-01 | N/A | Supported `rec:+...` and `rec:...` tokens are parsed, while malformed rules remain non-operative and do not panic | unit | `cargo test -p todotxt-core recurrence` | ✅ | ✅ green |
| 48-01-02 | 01 | 1 | REC-03 | N/A | Strict recurrence anchors from original `due:`, relative recurrence anchors from completion date, and no-due fallback uses completion date | unit | `cargo test -p todotxt-core recurrence` | ✅ | ✅ green |
| 48-01-03 | 01 | 1 | REC-01, REC-03 | N/A | Next occurrence preserves non-completion metadata, resets completion state, and serializes exactly one recalculated `due:` token | unit | `cargo test -p todotxt-core recurrence` | ✅ | ✅ green |
| 48-02-01 | 02 | 2 | REC-02, REC-03, REC-04 | N/A | CLI `do` on a recurring task completes the original and appends exactly one promptless next occurrence | integration | `cargo test -p todotxt-cli recurring_cli` | ✅ | ✅ green |
| 48-02-02 | 02 | 2 | REC-02, REC-03, REC-04 | N/A | CLI multi-ID `do` generates one next occurrence per newly completed recurring task and skips already-completed recurring tasks | integration | `cargo test -p todotxt-cli recurring_cli` | ✅ | ✅ green |
| 48-02-03 | 02 | 2 | REC-04 | N/A | CLI completion uses the shared core recurrence helper rather than a duplicate date-calculation path | integration + crate | `cargo test -p todotxt-cli && cargo test -p todotxt-core` | ✅ | ✅ green |
| 48-03-01 | 03 | 2 | REC-02, REC-03, REC-04 | N/A | TUI single-task completion creates one promptless next occurrence with the same strict/relative behavior as core | unit | `cargo test -p todotxt-tui recurring_tui` | ✅ | ✅ green |
| 48-03-02 | 03 | 2 | REC-02, REC-03, REC-04 | N/A | TUI bulk mark-done creates one next occurrence per newly completed recurring task and does not duplicate already-completed recurring tasks | unit | `cargo test -p todotxt-tui recurring_tui` | ✅ | ✅ green |
| 48-03-03 | 03 | 2 | REC-04 | N/A | TUI recurring completion preserves undo, selection clearing, and pane rebuild behavior while sharing the same recurrence contract as CLI | unit + crate | `cargo test -p todotxt-tui recurring_tui && cargo test -p todotxt-tui` | ✅ | ✅ green |

## Wave 0 Requirements

- Existing Rust test infrastructure covers all Phase 48 requirements.
- Full crate suites passed for `todotxt-core`, `todotxt-cli`, and `todotxt-tui` after the recurrence changes.

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

---
phase: 49
slug: archive-hygiene
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-19
---

# Phase 49 - Validation Strategy

> Per-phase validation contract reconstructed from execution artifacts and verification results.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | `crates/todotxt-core/Cargo.toml`, `crates/todotxt-cli/Cargo.toml`, `crates/todotxt-tui/Cargo.toml`, workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p todotxt-core archive_rotation` |
| **Full suite command** | `cargo test -p todotxt-core && cargo test -p todotxt-cli && cargo test -p todotxt-tui` |
| **Estimated runtime** | ~75 seconds |

## Sampling Rate

- **After every task commit:** Run the focused archive rotation filter for the touched subsystem.
- **After every plan wave:** Run the relevant crate-level archive coverage commands.
- **Before `$gsd-verify-work`:** Core, CLI, and TUI crate suites must all be green.
- **Max feedback latency:** <120 seconds.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 49-01-01 | 01 | 1 | DONE-01, DONE-03 | N/A | Monthly cadence maps archive writes into deterministic period buckets and produces stable rotated filenames such as `done-YYYY-MM.txt` | unit | `cargo test -p todotxt-core archive_rotation` | ✅ | ✅ green |
| 49-01-02 | 01 | 1 | DONE-01 | N/A | Shared core helper rotates only when an existing active archive belongs to an earlier period and contains content | unit | `cargo test -p todotxt-core archive_rotation` | ✅ | ✅ green |
| 49-01-03 | 01 | 1 | DONE-03 | N/A | CLI and TUI config expose cadence-based archive rotation with monthly defaults and no retention cleanup knobs | unit | `cargo test -p todotxt-cli archive && cargo test -p todotxt-tui archive` | ✅ | ✅ green |
| 49-02-01 | 02 | 2 | DONE-01, DONE-02 | N/A | CLI archive appends within the same period, rotates prior-period `done.txt` into the correct period file, and starts a fresh active archive when needed | integration | `cargo test -p todotxt-cli archive` | ✅ | ✅ green |
| 49-02-02 | 02 | 2 | DONE-01, DONE-02 | N/A | CLI archive reports rotation explicitly and preserves write-first safety when moving completed tasks out of `todo.txt` | integration + crate | `cargo test -p todotxt-cli archive && cargo test -p todotxt-cli` | ✅ | ✅ green |
| 49-03-01 | 03 | 2 | DONE-01, DONE-02 | N/A | TUI archive confirmation preserves existing cancel/undo boundaries while rotating prior-period `done.txt` during archive writes | unit | `cargo test -p todotxt-tui archive` | ✅ | ✅ green |
| 49-03-02 | 03 | 2 | DONE-01, DONE-02 | N/A | TUI archive surfaces explicit rotation feedback and keeps CLI/TUI archive semantics aligned | unit + crate | `cargo test -p todotxt-tui archive && cargo test -p todotxt-tui` | ✅ | ✅ green |

## Wave 0 Requirements

- Existing Rust test infrastructure covers all Phase 49 requirements.
- Full crate suites passed for `todotxt-core`, `todotxt-cli`, and `todotxt-tui` after the archive rotation changes.

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

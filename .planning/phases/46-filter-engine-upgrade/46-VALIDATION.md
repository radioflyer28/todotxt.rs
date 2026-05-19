---
phase: 46
slug: filter-engine-upgrade
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-15
---

# Phase 46 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | `crates/todotxt-core/Cargo.toml`, `crates/todotxt-cli/Cargo.toml`, workspace `Cargo.toml` |
| **Quick run command** | `cargo test -p todotxt-core filter` |
| **Full suite command** | `cargo test -p todotxt-core filter && cargo test -p todotxt-cli list` |
| **Estimated runtime** | ~20 seconds |

## Sampling Rate

- **After every task commit:** Run quick command from the relevant subsystem crate.
- **After every plan wave:** Run full command.
- **Before `$gsd-verify-work`:** Full suite must be green.
- **Max feedback latency:** <120 seconds.

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 46-01-01 | 01 | 1 | FILT-01 | N/A | `Filter::from_query("@work\|@home")` matches either context/project/priority token | unit | `cargo test -p todotxt-core filter` | ✅ | ✅ green |
| 46-01-01 | 01 | 1 | FILT-02 | N/A | OR terms compose with whitespace AND terms | unit | `cargo test -p todotxt-core filter` | ✅ | ✅ green |
| 46-01-01 | 01 | 1 | FILT-03 | N/A | Unsupported grouped negation is not introduced; grouped forms treated as non-special syntax | unit | `cargo test -p todotxt-core filter` | ✅ | ✅ green |
| 46-01-01 | 01 | 1 | D-01 | N/A | `parse_token` handles token-local `|` expressions | unit | `cargo test -p todotxt-core filter` | ✅ | ✅ green |
| 46-01-01 | 01 | 1 | D-05 | N/A | Empty OR branches are ignored | unit | `cargo test -p todotxt-core filter` | ✅ | ✅ green |
| 46-02-01 | 02 | 2 | FILT-01 | N/A | CLI positional and `--filter` OR examples return expected rows | integration | `cargo test -p todotxt-cli list` | ✅ | ✅ green |
| 46-02-01 | 02 | 2 | FILT-02 | N/A | CLI OR behavior composes with additional AND terms | integration | `cargo test -p todotxt-cli list` | ✅ | ✅ green |
| 46-02-01 | 02 | 2 | FILT-03 | N/A | CLI docs state unsupported grouped negation explicitly | manual review of docs + integration | `cargo test -p todotxt-cli list` | ✅ | ✅ green |

## Wave 0 Requirements

- Existing Rust test infrastructure covers all phase requirements.

## Manual-Only Verifications

*If none: "All phase behaviors have automated verification."*

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 120s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-05-15


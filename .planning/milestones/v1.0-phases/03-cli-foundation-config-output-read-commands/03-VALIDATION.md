---
phase: 3
slug: cli-foundation-config-output-read-commands
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-15
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in (`cargo test`) + `assert_cmd` 2.2.0 integration tests |
| **Config file** | none — `cargo test` needs no config file |
| **Quick run command** | `cargo test -p todotxt-cli` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~10–15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p todotxt-cli`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green + `cargo clippy --workspace -- -D warnings`
- **Max feedback latency:** ~15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|--------|
| 03-01-01 | 01 | 1 | CFG-01, CFG-02 | T-03-01 | Config dir created with restricted perms; no secrets in config | unit | `cargo test -p todotxt-cli config` | ⬜ pending |
| 03-01-02 | 01 | 1 | READ-06, READ-07, READ-08 | T-03-02 | JSON envelope never exposes internal paths | unit | `cargo test -p todotxt-cli output` | ⬜ pending |
| 03-02-01 | 02 | 2 | READ-01 | T-03-03 | Filter args not injected/escaped via shell | integration | `cargo test -p todotxt-cli list` | ⬜ pending |
| 03-02-02 | 02 | 2 | READ-02, READ-03, READ-04, READ-05 | T-03-04 | show ID is integer-validated before use | integration | `cargo test -p todotxt-cli stats_show` | ⬜ pending |
| 03-03-01 | 03 | 3 | PLAT-01 | — | N/A | integration | `cargo test -p todotxt-cli completions` | ⬜ pending |
| 03-03-02 | 03 | 3 | READ-08, READ-06 | T-03-02 | Exit codes correct; JSON error not leaked to stderr | integration | `cargo test -p todotxt-cli exit_codes` | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers the core test harness (`cargo test`). The following must be created as part of Wave 1:

- [ ] `crates/todotxt-cli/tests/helpers.rs` — shared test utilities (TempDir + sample todo.txt builder)
- [ ] Integration test scaffold in `crates/todotxt-cli/tests/` before command implementation

*All test files are created in the same plan wave as the feature they test.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `todotxt list` renders priority badge in correct terminal color (red/yellow/green) | READ-07 | ANSI color rendering requires visual inspection | Run `todotxt list` against a sample todo.txt with (A), (B), (C) tasks; verify colors |
| `NO_COLOR=1 todotxt list` produces no ANSI codes | READ-07 | Can be verified by piping to `cat` and checking for ESC chars | `NO_COLOR=1 todotxt list \| cat -v \| grep -c '\^\\['` should output `0` |
| Config auto-created at correct platform path | CFG-01 | Path is OS-dependent | Delete config, run `todotxt list`, verify config appears at platform path |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending

---
phase: 21
slug: smart-text-normalization
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-25
---

# Phase 21 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[test]` + `rstest 0.26` |
| **Config file** | `Cargo.toml` `[dev-dependencies]` (already configured) |
| **Quick run command** | `cargo test -p todotxt-core normalize` |
| **Full suite command** | `cargo test --workspace` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p todotxt-core normalize`
- **After every plan wave:** Run `cargo test --workspace`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 21-01-01 | 01 | 0 | NORM-01 – NORM-05 | — | N/A | unit | `cargo test -p todotxt-core normalize` | ❌ W0 | ⬜ pending |
| 21-01-02 | 01 | 1 | NORM-01 | — | N/A | unit | `cargo test -p todotxt-core normalize::priority` | ❌ W0 | ⬜ pending |
| 21-01-03 | 01 | 1 | NORM-01 | — | N/A | unit | `cargo test -p todotxt-core normalize::priority_conflict` | ❌ W0 | ⬜ pending |
| 21-01-04 | 01 | 1 | NORM-02 | — | N/A | unit | `cargo test -p todotxt-core normalize::projects` | ❌ W0 | ⬜ pending |
| 21-01-05 | 01 | 1 | NORM-03 | — | N/A | unit | `cargo test -p todotxt-core normalize::contexts` | ❌ W0 | ⬜ pending |
| 21-01-06 | 01 | 1 | NORM-04 | — | N/A | unit | `cargo test -p todotxt-core normalize::dates` | ❌ W0 | ⬜ pending |
| 21-01-07 | 01 | 1 | NORM-05 | — | N/A | unit | `cargo test -p todotxt-core normalize::unknown_tokens` | ❌ W0 | ⬜ pending |
| 21-02-01 | 02 | 2 | NORM-06 | — | N/A | integration | `cargo test -p todotxt-tui` | ✅ exists | ⬜ pending |
| 21-02-02 | 02 | 2 | NORM-01–06 | — | N/A | integration | `cargo test --workspace` | ✅ exists | ⬜ pending |
| 21-03-01 | 03 | 3 | NORM-01–06 | — | N/A | integration | `cargo test --workspace` | ✅ exists | ⬜ pending |
| 21-03-02 | 03 | 3 | NORM-06 | — | N/A | integration | `cargo test -p todotxt-cli` | ✅ exists | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/todotxt-core/tests/normalize_tests.rs` — test stubs for NORM-01 through NORM-05 (priority, projects, contexts, dates, unknown tokens)
- [ ] No framework install needed — `rstest` already in `[dev-dependencies]`
- [ ] No new test config files needed

*Plan 21-01 Wave 0 creates the test file with stubs before implementing `normalize_append`.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Config toggle `normalize_append = false` falls back to raw concat | NORM-01 | Requires config file edit + TUI interaction | Edit `config.toml` to set `normalize_append = false`, run TUI, append `(B)` to a task, confirm raw concat (not normalized) |
| Config toggle `normalize_edit = false` skips normalization on save | NORM-01 | Requires config file edit + TUI interaction | Edit `config.toml` to set `normalize_edit = false`, run TUI, edit a task with priority token, confirm Task::parse result |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending

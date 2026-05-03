---
phase: 01
slug: workspace-bootstrap-core-library-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-15
---

# Phase 01 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | rstest 0.26 + insta 1.47 (Rust native) |
| **Config file** | `crates/todotxt-core/Cargo.toml` (dev-dependencies section) |
| **Quick run command** | `cargo test -p todotxt-core` |
| **Full suite command** | `cargo test -p todotxt-core && cargo clippy -p todotxt-core -- -D warnings` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p todotxt-core`
- **After every plan wave:** Run `cargo test -p todotxt-core && cargo clippy -p todotxt-core -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | CORE-01 | — | N/A | unit | `cargo test -p todotxt-core` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | CORE-02 | — | N/A | snapshot | `cargo insta test -p todotxt-core` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 2 | CORE-03 | T-01-01 | Atomic write prevents data loss on crash | integration | `cargo test -p todotxt-core` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 2 | CORE-07 | — | N/A | integration | `cargo test -p todotxt-core` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/todotxt-core/tests/` directory created with test infrastructure
- [ ] `rstest` and `insta` added as dev-dependencies in `crates/todotxt-core/Cargo.toml`
- [ ] Test fixtures file at `crates/todotxt-core/tests/fixtures/todo.txt`

*Wave 0 is embedded in Plan 01 — workspace setup creates test infrastructure.*

---

## Manual-Only Verifications

*All phase behaviors have automated verification.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending

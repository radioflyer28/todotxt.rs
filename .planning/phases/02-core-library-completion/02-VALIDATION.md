---
phase: 2
slug: core-library-completion
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-15
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test runner + `rstest 0.26` + `insta 1.47` |
| **Config file** | `crates/todotxt-core/Cargo.toml` |
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
| 02-01-01 | 01 | 1 | CORE-05 | — | N/A (pure logic) | unit | `cargo test -p todotxt-core filter` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | CORE-06 | — | N/A (pure logic) | unit | `cargo test -p todotxt-core sort` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 1 | CORE-03 | — | fail-fast index validation | unit | `cargo test -p todotxt-core batch` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02 | 1 | CORE-08 | — | N/A | unit | `cargo test -p todotxt-core portable` | ❌ W0 | ⬜ pending |
| 02-03-01 | 03 | 2 | CORE-04 | — | no path traversal in watch | integration | `cargo test -p todotxt-core --features watching watcher` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/todotxt-core/tests/filter_tests.rs` — stubs for CORE-05 filter matrix
- [ ] `crates/todotxt-core/tests/sort_tests.rs` — stubs for CORE-06 sort stability
- [ ] `crates/todotxt-core/tests/batch_portable_tests.rs` — stubs for CORE-03 batch + CORE-08 portable
- [ ] `crates/todotxt-core/tests/watcher_tests.rs` — integration test stubs for CORE-04 (feature-gated)

*All test files use existing `rstest` infrastructure (already in `[dev-dependencies]`).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| File watcher fires on save from external editor | CORE-04 | Cross-process file event timing | 1. Run watcher integration test with `--test-threads=1`. 2. The 2s timeout in the test covers this if using `std::thread::sleep`. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending

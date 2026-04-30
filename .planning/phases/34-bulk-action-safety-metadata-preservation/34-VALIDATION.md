---
phase: 34
slug: bulk-action-safety-metadata-preservation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-30
---

# Phase 34 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `cargo test` (todotxt-core unit tests) |
| **Config file** | `crates/todotxt-core/Cargo.toml` |
| **Quick run command** | `cargo test -p todotxt-core` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p todotxt-core`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** ~10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 34-01-01 | 01 | 1 | CAP-05 | — | metadata preserved after mutation | unit | `cargo test -p todotxt-core -- task::tests` | ❌ W0 | ⬜ pending |
| 34-01-02 | 01 | 1 | BULK-01,BULK-02 | — | count preview shows N; cancel preserves selection | integration | `cargo test -p todotxt-tui -- count_preview` | ❌ W0 | ⬜ pending |
| 34-02-01 | 02 | 1 | CAP-04 | — | priority picker opens, navigates, accepts A–Z | integration | `cargo test -p todotxt-tui -- priority_picker` | ❌ W0 | ⬜ pending |
| 34-02-02 | 02 | 1 | TAG-03,BULK-03 | — | bulk priority/due operations preserve all metadata | unit | `cargo test -p todotxt-core -- mutation_roundtrip` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/todotxt-core/src/task.rs` test module — mutation round-trip tests for `with_priority` and `with_due_date` (completed/priority/context/project preservation)
- [ ] Integration test stubs in `crates/todotxt-tui/tests/` — count preview gate (N > 1 → mode transition), cancel path (Esc preserves `selected_tasks`)

*Existing `cargo test` infrastructure covers the framework; only new test functions needed.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Priority picker overlay renders correctly in terminal | CAP-04 | TUI rendering requires visual inspection | Run app, select 2+ tasks, press `i`, verify overlay shows "Setting priority — N tasks" and A–Z list |
| Type-to-jump in priority picker jumps cursor to correct letter | CAP-04 | Keyboard interaction requires live terminal | In priority picker, type `e`, verify cursor jumps to `E` entry |
| AppendTextConfirm banner appears before text entry for T with N > 1 | BULK-01 | TUI mode transition requires visual inspection | Select 3 tasks, press `T`, verify count banner shows "Appending to 3 tasks" before text box opens |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending

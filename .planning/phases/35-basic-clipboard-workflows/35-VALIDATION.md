---
phase: 35
slug: basic-clipboard-workflows
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-30
---

# Phase 35 — Validation Strategy

> Nyquist validation audit for completed phase 35 (State B reconstruction from PLAN/SUMMARY artifacts).

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` |
| **Config file** | `crates/todotxt-tui/Cargo.toml` |
| **Quick run command** | `cargo test -p todotxt-tui y_copies_active_task_to_clipboard -- --nocapture` |
| **Full suite command** | `cargo test -p todotxt-tui` |
| **Estimated runtime** | ~30-60 seconds |

---

## Sampling Rate

- **After every task commit:** Run a focused command for the touched behavior
- **After every plan wave:** Run `cargo test -p todotxt-tui`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** < 60 seconds for focused checks

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 35-01-01 | 01 | 1 | CLIP-01 | — | Clipboard dependency and lazy init compile and execute safely | unit | `cargo test -p todotxt-tui y_copies_active_task_to_clipboard -- --nocapture` | ✅ | ✅ green |
| 35-01-02 | 01 | 1 | CLIP-01 | — | Active-task copy writes raw todo line to clipboard | unit | `cargo test -p todotxt-tui y_copies_active_task_to_clipboard -- --nocapture` | ✅ | ✅ green |
| 35-01-03 | 01 | 1 | CLIP-01 | — | Multi-select copy preserves descending canonical order | unit | `cargo test -p todotxt-tui y_copies_selected_tasks_in_descending_canonical_order -- --nocapture` | ✅ | ✅ green |
| 35-01-04 | 01 | 1 | CLIP-01 | — | `y` copy path remains non-crashing in normal mode flow | unit | `cargo test -p todotxt-tui y_copies_active_task_to_clipboard -- --nocapture` | ✅ | ✅ green |
| 35-02-01 | 02 | 2 | CLIP-02 | — | Cut behavior composes copy + existing delete semantics | unit | `cargo test -p todotxt-tui cut_composes_copy_then_delete_for_single_selected_task -- --nocapture` | ✅ | ✅ green |
| 35-02-02 | 02 | 2 | CLIP-03 | — | `p` paste adds one task per non-empty clipboard line | unit | `cargo test -p todotxt-tui p_pastes_each_non_empty_clipboard_line_as_task -- --nocapture` | ✅ | ✅ green |
| 35-02-03 | 02 | 2 | CLIP-03 | — | Paste appends tasks in source order | unit | `cargo test -p todotxt-tui p_pastes_each_non_empty_clipboard_line_as_task -- --nocapture` | ✅ | ✅ green |
| 35-02-04 | 02 | 2 | CLIP-04 | — | Ctrl+V in Adding mode inserts first clipboard line only | unit | `cargo test -p todotxt-tui ctrl_v_in_adding_mode_pastes_first_clipboard_line_only -- --nocapture` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

- All phase behaviors have automated verification.

---

## Validation Audit 2026-04-30

| Metric | Count |
|--------|-------|
| Gaps found | 5 |
| Resolved | 5 |
| Escalated | 0 |

---

## Validation Sign-Off

- [x] All tasks have automated verification coverage
- [x] Sampling continuity preserved across both plans
- [x] No Wave 0 dependencies required
- [x] No watch-mode flags used
- [x] Feedback latency < 60s for focused checks
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved 2026-04-30

---
id: SEED-005
status: dormant
planted: 2026-04-28
planted_during: v1.3 / Phase 22 (keymap-help-parity)
trigger_when: any milestone that adds new AppMode variants or tests the key dispatch path
scope: Small
---

# SEED-005: Add unit tests for Phase 22 manual-only validation gaps (mode transitions + filter mutations)

## Why This Matters

Phase 22 VALIDATION.md has 11 manual-only items that could be automated. They all follow
the same pattern — construct a `KeyEvent`, call `handle_normal_key` (or a mode handler),
then assert `app.mode` or `app.filter_query`. The only blocker is that `App::new` requires
a real `TaskList` from a path, which makes it awkward to test in isolation.

Adding a `make_app_with_keymap` helper (or extending the existing `make_app_with_tasks`)
to accept a config with a `keymap` field would unblock all 11 tests and bring Phase 22
to `nyquist_compliant: true`.

## When to Surface

**Trigger:** When we add new `AppMode` variants, expand key dispatch, or run a general
test-coverage improvement milestone.

This seed should be presented during `/gsd-new-milestone` when the milestone scope
matches any of these conditions:
- Adds new `AppMode` variants to `app.rs`
- Works on test infrastructure for the TUI crate
- Is a test/quality/coverage sprint

## Scope Estimate

**Small** — A few hours. The test helper is one function; each test is ~10 lines.
Estimate: 1 phase, ~2 plans.

## Gaps to Close (from 22-VALIDATION.md)

| Gap | Requirement | Test approach |
|-----|-------------|---------------|
| `App::new` initializes `effective_keymap` and `keymap_warnings` | KEY-01 | `make_app_with_tasks` already calls `App::new`; assert `app.effective_keymap.len() == 19` and `app.keymap_warnings.is_empty()` |
| `handle_normal_key` dispatches 16+ actions via `key_is_action` | KEY-01 | Construct `KeyEvent { code: Char('n'), .. }`, call `handle_normal_key`, assert `app.mode == AppMode::Adding` |
| Status bar `⚠ keymap: N warning(s)` text | KEY-02 | Inject `keymap_warnings` into app directly; call the status-bar logic inline (copy pattern from existing `status_bar_selection_indicator_*` tests) |
| Status bar silent when warnings empty | KEY-02 | Same pattern — empty `keymap_warnings` vec |
| `'!'` → `AppMode::KeymapErrors` | KEY-02 | `app.keymap_warnings.push("x".into()); handle_normal_key(Char('!')); assert mode == KeymapErrors` |
| Esc closes `KeymapErrors` | KEY-02 | `app.mode = KeymapErrors; handle_keymap_errors_key(Esc); assert mode == Normal` |
| `'0'` clears `filter_query` | PAR-01 | `app.filter_query = "foo".into(); handle_normal_key(Char('0')); assert app.filter_query.is_empty()` |
| `'1'`-`'9'` applies preset | PAR-01 | Populate `app.config.presets` with a `TuiPreset`; `handle_normal_key(Char('1'))`; assert `filter_query` updated |
| `'.'` calls `task_list.reload()` | PAR-01 | Needs real temp-file; use `tempfile` crate or write to `std::env::temp_dir()`; assert `task_list.len()` changes |
| `'?'` → `AppMode::Help` | PAR-02 | `handle_normal_key(Char('?')); assert mode == Help` |
| Esc/q closes `Help` | PAR-02 | `app.mode = Help; handle_help_key(Esc); assert mode == Normal` |

## Breadcrumbs

- `crates/todotxt-tui/src/app.rs` — `make_app_with_tasks` test helper (~line 2060); `handle_normal_key`; `handle_keymap_errors_key`; `handle_help_key`; `AppMode` enum
- `crates/todotxt-tui/src/config.rs` — `resolve_keymap`, `TuiPreset`, `TuiConfig`
- `.planning/phases/22-keymap-help-parity/22-VALIDATION.md` — full gap table
- `.planning/phases/22-keymap-help-parity/22-01-PLAN.md` — KEY-01 behavior specs
- `.planning/phases/22-keymap-help-parity/22-02-PLAN.md` — KEY-02 behavior specs
- `.planning/phases/22-keymap-help-parity/22-03-PLAN.md` — PAR-01/PAR-02 behavior specs

## Notes

The `'.'` reload test is the only one that needs real I/O. All others are pure state
machine tests that can run without touching the filesystem. Consider splitting into two
plans: one for the 10 pure state tests, one for the reload I/O test (which may need
`tempfile` added as a dev-dependency).

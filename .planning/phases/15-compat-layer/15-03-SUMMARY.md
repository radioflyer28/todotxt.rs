# 15-03 Summary — Regression Tests

## Status: COMPLETE

## What was done

Created `crates/todotxt-cli/tests/compat_tests.rs` with 31 integration tests covering the full Phase 15 compat surface.

### Test groups

| Group | Tests | Coverage |
|-------|-------|----------|
| Alias tests | 9 | `a`, `rm`, `done`, `dp`, `p`, `app`, `prep`, `lsc`, `lsprj` |
| `--all` flag | 5 | future `t:` hidden/shown, `h:1` hidden/shown, past `t:` always shown |
| `--compat` flag | 2 | format `{N} {raw}`, 1-based numbering, no table borders |
| `listpri` | 6 | default (A-Z), single letter, range, `lsp` alias, no matches, invalid spec |
| `listall` | 4 | merges todo+done, missing done.txt, `lsa` alias, shows hidden tasks |
| `deduplicate` | 5 | removes exact dup, no-op when clean, multiple dups, case-sensitive, idempotent |

### Fix applied during testing
- Initial test assumed 1-based IDs from `list --compat`. Actual output was 0-based (filter returns 0-based indices). Fixed `list.rs` to use `id + 1` to match the 1-based convention used by the table renderer. Tests updated to assert on correct values.

## Deviations
- Test used `predicates::prelude::PredicateBooleanExt` import for `.not()` — not imported in plan template.

## Verification
- `cargo test --test compat_tests` → 31 passed, 0 failed ✅
- `cargo test` (full suite) → all tests pass, no regressions ✅

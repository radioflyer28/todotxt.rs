---
plan: "02-01"
phase: 02-core-library-completion
status: complete
commits:
  - aa51e74
  - cd6a3ad
key-files:
  created:
    - crates/todotxt-core/src/filter.rs
    - crates/todotxt-core/src/sort.rs
    - crates/todotxt-core/src/portable.rs
  modified:
    - crates/todotxt-core/src/lib.rs
tests-added: 22
tests-total: 33
---

## Plan 02-01 Summary: Pure Logic Modules

Created three self-contained modules with no new dependencies: `filter.rs`, `sort.rs`, and `portable.rs`. Updated `lib.rs` to export all new public types.

### What Was Built

**filter.rs** — `FilterTerm` enum (12 variants) + `Filter` struct:
- `DONE`/`-DONE` case-sensitive; `due:*` tokens case-insensitive
- Pre-filters: `suppress_hidden` (h:1 exclusion) and `suppress_future_threshold`
- `matches_with_date(task, today)` for fully deterministic testing
- All negated due variants converted to idiomatic `is_none_or()` (clippy compliant)
- 15 inline unit tests covering all token types and AND-logic

**sort.rs** — `SortOrder` enum (5 variants, `#[non_exhaustive]`):
- Priority, DueDate, Alphabetical, Project, Context
- `None`-last for all orderings (consistent with C# reference implementation)
- 6 inline unit tests covering all sort orders

**portable.rs** — `resolve_config_path()`:
- Returns `binary_dir` when `config.toml` exists beside the binary; `platform_dir` otherwise
- 2 inline unit tests using `tempfile`

**lib.rs** — updated to export: `Filter`, `FilterTerm`, `SortOrder`, `resolve_config_path`

### Verification

- `cargo clippy -p todotxt-core -- -D warnings` → ✓ clean (0 warnings)
- `cargo test -p todotxt-core` → ✓ 33/33 passed

### Deviations

- `map_or(false, ...)` pattern replaced with `is_some_and()` per clippy suggestion
- Negated `!is_some_and()` patterns further simplified to `is_none_or()` per clippy `nonminimal_bool` lint

### Self-Check: PASSED

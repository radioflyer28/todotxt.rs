---
plan: "02-02"
phase: 02-core-library-completion
status: complete
commits:
  - ea495d8
  - 864ec3b
key-files:
  modified:
    - crates/todotxt-core/src/task_list.rs
  created:
    - crates/todotxt-core/tests/filter_tests.rs
    - crates/todotxt-core/tests/sort_tests.rs
    - crates/todotxt-core/tests/batch_tests.rs
tests-added: 24
tests-total: 95
---

## Plan 02-02 Summary: TaskList Integration + Test Coverage

Extended `TaskList` with three new methods and added integration tests covering filter matrix, sort stability, and batch atomicity.

### What Was Built

**task_list.rs** — three new methods:
- `filter(&self, filter: &Filter) -> Vec<(usize, &Task)>` — returns indexed matching tasks using `Filter::matches()`
- `sort(&mut self, order: SortOrder)` — stable in-memory sort; does NOT save to disk
- `batch_update(&mut self, replacements: Vec<(usize, Task)>) -> Result<(), TodoError>` — validates all indices first (fail-fast), then applies all replacements, then single `save()` call for atomicity

**filter_tests.rs** — 13 integration tests:
- DONE / −DONE case-sensitive matching
- `due:today`, `due:past`, `due:future`, `due:active` token coverage
- Substring include/exclude (case-insensitive)
- Multi-token AND logic
- `suppress_hidden` (h:1) and `suppress_future_threshold` pre-filter behaviour
- Index accuracy for non-contiguous result sets

**sort_tests.rs** — 7 integration tests:
- Priority order (A before B before None); stability for equal priorities
- DueDate earliest-first, None-last
- Alphabetical case-insensitive
- Project and Context first-tag sort, None-last
- Confirmed `sort()` does NOT write to disk (on-disk file unchanged)

**batch_tests.rs** — 4 integration tests:
- Valid 2-replacement batch: memory and disk both updated
- Single replacement: works correctly
- Out-of-bounds fail-fast: error returned, no mutation applied at index 0
- Empty batch: no-op, file round-trips cleanly

### Verification

- `cargo clippy -p todotxt-core -- -D warnings` → ✓ clean (0 warnings)
- `cargo test -p todotxt-core` → ✓ 95/95 passed

### Deviations

- Used `TempDir` pattern (not `NamedTempFile`) to match existing test convention and avoid Windows file-handle conflicts during `save()` atomic rename.
- `tl.path()` getter used in batch tests to read the saved file back from disk.

### Self-Check: PASSED

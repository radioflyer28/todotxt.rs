---
plan: "02-03"
phase: 02-core-library-completion
status: complete
commits:
  - 5a52034
  - 76ea109
  - d451ceb
key-files:
  created:
    - crates/todotxt-core/src/watcher.rs
    - crates/todotxt-core/tests/watcher_tests.rs
  modified:
    - Cargo.toml
    - crates/todotxt-core/Cargo.toml
    - crates/todotxt-core/src/error.rs
    - crates/todotxt-core/src/lib.rs
tests-added: 3
tests-total: 98
---

## Plan 02-03 Summary: File Watcher Feature Flag

Added `FileWatcher` behind an optional `watching` Cargo feature, keeping the default build free of extra transitive dependencies.

### What Was Built

**Cargo dependency changes:**
- `notify-debouncer-mini = "0.7"` added to `[workspace.dependencies]` in workspace `Cargo.toml`
- `crates/todotxt-core/Cargo.toml` gains `[features] watching = ["dep:notify-debouncer-mini"]` and the dep as optional

**error.rs** — `TodoError::Watch` variant added:
- `#[cfg(feature = "watching")]` gate
- `#[from] notify_debouncer_mini::notify::Error` provides ergonomic `?` propagation

**watcher.rs** — `FileWatcher` struct:
- `new(path, Arc<dyn Fn() + Send + Sync>)` watches the parent directory with `RecursiveMode::NonRecursive`
- Filters debounce events to the specific target filename to avoid false positives from sibling files
- 1-second debounce window via `new_debouncer(Duration::from_secs(1), ...)`
- `stop(self)` drops the debouncer explicitly; background thread also stops on Drop
- Returns `TodoError::Watch` if the underlying notify watcher fails to start

**lib.rs** updated:
- `#[cfg(feature = "watching")] pub mod watcher`
- `#[cfg(feature = "watching")] pub use watcher::FileWatcher`

**watcher_tests.rs** — 3 integration tests:
- `watcher_fires_callback_on_file_write` — callback fires within 3s of a direct write
- `watcher_fires_on_atomic_write_rename` — callback fires after write-to-temp + rename pattern (simulates `TaskList::save()`)
- `watcher_stop_does_not_panic` — `stop()` is safe to call

### Verification

- `cargo build -p todotxt-core` → ✓ clean (no watching dep compiled)
- `cargo build -p todotxt-core --features watching` → ✓ clean
- `cargo clippy -p todotxt-core -- -D warnings` → ✓ clean
- `cargo clippy -p todotxt-core --features watching -- -D warnings` → ✓ clean
- `cargo test -p todotxt-core --features watching -- --test-threads=1` → ✓ 98/98 passed (watcher tests ran in 2.22s)

### Deviations

None — implemented exactly as specified in the plan.

### Self-Check: PASSED

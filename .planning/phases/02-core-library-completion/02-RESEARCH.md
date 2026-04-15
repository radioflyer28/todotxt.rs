# Phase 2 Research: Core Library Completion

**Phase:** 2 — Core Library Completion
**Date:** 2026-04-15
**Status:** Complete

## Research Summary

All work is inside `crates/todotxt-core/`. Five deliverables: `filter.rs`, `sort.rs`, `watcher.rs` (optional feature), additions to `task_list.rs` (`batch_update`), and `portable.rs`. Only `watcher.rs` requires a new dependency. Filter, sort, batch, and portable use only the already-available crates.

---

## Standard Stack

| Component | Crate | Already in workspace |
|-----------|-------|---------------------|
| File watching | `notify-debouncer-mini 0.7.0` (re-exports `notify 8.2.0`) | ❌ Add |
| Filter engine | pure Rust + existing `chrono` | ✅ |
| Sort engine | pure Rust + existing `chrono` | ✅ |
| Batch update | extend existing `task_list.rs` | ✅ |
| Portable mode | pure Rust `std::path` | ✅ |

---

## 1. File Watcher — `notify-debouncer-mini 0.7.0`

### Cargo Setup

In `Cargo.toml` (workspace root), add to `[workspace.dependencies]`:
```toml
notify-debouncer-mini = "0.7"
```

In `crates/todotxt-core/Cargo.toml`, add:
```toml
[features]
watching = ["dep:notify-debouncer-mini"]

[dependencies]
notify-debouncer-mini = { workspace = true, optional = true }
```

### API (confirmed from docs.rs/notify-debouncer-mini/0.7.0)

The 0.7.0 re-exports `notify 8.2.0` — **do not add `notify` as a separate dependency**. Use `notify_debouncer_mini::notify::*` for all notify types.

```rust
use notify_debouncer_mini::{
    new_debouncer, DebounceEventResult, DebounceEventHandler, Debouncer,
    notify::{RecommendedWatcher, RecursiveMode},
};
use std::time::Duration;

// Create a debounced watcher with 1-second window
let mut debouncer = new_debouncer(
    Duration::from_secs(1),
    |res: DebounceEventResult| { /* handler */ },
)?;

// Start watching
debouncer.watcher().watch(path, RecursiveMode::NonRecursive)?;

// Watcher stops when Debouncer is dropped
drop(debouncer); // equivalent to FileWatcher::stop(self)
```

**`DebounceEventHandler`** is a trait implemented for all `F: Fn(DebounceEventResult) + Send + 'static`. Our `Arc<dyn Fn() + Send + Sync + 'static>` callback satisfies this when wrapped: `move |_res| { callback(); }`.

**`DebounceEventResult`** is `Result<Vec<DebouncedEvent>, Vec<notify::Error>>`. Each `DebouncedEvent` has `path: PathBuf` and `kind: DebouncedEventKind`.

### Watch Strategy: Parent Directory + Filename Filter

**Problem:** Watching a file by path directly is fragile with atomic writes. When `TaskList::save()` does a tempfile → rename, the file's inode changes. On Linux, `inotify` stops watching the old inode. The new inode is not watched.

**Solution:** Watch the **parent directory** (`RecursiveMode::NonRecursive`) and filter events by filename.

```rust
let parent = path.parent().unwrap_or(Path::new("."));
let filename = path.file_name().unwrap().to_os_string();

debouncer.watcher().watch(parent, RecursiveMode::NonRecursive)?;

// In callback:
Ok(events) => {
    if events.iter().any(|e| e.path.file_name() == Some(filename.as_os_str())) {
        callback();
    }
}
```

This pattern also handles editor behavior (vim creates temp file + rename, same as our atomic write).

### Error Type — New `TodoError` Variant Required

`notify::Error` is **not** `std::io::Error`. `new_debouncer` returns `notify::Error`, and `watcher().watch()` also returns `notify::Error`.

Add a new variant to `TodoError` (feature-gated):

```rust
// In error.rs
#[cfg(feature = "watching")]
#[error("file watcher error: {0}")]
Watch(#[from] notify_debouncer_mini::notify::Error),
```

**Important:** `#[from]` works here because `notify::Error` implements `std::error::Error`. The feature gate keeps `error.rs` free of the `notify` dependency when `watching` is not enabled.

### Thread Safety

The debouncer handler runs on a background thread. `Arc<dyn Fn() + Send + Sync + 'static>` is the correct callback type — it's `Clone` (via `Arc::clone`), `Send` (required for background thread), and `Sync` (safe to call concurrently if needed).

Move the Arc clone into the closure:
```rust
let cb = Arc::clone(&callback);
let handler = move |res: DebounceEventResult| {
    if let Ok(events) = res {
        if events.iter().any(|e| e.path.file_name() == Some(filename.as_os_str())) {
            cb();
        }
    }
};
```

### Stopping the Watcher

`Debouncer` drops the background thread on `Drop`. So `FileWatcher::stop(self)` is simply:
```rust
pub fn stop(self) {
    // debouncer is dropped here, background thread stops
    drop(self.debouncer);
}
```

No explicit `stop()` call on the debouncer is needed (there is no `debouncer.stop()` API — drop is sufficient).

### cfg(feature) Guard Pattern

All watcher code must be behind `#[cfg(feature = "watching")]`. The `FileWatcher` struct, `impl`, and the `Watch` error variant in `error.rs` all need this guard.

---

## 2. Filter Engine — `filter.rs`

### No New Dependencies

Filter uses only:
- `chrono::Local` (already in workspace) for `today` date comparisons
- `task.to_raw()` for substring matching
- `task.due_status()` for due token evaluation
- `task.threshold_date` for suppression
- `task.completed` for DONE/not-DONE

### `h:1` Detection

The `h:1` hidden tag check must match C# behavior. C# checks `task.raw.Contains("h:1")`. In Rust, use `task.to_raw().contains("h:1")`. This is intentionally simple — a task with `h:1` in the body (even in a project name like `+gh:1`) would be suppressed. This matches the C# reference behavior.

### `due:active` Semantics

`due:active` = has a due date AND that date is ≤ today (overdue OR today). Map to:
```rust
FilterTerm::DueActive => {
    match task.due_date {
        None => false,
        Some(d) => d <= today,
    }
}
```

### Threshold Detection

`suppress_future_threshold` suppresses tasks where `threshold_date > today`:
```rust
if filter.suppress_future_threshold {
    if let Some(t) = task.threshold_date {
        if t > today {
            return false; // pre-filter, before token evaluation
        }
    }
}
```

### Implementation Pattern

The filter logic evaluates in this order (matching C# `FilterList`):
1. Pre-filter: `suppress_hidden` check → exclude `h:1` tasks
2. Pre-filter: `suppress_future_threshold` → exclude future-threshold tasks
3. Token AND-evaluation: all `FilterTerm`s must match

`TaskList::filter` returns `Vec<(usize, &Task)>` with the index — so callers know the position of each matching task. This is more useful than `Vec<&Task>` alone and avoids a second lookup for mutations.

**Revised API (more useful than CONTEXT.md's Vec<&Task>):**
```rust
pub fn filter(&self, filter: &Filter) -> Vec<(usize, &Task)>
```
Returning both index and reference. This is a pure CONTEXT extension — CONTEXT said `Vec<&Task>`, but `Vec<(usize, &Task)>` is strictly more useful since the index is required for any subsequent edit/delete operations.

---

## 3. Sort Engine — `sort.rs`

### No New Dependencies

All sort uses `std::cmp::Ordering` and existing `Task` fields.

### Stable Sort — `sort_by`

Use `self.tasks.sort_by(|a, b| compare(a, b))`. Rust's `sort_by` is stable (mergesort-based). Tasks that compare `Equal` preserve their original order. This matches LINQ `OrderBy` behavior from C#.

### Sort Comparators

```rust
SortOrder::Priority => {
    // Some('A') < Some('B') < ... < None (None sorts last)
    match (a.priority, b.priority) {
        (None, None) => Equal,
        (None, _) => Greater,   // None sorts after all priorities
        (_, None) => Less,
        (Some(pa), Some(pb)) => pa.cmp(&pb),
    }
}

SortOrder::DueDate => {
    // Some(earlier) < Some(later) < None (no due date sorts last)
    match (a.due_date, b.due_date) {
        (None, None) => Equal,
        (None, _) => Greater,
        (_, None) => Less,
        (Some(da), Some(db)) => da.cmp(&db),
    }
}

SortOrder::Alphabetical => {
    // Case-insensitive raw string comparison
    a.to_raw().to_lowercase().cmp(&b.to_raw().to_lowercase())
}

SortOrder::Project => {
    // First +Project alphabetically; tasks with no project sort last
    let pa = a.projects.first().map(|s| s.as_str());
    let pb = b.projects.first().map(|s| s.as_str());
    match (pa, pb) {
        (None, None) => Equal,
        (None, _) => Greater,
        (_, None) => Less,
        (Some(pa), Some(pb)) => pa.to_lowercase().cmp(&pb.to_lowercase()),
    }
}

SortOrder::Context => {
    // Same pattern as Project but with @Context
    let ca = a.contexts.first().map(|s| s.as_str());
    let cb = b.contexts.first().map(|s| s.as_str());
    // ... same None-last pattern
}
```

### `#[non_exhaustive]` on `SortOrder`

Mark `SortOrder` as `#[non_exhaustive]` to allow adding `Completed`, `CreatedDate`, `None` variants in future phases without breaking downstream match statements.

---

## 4. Batch Update — `task_list.rs` Addition

### Implementation Pattern

```rust
pub fn batch_update(&mut self, replacements: Vec<(usize, Task)>) -> Result<(), TodoError> {
    let count = self.tasks.len();
    // Validate ALL indices first (fail-fast, no partial mutation)
    for &(index, _) in &replacements {
        if index >= count {
            return Err(TodoError::IndexOutOfBounds { index, count });
        }
    }
    // Apply all replacements in a single pass
    for (index, new_task) in replacements {
        self.tasks[index] = new_task;
    }
    // Single save() call
    self.save()
}
```

### No New Dependencies

This is a pure addition to the existing `task_list.rs` using the existing `save()` method and `TodoError`.

---

## 5. Portable Mode — `portable.rs`

### No New Dependencies

Uses only `std::path::{Path, PathBuf}`.

### Implementation

```rust
/// Returns the config directory to use.
///
/// If `binary_dir/config.toml` exists, returns `binary_dir` (portable mode).
/// Otherwise returns `platform_dir` (standard platform-appropriate path).
pub fn resolve_config_path(binary_dir: &Path, platform_dir: &Path) -> PathBuf {
    if binary_dir.join("config.toml").exists() {
        binary_dir.to_path_buf()
    } else {
        platform_dir.to_path_buf()
    }
}
```

**Why return a directory, not a file path?** The caller (CLI config loader) appends the filename. This keeps the function simple and the caller flexible (it could use `config.toml` or any other name).

The function is pure and testable: use `tempfile::TempDir` to create a temp directory, write a `config.toml` to it, and verify the function returns the portable path.

---

## Architecture Patterns

### Module Structure

New files to create:
```
crates/todotxt-core/src/
  filter.rs      ← Filter, FilterTerm types + matching logic
  sort.rs        ← SortOrder enum + TaskList::sort()
  watcher.rs     ← FileWatcher (cfg(feature = "watching"))
  portable.rs    ← resolve_config_path()
```

Update files:
```
crates/todotxt-core/src/
  lib.rs         ← Add pub mod + pub use for new types
  error.rs       ← Add Watch variant (cfg(feature = "watching"))
  task_list.rs   ← Add batch_update() method
Cargo.toml (workspace)  ← Add notify-debouncer-mini dep
crates/todotxt-core/Cargo.toml  ← Add [features] watching, optional dep
```

### Public API Surface (new exports)

```rust
// lib.rs additions
pub mod filter;
pub mod sort;
pub mod portable;
#[cfg(feature = "watching")]
pub mod watcher;

pub use filter::{Filter, FilterTerm};
pub use sort::SortOrder;
pub use portable::resolve_config_path;
#[cfg(feature = "watching")]
pub use watcher::FileWatcher;
```

---

## Common Pitfalls

| Pitfall | Avoidance |
|---------|-----------|
| Watching file directly (inode issue with atomic rename) | Watch parent dir, filter by filename |
| `notify::Error` is not `std::io::Error` | Add `TodoError::Watch(notify::Error)` variant |
| Forgetting `#[cfg(feature = "watching")]` guard on `Watch` variant | Feature-gate in `error.rs` |
| `new_debouncer` returning `notify::Error`, not `Result<_, TodoError>` | Use `map_err` with `TodoError::Watch` |
| Sort `None`-last vs None-first confusion | Always `(None, _) => Greater` for "no value sorts last" |
| `filter()` returning `Vec<&Task>` without indices | Return `Vec<(usize, &Task)>` for post-filter mutation support |
| `batch_update` with partial index validation | Validate ALL before ANY mutation |
| `notify-debouncer-mini` 0.7.0 re-exports `notify 8.2.0` | Don't add `notify` as separate workspace dep |

---

## Validation Architecture

### Filter Token Matrix (CORE-05)

Test all 12 filter token types from CONTEXT.md decision 1, plus `suppress_hidden` and `suppress_future_threshold`. Parametrize with `rstest`:

```rust
#[rstest]
#[case("DONE", task_completed, true)]
#[case("DONE", task_incomplete, false)]
#[case("-DONE", task_completed, false)]
#[case("-DONE", task_incomplete, true)]
#[case("due:today", task_due_today, true)]
#[case("due:today", task_due_past, false)]
#[case("due:past", task_due_past, true)]
#[case("due:past", task_due_today, false)]
#[case("due:future", task_due_future, true)]
#[case("due:future", task_due_today, false)]
#[case("due:active", task_due_today, true)]
#[case("due:active", task_due_past, true)]
#[case("due:active", task_due_future, false)]
#[case("@home", task_with_context, true)]
#[case("@home", task_without_context, false)]
#[case("-@home", task_with_context, false)]
// ... etc
fn test_filter_token(#[case] query: &str, #[case] task: Task, #[case] expected: bool) { ... }
```

### Sort Stability Tests (CORE-06)

For each sort order: create a list where multiple tasks compare equal, sort, verify original relative order preserved.

### File Watcher Integration Test (CORE-04)

Use `tempfile::NamedTempFile`, write to it, verify callback fires within 2 seconds:
```rust
#[test]
fn test_watcher_fires_on_write() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let flag = Arc::new(AtomicBool::new(false));
    let flag2 = flag.clone();
    
    let _watcher = FileWatcher::new(
        tmp.path(),
        Arc::new(move || { flag2.store(true, Ordering::SeqCst); })
    ).unwrap();
    
    // Write to file
    std::fs::write(tmp.path(), "new content").unwrap();
    
    // Wait up to 2s for debounce
    for _ in 0..20 {
        if flag.load(Ordering::SeqCst) { break; }
        std::thread::sleep(Duration::from_millis(100));
    }
    
    assert!(flag.load(Ordering::SeqCst), "watcher callback did not fire within 2s");
}
```

Note: watcher integration tests are inherently timing-sensitive. Mark as `#[ignore]` in CI if flaky, or use a longer timeout.

### Portable Mode Tests (CORE-08)

Two cases: config file present beside binary → portable path; config file absent → platform path.

### Batch Update Tests

- Valid batch → all replacements applied, single save
- Partial out-of-bounds → `IndexOutOfBounds` error, NO tasks mutated
- Empty batch → no-op, no save called (verify with mock or read file unchanged)

---

## ## RESEARCH COMPLETE

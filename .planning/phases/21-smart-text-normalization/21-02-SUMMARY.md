# Plan 21-02 Summary: TUI Config Toggles + Append Flow Wiring

**Status:** ✅ COMPLETE

**Commit:** ed22611

**Changes:** 8 files, 621 insertions, 7 deletions

## Implementation Details

### Task 1: Config Fields (COMPLETE)
**Location:** `crates/todotxt-tui/src/config.rs`

Added smart text normalization toggles to `TuiConfig`:
- Added module-level `fn default_true() -> bool { true }` helper
  - Required because `#[serde(default)]` alone defaults bools to false
  - Enables explicit "default to true" for normalization toggles
- Added `pub normalize_append: bool` field with `#[serde(default = "default_true")]`
  - Controls whether appended text tokens are parsed and merged (D-07)
  - Documentation references 21-CONTEXT.md
- Added `pub normalize_edit: bool` field with `#[serde(default = "default_true")]`
  - Controls whether inline priority tokens are lifted during edit save (D-06)
  - Documentation references 21-CONTEXT.md

Added 3 unit tests in `#[cfg(test)]` module:
- `deserialize_normalize_flags_false()`: Verifies false values parse correctly from TOML
- `deserialize_normalize_flags_true()`: Verifies true values parse correctly from TOML
- `deserialize_normalize_flags_default()`: Verifies default to true when fields omitted
- All 3 tests passing

### Task 2: Append Flow Wiring (COMPLETE)
**Location:** `crates/todotxt-tui/src/app.rs`

Added `normalize_append` to todotxt_core imports:
```rust
use todotxt_core::{Filter, SortOrder, Task, TaskList, normalize_append};
```

Updated `handle_append_text_key` Enter key handler (lines ~968-985):
- Replaced single-strategy raw concat with branching logic:
  ```rust
  let new_task = if self.config.normalize_append {
      // Smart parse-then-merge strategy (default, D-07, D-08)
      normalize_append(t, &text)
  } else {
      // Phase 20 fallback: raw concat + parse
      let new_raw = format!("{} {}", t.to_raw().trim_end(), &text);
      Task::parse(&new_raw)
  };
  ```
- When toggle ON (default): Calls `todotxt_core::normalize_append()`
  - Priority tokens merged into priority field (append wins)
  - Projects/contexts deduplicated via BTreeSet
  - Due dates, threshold dates merged with priority
  - Unknown tokens preserved in body (NORM-05)
  - Result rebuilt canonically via `rebuild_raw()` + re-parse
- When toggle OFF: Preserves Phase 20 behavior
  - Raw string concat with space
  - Parse once, no normalization
  - Provides migration path for users preferring old behavior

## Testing Results

**Unit Tests (config.rs):** 3 passing
- All deserialization scenarios covered

**Integration/Workspace Tests:** 380 passing, 0 failures
- No regressions from Plan 21-01 or prior phases
- All todotxt-core normalize tests still passing (16+ tests)
- All TUI tests passing

## Design Rationale

### Toggle Default Values (T-21-04 mitigation)
- Both toggles default to `true` to enable smart normalization by default
- Users can opt-out per toggle in config.toml
- Backward compatibility: old configs without these fields work unchanged
- Serde default behavior: `#[serde(default)]` → false, so explicit `default_true()` needed

### Branching Strategy (D-08, D-09 design)
- Config check in hot path: single bool branch per append operation (negligible overhead)
- Append workflow unchanged: Phase 20 fallback path identical to previous code
- Future-proof: Plan 21-03 will wire `normalize_edit` same way in edit_save flow

## Dependency Chain

**Unblocks:** Plan 21-03 (TUI Edit Flow + CLI Flag)
- Plan 21-03 will use the `normalize_edit` field added here
- CLI flag will respect both TUI config + CLI override

**Depends On:** Plan 21-01 (Core Normalization Helpers) ✅
- Uses `todotxt_core::normalize_append()` from 21-01

## Verification

- ✅ Config fields deserialize correctly (3 unit tests)
- ✅ handle_append_text_key branches on toggle correctly
- ✅ normalize_append() imported successfully
- ✅ Build succeeds without errors
- ✅ 380 workspace tests pass with zero regressions
- ✅ Commit contains exactly the intended changes

## Commit Message
```
Phase 21-02: TUI config toggles + append flow wiring

- Add default_true() helper function to config.rs for serde defaults
- Add normalize_append: bool and normalize_edit: bool config fields to TuiConfig
  - Both default to true via #[serde(default = "default_true")]
  - normalize_append controls smart text normalization on append (D-07)
  - normalize_edit controls smart text normalization on edit (D-06)
- Wire handle_append_text_key to branch on self.config.normalize_append
  - When true (default): calls todotxt_core::normalize_append() for smart parsing
  - When false: falls back to Phase 20 raw concat + Task::parse()
- Add 3 unit tests verifying config deserialization (false, true, default)
- All 380 workspace tests pass with zero regressions

See 21-CONTEXT.md D-06, D-07, D-08, D-09 for design background.
```

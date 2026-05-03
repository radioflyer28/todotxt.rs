# Phase 14: Deferred Task (t:) Implementation Spec
Status: LOCKED — V12-TUI-DEFER-01 resolved
Decision: implement full t: filtering parity (D-04 through D-08)
Governs: Phase 15 (CLI) and Phase 17 (TUI)

---

## Section 1: Current State

**The core filtering is already implemented — no changes needed to `todotxt-core`.**

- **File:** `crates/todotxt-core/src/filter.rs`
- **Line 35:** `suppress_future_threshold: true` is the **default** on the `Filter` struct
- Tasks with a future `t:YYYY-MM-DD` are **already hidden** from `list` output at the core library level
- The `Filter` struct default:
  ```rust
  impl Default for Filter {
      fn default() -> Self {
          Filter {
              terms: Vec::new(),
              suppress_hidden: true,
              suppress_future_threshold: true,  // ← ALREADY IMPLEMENTED
          }
      }
  }
  ```

**Edge cases (confirmed correct — no changes needed):**
- `t:PAST` (past date) → **shown** ✅ — check is `t > today`, so past dates pass through
- `t:TODAY` (today's date) → **shown** ✅ — check is `t > today`, so today equals threshold → passes through (not deferred)
- `t:FUTURE` → **hidden** ✅ — filtered by `suppress_future_threshold: true`

**What is still missing for full D-04–D-08 parity:** The CLI `--all` flag and the TUI toggle. Both are documented below.

---

## Section 2: Phase 15 Changes Required (CLI)

Phase 15 must add exactly two code changes. No core library changes are needed.

### Change 1: Add `--all` flag to `ListArgs` in `crates/todotxt-cli/src/cli.rs`

Find the `ListArgs` struct (currently around line 147) and add the following field:

```rust
/// Include deferred tasks (those with a future t: threshold date).
/// Also includes hidden tasks (h:1). Shows all tasks regardless of filter suppression.
#[arg(long)]
pub all: bool,
```

### Change 2: Wire `--all` in `crates/todotxt-cli/src/commands/list.rs`

In the `build_filter` function (or wherever the `Filter` struct is constructed from `ListArgs`), add after the filter is built:

```rust
if args.all {
    filter.suppress_future_threshold = false;
    filter.suppress_hidden = false;
}
```

**Rationale for including `suppress_hidden = false`:** `--all` semantically means "show everything" — both deferred tasks and hidden tasks. Consistent with the `--all` convention in other CLI tools.

### Testing requirement for Phase 15

Phase 15 must add tests for this behavior:

| Test case | Expected result |
|---|---|
| `list` with a task that has a future `t:` date | Task NOT shown |
| `list --all` with a task that has a future `t:` date | Task shown |
| `list` with `t:TODAY` (today's date) | Task shown (not deferred) |
| `list` with `t:PAST` (past date) | Task shown |
| `list --all` with `h:1` task | Task shown |

---

## Section 3: Phase 17 Changes Required (TUI)

Phase 17 must add a TUI toggle for deferred task visibility plus greyed-out rendering for deferred tasks when shown.

### App State Change (`crates/todotxt-tui/src/app.rs` or equivalent)

Add a field to the `App` struct:

```rust
/// When true, tasks with a future t: threshold date are shown (greyed out).
/// When false (default), deferred tasks are hidden from the list.
pub show_deferred: bool,
```

Default: `false` (deferred tasks hidden on startup — matches current behavior).

### Key Toggle

Add a key event handler for toggling deferred task visibility. The Phase 17 planner should pick a key that doesn't conflict with existing bindings (suggestion: `D` uppercase, since lowercase `d` may be used for delete or similar). The toggle:

```rust
// on key press for chosen key:
self.app.show_deferred = !self.app.show_deferred;
// trigger a list refresh to rebuild the filter
```

### Filter Wiring

When building the `Filter` for the TUI list, apply the `show_deferred` flag:

```rust
let mut filter = Filter::default(); // suppress_future_threshold: true by default
if self.app.show_deferred {
    filter.suppress_future_threshold = false;
}
// ... apply other filters (search terms, etc.)
```

### Greyed-Out Styling for Deferred Tasks

When `show_deferred` is `true` and a task has a future `t:` date, render it with a dim/grey style using the existing Phase 13 `StyleSheet` system (`crates/todotxt-tui/src/theme.rs` or equivalent).

- Use `Modifier::DIM` or the light-grey color from the theme's inactive/low-priority slot
- Greyed styling applies **only when deferred tasks are visible** — when hidden they don't appear at all
- Do NOT apply greyed styling to regular tasks or tasks with past/today `t:` dates

Implementation pattern (using existing Phase 13 owo-colors style system):
```rust
// In the task rendering loop:
let style = if show_deferred && task.threshold_date.map(|t| t > today).unwrap_or(false) {
    theme.deferred_style()  // dim / light-grey
} else {
    // normal priority-based style
    theme.task_style(&task)
};
```

### Status Bar Indicator

When `show_deferred` is `true`, the TUI status bar should show a visual indicator. Suggested format:
```
[+deferred]
```
Reuse the existing status bar rendering pattern from Phase 13. The indicator should be placed near other filter status indicators (if any).

### Files Phase 17 Will Modify

| File | Change |
|---|---|
| `crates/todotxt-tui/src/app.rs` (or equivalent) | Add `show_deferred: bool` field to `App` struct |
| TUI key handler file | Add toggle key event |
| TUI renderer/list file | Add deferred styling branch in task render loop |
| TUI status bar file | Add `[+deferred]` indicator when `show_deferred` is true |

---

## Section 4: Decision Log

| Decision ID | Status | Description |
|---|---|---|
| V12-TUI-DEFER-01 | **CONFIRMED** — implement full t: filtering parity | Deferred-task filtering is confirmed in scope for v1.2 |
| V12-TUI-DEFER-02 | **IN SCOPE** — due in Phase 15 (CLI) and Phase 17 (TUI) | Both CLI and TUI surfaces must expose the show/hide control |
| D-04 | LOCKED | Tasks with future `t:` are hidden from `list` by default |
| D-05 | LOCKED | Toggle: `list --all` (CLI) + TUI key toggle |
| D-06 | LOCKED | Deferred tasks shown with greyed-out color when visible |
| D-07 | LOCKED | Hiding scope is `list` only — `do`, `del`, `edit`, `append`, `pri` are unaffected |
| D-08 | LOCKED | `t:` filtering parity confirmed; implementation in Phases 15/17 |

**Hiding scope clarification (D-07):** The `suppress_future_threshold` filter only applies when building the display list. All mutating commands (`do`, `del`, `edit`, `append`, `pri`) operate on tasks by line number — they are never affected by `suppress_future_threshold`. A user can always address a deferred task by its line number even when it's hidden from the default `list` view.

---

*Phase: 14-compat-discovery*
*Spec locked: 2026-04-23*

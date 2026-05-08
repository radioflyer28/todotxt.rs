---
phase: 260507-nh1-fix-ctrl-left-right-task-movement-betwee
reviewed: 2026-05-07T00:00:00Z
depth: quick
files_reviewed: 1
files_reviewed_list:
  - crates/todotxt-tui/src/app.rs
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 260507-nh1: Code Review Report

**Reviewed:** 2026-05-07  
**Depth:** quick (targeted at `extract_tag_tokens` and `pane_move_task`)  
**Files Reviewed:** 1  
**Status:** issues_found

## Summary

Reviewed the new `extract_tag_tokens` helper (line 295) and the updated `pane_move_task` function (line 320) in `crates/todotxt-tui/src/app.rs`. No security or data-loss–level issues were found. Three warnings were identified: two logic defects in the tag extractor (dead negation guards and an unguarded bare-token case) and one error-handling defect in the bulk-mutation loop. Two test coverage gaps are noted as info.

The filtered→filtered path (PMOVE-02 existing test) is unaffected by the new code. The four new NH1-FIX tests adequately cover the primary happy paths.

---

## Warnings

### WR-01: Dead negation guards in `extract_tag_tokens`

**File:** `crates/todotxt-tui/src/app.rs:299`  
**Issue:** The two negation checks `!t.starts_with("-@")` and `!t.starts_with("-+")` are unreachable dead code. The preceding predicate `t.starts_with('@') || t.starts_with('+')` already excludes any token that begins with `-`, so those guards can never fire. The doc comment ("negated `-@foo`, `-+bar`, slash-forms are ignored") implies they do the filtering work, but they don't — which is misleading and could cause a silent regression if a future filter syntax change adds negation with a different prefix.

```rust
// Current — negation guards are unreachable:
.filter(|t| {
    (t.starts_with('@') || t.starts_with('+'))
        && !t.starts_with("-@")   // dead: already excluded by the first predicate
        && !t.starts_with("-+")   // dead: already excluded by the first predicate
        && !t.contains('/')
})

// Fix — remove the dead guards and document the actual behaviour:
.filter(|t| {
    // Only plain @context / +project tokens. Negated forms (-@foo, -+bar)
    // are already excluded because they don't start with @ or +.
    // Slash-composite forms (@context/sub) are excluded explicitly.
    (t.starts_with('@') || t.starts_with('+'))
        && !t.contains('/')
})
```

---

### WR-02: Bare `@` or `+` token not guarded — malformed tag injection

**File:** `crates/todotxt-tui/src/app.rs:297`  
**Issue:** A filter query containing a lone `@` or `+` (e.g. `@ +project`, possibly from a partially typed filter saved to config) passes all predicates and is returned as a "tag token". When `pane_move_task` then appends it to task raw text, the task gains a bare `@` which is malformed per the todo.txt spec and will render oddly in any compliant client. The guard `t.len() > 1` prevents this:

```rust
// Fix:
.filter(|t| {
    t.len() > 1
        && (t.starts_with('@') || t.starts_with('+'))
        && !t.contains('/')
})
```

---

### WR-03: Partial disk mutation on `update()` failure — `Ok(())` returned to caller

**File:** `crates/todotxt-tui/src/app.rs:382`  
**Issue:** The bulk-mutation loop calls `task_list.update(task_idx, new_task)` for each selected task. `update()` writes to disk immediately. If it fails at task N, tasks 0..N-1 have already been mutated and saved, but the function returns `Ok(())` — not an `Err` — so the call site (lines 1405/1409) does not propagate a failure. The user sees a runtime warning but the TUI considers the operation successful and continues. The undo entry covers recovery, but the partial-mutation state is invisible to the caller.

Suggested fix: return the error rather than swallowing it with `Ok(())`:

```rust
if let Err(e) = self.task_list.update(task_idx, new_task) {
    self.push_runtime_warning(format!("pane_move_task: update failed: {e}"));
    // Return Err so the caller (handle_normal_key) can surface this properly:
    return Err(color_eyre::eyre::eyre!("pane_move_task: update failed: {e}"));
}
```

---

## Info

### IN-01: No test for dest tag already present on task

**File:** `crates/todotxt-tui/src/app.rs:6926` (test module)  
**Issue:** The `already_present` deduplication check in `pane_move_task` (line 373) is correct, but no test exercises it. Moving a task that already carries the destination tag (e.g. `todo @work @home` from the `@work` pane to the `@home` pane) should remove `@work` and leave `@home` exactly once. A regression here would silently produce duplicate tags.

Suggested test name: `pane_move_task_dest_tag_already_present`

```rust
#[test]
fn pane_move_task_dest_tag_already_present() {
    // task already has @home; after move @work is removed, @home appears exactly once
    let mut app = make_app_with_config(
        &["todo @work @home task"],
        two_pane_config("@work", "@home"),
    );
    app.pane_move_task(1).unwrap();
    let raw = app.task_list.tasks()[0].to_raw().to_string();
    assert!(!raw.contains("@work"), "@work must be removed: {}", raw);
    let home_count = raw.split_whitespace().filter(|&t| t == "@home").count();
    assert_eq!(home_count, 1, "@home must appear exactly once, got: {}", raw);
}
```

---

### IN-02: No test for src ⊂ dest tag overlap (partial filter intersection)

**File:** `crates/todotxt-tui/src/app.rs:6926` (test module)  
**Issue:** When the source filter's tags are a strict subset of the destination filter's tags (e.g. src=`@work`, dest=`@work @home`), the current logic removes `@work` from the task, then sees `already_present = false` (because `@work` was just removed), and adds it back along with `@home`. The net result is correct (`@home @work` preserved), but this scenario is untested and an `already_present` check that runs against the *original* raw (before src removal) would yield the wrong answer — confirming the current order of operations matters.

Suggested test name: `pane_move_task_src_subset_of_dest_tags`

---

_Reviewed: 2026-05-07_  
_Reviewer: gsd-code-reviewer_  
_Depth: quick (targeted)_

# Quick Task 260508-l56: Remove Line Numbers from Task List Display — Research

**Researched:** 2026-05-08
**Confidence:** HIGH

---

## 1. Exact Format String Location

**File:** `crates/todotxt-tui/src/app.rs`, **line 3688**, inside `fn render_task_list`.

```rust
// Current (line 3688):
let content = format!("{}{}{}: {}", prefix, indent, ci + 1, t.to_raw());

// Target:
let content = format!("{}{}{}", prefix, indent, t.to_raw());
```

`ci` is the raw task-list index (0-based); `ci + 1` is the display number.  
Both `prefix` and `indent` are kept as-is.

---

## 2. pane_list.rs — No Change Needed

`crates/todotxt-tui/src/components/pane_list.rs` line 144:

```rust
let prefixed = format!("{}{}", prefix, full_content);
```

No line number in the pane secondary list. **Confirmed — zero changes required there.**

---

## 3. Test Assertions That Need Updating

**Result: none.** No test in the codebase asserts on rendered `ListItem` content
containing the `N: ` pattern. Verified by:

- Grepping all `*.rs` under `crates/todotxt-tui/` for `"[0-9]+: .*"` patterns in assert
  statements — zero matches.
- `fallback_test.rs` `test_display_rows_fallback` / `test_display_rows_multi_pane`:
  only assert `rows.len()`, not content.
- `render_task_list` is a pure rendering function; it is called at lines 3751 and 3756
  inside the draw pipeline and has no unit tests asserting its output content.

**No test changes required.**

---

## 4. Other Callers / Snapshot Tests

`render_task_list` has exactly two call sites — both inside the frame draw pipeline
(lines 3751 and 3756), both rendering-only.  
No snapshot test infrastructure (e.g., `insta`, `expect-test`) is present in the codebase.  
No external integration tests assert on visible row text.

**No other files need changes.**

---

## 5. Is the Change Purely Cosmetic?

Yes. `ci` is used in the format string only. Task identity throughout the app
is tracked by index (`display_rows`, `display_indices`, `selected_tasks`, `selected`)
— none of those read the rendered string back. No hotkey, jump-to-line, filter,
search, clipboard, or undo feature uses the `N: ` prefix.

---

## Summary

| Item | Finding |
|------|---------|
| Change location | `app.rs:3688` — one line |
| Change type | Remove `ci + 1, ` and trailing `": "` from format string |
| pane_list.rs | Already clean, no change |
| Tests to update | **0** |
| Other files | **0** |
| Risk | Cosmetic-only, no logic dependency |

# Quick Task 260507-nh1: Fix ctrl-left/right task movement — Research

**Researched:** 2026-05-07
**Domain:** Rust TUI / pane task movement logic
**Confidence:** HIGH (all findings from direct codebase inspection)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Apply ALL context/project tokens from the destination pane's filter to the task
- If destination pane filter has multiple context/project tokens, add ALL of them
- When moving FROM a filtered pane TO an unfiltered pane, REMOVE the source pane's context/project tokens from the task

### Agent's Discretion
- Panes with non-context/project filter tokens (e.g. priority filters, `due:today`) — ignore for tag mutation; only `@context` and `+project` tokens participate in add/remove logic

### All four movement combinations must work
1. Filtered → Filtered: remove src tags, add dest tags (existing behavior)
2. Filtered → Unfiltered: remove source filter's @context/+project tags
3. Unfiltered → Filtered: add dest filter's @context/+project tags
4. Unfiltered → Unfiltered: no tag changes (task moves as-is)
</user_constraints>

---

## Finding 1: Key Dispatch — Where Ctrl+Left/Right is wired

**File:** `crates/todotxt-tui/src/config.rs` lines 498–499
```rust
m.insert("pane_move_left".into(),  (KeyCode::Left,  KeyModifiers::CONTROL));
m.insert("pane_move_right".into(), (KeyCode::Right, KeyModifiers::CONTROL));
```

**File:** `crates/todotxt-tui/src/app.rs` lines 1411–1418
```rust
// Ctrl+Left/Right moves task to adjacent pane (Phase 41, PMOVE-02).
_ if self.key_is_action(key, "pane_move_left") => {
    self.pane_move_task(-1)?;
}
_ if self.key_is_action(key, "pane_move_right") => {
    self.pane_move_task(1)?;
}
```

**Note:** An earlier bug (BUG-41-01, fixed in Phase 44) had an unguarded `KeyCode::Right =>` arm that intercepted Ctrl+Right before the action guard. The fix is confirmed working — `_ if self.key_is_action(...)` pattern guards correctly.

---

## Finding 2: Core Implementation — `pane_move_task` and the Bug

**File:** `crates/todotxt-tui/src/app.rs` lines 292–410

### `is_single_tag_token` helper (lines 292–302)
```rust
fn is_single_tag_token(filter: &str) -> bool {
    if filter.is_empty() {
        return false;  // ← EMPTY STRINGS REJECTED HERE
    }
    let trimmed = filter.trim();
    (trimmed.starts_with('@') || trimmed.starts_with('+'))
        && !trimmed.contains(char::is_whitespace)
}
```

### `pane_move_task` validation block (lines 328–342) — **the bug location**
```rust
let src_filter = self.panes[src_idx].filter_query.trim().to_string();
let dest_filter = self.panes[dest_idx].filter_query.trim().to_string();

if !Self::is_single_tag_token(&src_filter) {
    self.push_runtime_warning(format!(
        "Cannot move: source pane filter '{}' is not a single @/+ tag.",
        src_filter
    ));
    return Ok(());  // ← EARLY EXIT when src filter is empty or multi-token
}
if !Self::is_single_tag_token(&dest_filter) {
    self.push_runtime_warning(format!(
        "Cannot move: destination pane filter '{}' is not a single @/+ tag.",
        dest_filter
    ));
    return Ok(());  // ← EARLY EXIT when dest filter is empty or multi-token
}
```

**Root cause:** Both guards reject empty strings (unfiltered panes) AND multi-token filters. Any pane with `filter_query = ""` causes the move to abort with a "not a single @/+ tag" warning.

### Current mutation block (lines 367–397) — handles only single-token src/dest
```rust
// Remove source filter token (word-by-word, case-sensitive exact match).
let filtered_tokens: Vec<&str> = raw
    .split_whitespace()
    .filter(|&t| t != src_filter)
    .collect();
let mut new_raw = filtered_tokens.join(" ");

// Append dest filter token if not already present.
let already_has_dest = new_raw
    .split_whitespace()
    .any(|t| t == dest_filter);
if !already_has_dest {
    if !new_raw.is_empty() { new_raw.push(' '); }
    new_raw.push_str(&dest_filter);
}
```
This works for single-token cases. Must be generalized to handle `Vec<String>` of tokens.

---

## Finding 3: Pane Filter Data Structure

**File:** `crates/todotxt-tui/src/state.rs` lines 43–45
```rust
/// Query filter state specific to this pane
pub filter_query: String,
```

- `filter_query` is a plain `String` — the raw query text typed by the user (e.g. `"@work"`, `"@work +project"`, `""` for no filter)
- Applied via `todotxt_core::Filter::from_query(&filter_query)` during rebuild

**Extracting @context/+project tokens from a filter query** (derived from `filter.rs`):

`Filter::from_query()` tokenizes by `split_ascii_whitespace()`. Context/project tokens are:
- `@foo` (no `/`) → `FilterTerm::ContextPrefix("foo")` — raw token `@foo`
- `+bar` (no `/`) → `FilterTerm::ProjectPrefix("bar")` — raw token `+bar`
- `@foo/bar` with slash → falls through to `FilterTerm::Include` (exact match, NOT a tag token)
- `-@foo`, `-+bar` → negated forms — should be **ignored** for tag mutation (per agent's discretion)
- `due:today`, `DONE`, priority filters → non-tag tokens — **ignored** for tag mutation

**Helper to implement** — extract tag tokens as raw strings with sigil:
```rust
fn extract_tag_tokens(filter_query: &str) -> Vec<String> {
    filter_query
        .split_whitespace()
        .filter(|t| {
            (t.starts_with('@') || t.starts_with('+'))
                && !t.starts_with("-@")
                && !t.starts_with("-+")
                && !t.contains('/')
        })
        .map(|t| t.to_string())
        .collect()
}
```
- Empty filter → returns `[]` (unfiltered pane)
- `"@work"` → `["@work"]`
- `"@work +project"` → `["@work", "+project"]`
- `"@work due:today"` → `["@work"]` (non-tag terms ignored)

---

## Finding 4: Task Tag Manipulation

**File:** `crates/todotxt-core/src/task.rs`

`Task` fields:
- `contexts: Vec<String>` — context names WITHOUT `@` sigil, sorted, deduplicated
- `projects: Vec<String>` — project names WITHOUT `+` sigil, sorted, deduplicated
- `raw: String` — canonical serialization form (private)

**No dedicated `add_tag`/`remove_tag` methods exist.** The existing mutation pattern in `pane_move_task` is the correct approach: operate on the raw string directly via `split_whitespace()` / filter / join, then call `Task::parse(&new_raw)`.

Builder methods exist for other fields (`with_priority`, `with_due_date`, `with_completed`, etc.) but none for context/project tags. The raw-string manipulation approach is already used and is correct.

**Mutation pattern to generalize** (handles multiple src/dest tokens):
```rust
// Remove all src tag tokens from raw string
let src_tags: Vec<String> = /* extract_tag_tokens(src_filter) */;
let dest_tags: Vec<String> = /* extract_tag_tokens(dest_filter) */;

let mut new_raw: String = raw
    .split_whitespace()
    .filter(|&t| !src_tags.iter().any(|s| s == t))
    .collect::<Vec<_>>()
    .join(" ");

// Append each dest tag token if not already present
for dest_tag in &dest_tags {
    let already_present = new_raw.split_whitespace().any(|t| t == dest_tag);
    if !already_present {
        if !new_raw.is_empty() { new_raw.push(' '); }
        new_raw.push_str(dest_tag);
    }
}
```

---

## Finding 5: Existing Tests for Pane Movement

All tests in `crates/todotxt-tui/src/app.rs`:

| Test name | Line | What it covers |
|-----------|------|----------------|
| `pane_move_task_tag_swap` | 6953 | Filtered→Filtered: removes src, adds dest |
| `pane_move_task_declined_compound_filter` | 6972 | Old guard: compound filter is **declined** (will need updating) |
| `pane_move_task_wraps_at_boundary` | 6989 | Wrap at pane 0 boundary |
| `pane_move_task_direct_moves_right` | 7110 | Direct method call, removes src + adds dest |
| `pane_move_task_pushes_undo_entry` | 7128 | Undo entry pushed before mutation |

**Tests that will break or need updating after the fix:**
- `pane_move_task_declined_compound_filter` — currently asserts that compound filter is **declined**. After the fix, multi-token src filters should be **accepted** (all tag tokens removed). This test needs rewriting to reflect new behavior, OR the test name/assertion should be updated.

**New tests needed:**
1. Filtered → Unfiltered: task loses src tag
2. Unfiltered → Filtered: task gains dest tag(s)
3. Unfiltered → Unfiltered: task moves unchanged
4. Multi-token filter: all tokens applied
5. Non-tag tokens in filter (e.g. `@work due:today`): only `@work` mutated, `due:today` ignored

---

## Fix Summary

### What to change in `pane_move_task` (`app.rs` lines 316–410)

**Step 1:** Add `extract_tag_tokens` helper (private `fn`, same impl block or just before `is_single_tag_token`).

**Step 2:** Replace the two `is_single_tag_token` guard blocks (lines 328–342) with:
```rust
let src_tags = Self::extract_tag_tokens(&src_filter);
let dest_tags = Self::extract_tag_tokens(&dest_filter);
// Both empty = unfiltered→unfiltered = move allowed, no tag changes
```
No early return needed for empty filters. The pane count check (< 2) and "no task selected" check remain.

**Step 3:** Replace the mutation block (lines 367–397) with the generalized multi-token version above.

**Step 4:** Update test `pane_move_task_declined_compound_filter` — compound filter is now **accepted**; assert that all tag tokens are applied. Add new tests for the four movement combinations.

### `is_single_tag_token` fate
After the fix, `is_single_tag_token` is no longer needed by `pane_move_task`. If no other callers exist, it can be removed. (Verify with grep — no other call sites found in this search.)

---

## Sources

- `crates/todotxt-tui/src/app.rs` — direct codebase inspection (lines 292–410, 1411–1418, 6951–7145)
- `crates/todotxt-tui/src/state.rs` — `Pane` struct fields (lines 30–100)
- `crates/todotxt-core/src/filter.rs` — `Filter::from_query` tokenization (lines 1–200)
- `crates/todotxt-core/src/task.rs` — `Task` struct and mutation patterns (lines 1–300)
- `crates/todotxt-tui/src/config.rs` — default keymap (lines 496–499)

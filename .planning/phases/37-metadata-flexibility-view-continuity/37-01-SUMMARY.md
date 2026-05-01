---
phase: 37
plan: 01
type: tdd
date_completed: 2026-04-30
---

# Plan 37-01: Hierarchical Tag Filtering (META-02) — COMPLETE

## Objective
Implement hierarchical tag filtering by adding `ContextPrefix` and `ProjectPrefix` FilterTerm variants to enable queries like `@email` to match both `@email` and `@email/waiting` contexts while preserving exact matching via Include terms for slash-delimited tokens.

## What Was Built

### 1. FilterTerm Enum Extensions
Added four new variants to `FilterTerm` enum in `crates/todotxt-core/src/filter.rs`:
- `ContextPrefix(String)` — Matches contexts that equal the prefix OR start with `{prefix}/`
- `ProjectPrefix(String)` — Matches projects following same semantics
- `NegContextPrefix(String)` — Negated context prefix matching
- `NegProjectPrefix(String)` — Negated project prefix matching

### 2. Parser Updates (Filter::from_query)
Enhanced parser logic to detect and parse prefix patterns:
- `@foo` (no slash) → `ContextPrefix("foo")`
- `@foo/bar` (with slash) → `Include("@foo/bar")` for exact matching
- `+client` (no slash) → `ProjectPrefix("client")`
- `+client/acme` (with slash) → `Include("+client/acme")` for exact matching
- `-@foo` (negated, no slash) → `NegContextPrefix("foo")`
- `-+client` (negated, no slash) → `NegProjectPrefix("client")`

### 3. Matching Logic (matches_with_date)
Implemented prefix matching with case-insensitive comparison:
- **ContextPrefix**: Check if task.contexts contains exact match (case-insensitive) OR any context starting with `{prefix}/` (case-insensitive)
- **ProjectPrefix**: Same logic applied to task.projects
- **NegContextPrefix/NegProjectPrefix**: Negated versions of above

### 4. Test Coverage
Added 16 comprehensive tests covering:
- **Parsing tests**: `@foo`, `@foo/bar`, `+client`, `+client/acme`, negated forms
- **Matching tests**: 
  - Exact context/project matching
  - Hierarchical matching (`@email/waiting` matches `@email`)
  - Non-matching cases
  - Case-insensitivity
  - Negated prefix matching
  - Exact slash-delimited matching
  - Mixed prefix + exact queries

All tests implement TDD cycle: RED (tests written first) → GREEN (implementation) → REFACTOR (cleanup).

## Key Files Modified
- `crates/todotxt-core/src/filter.rs`
  - Lines 1-19: Added 4 new FilterTerm variants
  - Lines 57-119: Updated Filter::from_query parser (63 lines of new logic)
  - Lines 138-195: Updated matches_with_date with 4 new match arms
  - Lines 385-490: Added 16 new tests

## Test Results
✅ All 38 todotxt-core tests PASS (0 failures, no regressions)
✅ 34 filter-specific tests PASS
✅ Complete coverage of hierarchical prefix scenarios

## Implementation Notes

### Decisions Made
- **D-06 Negation**: Implemented explicit `NegContextPrefix`/`NegProjectPrefix` variants (simpler than Exclude wrapper logic)
- **D-03 Parsing Rule**: Tokens with `/` become Include (exact match); tokens without `/` become Prefix (hierarchical match)
- **D-04/D-05 Case-Insensitivity**: Both prefix matching and exact matching use `.to_ascii_lowercase()` for comparison

### Design Validation
- Parent-prefix matching correctly distinguishes `@email` vs `@email/waiting` via starts_with check on format string
- Slash-delimited tokens (`@email/waiting`) are treated as exact matches via Include term (substring match on raw)
- Negated forms work correctly by inverting the prefix match boolean

## Integration Points
- No CLI or TUI changes needed — both benefit from filter layer enhancement automatically
- Backward compatible: existing Include/Exclude terms work unchanged
- Parsed FilterTerm enums are consumed by App/TUI pane filter logic unchanged

## Remaining Work
Plan 37-02 (VIEW-03) validates that pane filter/sort/group state remains preserved across all v1.5 flows with these new prefix filters active.

## Self-Check
- ✅ All RED tests written before implementation (TDD cycle followed)
- ✅ All GREEN tests passing
- ✅ REFACTOR phase: code is clean, no dead code or duplication
- ✅ No regressions in other modules (38 tests pass)
- ✅ Implementation matches design decisions D-01 through D-05
- ✅ Must-haves achieved: parent-prefix matching, exact slash-matching, negation support

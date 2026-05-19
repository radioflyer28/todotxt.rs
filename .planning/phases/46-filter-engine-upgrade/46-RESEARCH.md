# Phase 46 Research: Filter Engine Upgrade

**Phase:** 46-filter-engine-upgrade
**Date:** 2026-05-15
**Status:** Complete

## Summary

Phase 46 is best implemented in `todotxt-core/src/filter.rs`, because both CLI and TUI
already route query interpretation through `Filter::from_query`. The TUI call sites in
`crates/todotxt-tui/src/app.rs` should not need grammar-specific changes: once the core
filter understands token-local OR, every pane rebuild and active-pane filter path inherits
the behavior.

## Existing Shape

- `FilterTerm` is a simple enum of predicates.
- `Filter::from_query` splits on ASCII whitespace and maps each token to one
  `FilterTerm`.
- `matches_with_date` pre-filters hidden and future-threshold tasks, then AND-evaluates
  every parsed term.
- Context/project prefix matching is already represented by dedicated positive and
  negative variants.
- CLI list composition builds an effective query string and passes it to
  `Filter::from_query`.
- TUI pane filtering calls `Filter::from_query` in two places, so core behavior is reused.

## Recommended Implementation

Add `FilterTerm::Or(Vec<FilterTerm>)` and extract:

- `parse_single_token(token: &str) -> FilterTerm`
- `parse_token(token: &str) -> Option<FilterTerm>`
- `eval_term(term: &FilterTerm, task: &Task, raw: &str, today: NaiveDate) -> bool`

`parse_token` handles token-local `|` by splitting the token, trimming empty branches, and
mapping each non-empty branch through `parse_single_token`.

Recommended normalization:

- zero non-empty branches: drop the term
- one non-empty branch: return that branch directly
- two or more non-empty branches: return `FilterTerm::Or(branches)`

This implements the user decision to tolerate empty branches without creating a broad
"match everything" behavior.

## Important Decisions To Preserve

- OR support is token-local only.
- `-(@work|@home)` and richer grouped grammar are out of scope.
- A leading `-` should not be treated as grouped OR negation.
- Empty branches like `@work|`, `|@home`, and `(A)|` are ignored.
- Documentation should state the supported contract explicitly.

## Testing Targets

Core tests should cover:

- priority OR: `(A)|(B)`
- context OR: `@work|@home`
- project OR: `+work|+home`
- OR plus existing AND: `(A)|(B) @work`
- empty-branch tolerance: `(A)|`, `|@home`, `@work|`
- existing AND behavior unchanged: `@work +proj`
- unsupported grouped negation does not become magic: `-(@work|@home)`

CLI integration tests should prove that positional filters and `--filter` can use OR,
because users encounter the feature through `todotxt list`.


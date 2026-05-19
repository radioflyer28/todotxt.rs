# Phase 46: Filter Engine Upgrade - Context

**Gathered:** 2026-05-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 46 delivers OR-capable filter terms inside the existing whitespace-delimited filter
language. The scope is limited to token-local OR semantics such as `@work|@home`,
`+proj1|+proj2`, and `(A)|(B)`, while preserving existing space-separated AND behavior
 everywhere else.

</domain>

<decisions>
## Implementation Decisions

### OR Syntax Boundary
- **D-01:** OR support is limited to token-local forms inside one whitespace-delimited
  term. Supported shapes include `@work|@home`, `+proj1|+proj2`, and `(A)|(B)`.
- **D-02:** Grouped negation and broader grouped grammar are out of scope for v1.6.3.
  Forms such as `-(@work|@home)` are not part of this phase's contract.

### Negation Semantics
- **D-03:** A leading `-` does not create grouped-negation semantics for OR terms in
  v1.6.3. Users should not rely on `-@work|@home` as a special grouped form.
- **D-04:** Documentation should explicitly state that grouped negation is unsupported in
  this phase, rather than implying DeMorgan-style branch distribution.

### Malformed OR Handling
- **D-05:** Malformed OR input is handled permissively by ignoring empty branches.
  Examples like `@work|`, `|@home`, and `(A)|` should evaluate using the remaining valid
  branch or branches.
- **D-06:** The permissive behavior is meant to make filter editing forgiving, not to
  broaden the grammar. Ambiguous grouped-negation syntax remains unsupported even if the
  token contains `|`.

### Documentation and Examples
- **D-07:** Phase 46 should document the supported OR syntax explicitly, not just the
  happy path.
- **D-08:** The docs/tests contract should call out three rules:
  token-local OR is supported, grouped negation is unsupported, and empty OR branches are
  tolerated by ignoring empties.

### Folded Todos
- **Add OR operator support to filter engine** — The todo identified the core user problem
  that all filters are currently AND-only and proposed token-local `|` as the least
  disruptive syntax expansion. It fits this phase directly and anchors the parser/evaluator
  work in `todotxt-core` with transparent reuse from the TUI call sites.

### the agent's Discretion
- Exact internal representation for OR terms in the parser and evaluator helpers.
- Whether malformed empty-branch handling is implemented by filtering blank branches at
  parse time or by producing a normalized OR term before evaluation.
- Exact wording/location of the new syntax examples in user-facing help or docs.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and roadmap
- `.planning/ROADMAP.md` — Phase 46 goal, requirement mapping, and success criteria.
- `.planning/REQUIREMENTS.md` — `FILT-01`, `FILT-02`, and `FILT-03` define the required
  OR behavior and compatibility expectations.

### Prior filter/input context
- `.planning/phases/42-filter-autocomplete-coverage/42-CONTEXT.md` — established filter
  input behavior and confirms Phase 46 should stay focused on parser/evaluator semantics,
  not autocomplete mechanics.
- `.planning/phases/41-full-presets-filter-history-pane-task-movement/41-CONTEXT.md` —
  captures the existing filtering UX assumptions and shared autocomplete/filter history
  machinery around compound expressions.

### Existing implementation targets
- `crates/todotxt-core/src/filter.rs` — primary parser and evaluator change site for OR
  token support.
- `crates/todotxt-tui/src/app.rs` — existing `Filter::from_query` call sites in the TUI
  that should continue working without TUI-specific grammar logic.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/todotxt-core/src/filter.rs`: existing filter term parsing and evaluation are the
  natural place to add OR support without duplicating logic in UI layers.
- `crates/todotxt-tui/src/app.rs`: current TUI filtering flow already delegates query
  interpretation to core filter parsing, so OR support can remain transparent at the call
  sites.

### Established Patterns
- Filter behavior is currently built around whitespace-delimited terms with existing AND
  semantics. Extending one term to contain `|` is consistent with that contract and avoids
  inventing a larger expression grammar.
- Nearby phases kept filter UX enhancements localized: Phase 41 handled history/presets,
  Phase 42 handled autocomplete. This phase should follow that separation and keep changes
  centered on parsing/evaluation.

### Integration Points
- Parser changes belong in `todotxt-core`, with evaluator changes in the same module so
  CLI and TUI inherit identical semantics.
- Tests should live beside the filter parser/evaluator to lock down token-local OR,
  compatibility with AND, unsupported negation-group behavior, and permissive empty-branch
  handling.

</code_context>

<specifics>
## Specific Ideas

- Preferred examples to preserve in docs/tests: `@work|@home`, `+proj1|+proj2`,
  `(A)|(B)`, and a mixed expression like `(A)|(B) @work`.
- Explicitly avoid implying support for `-(@work|@home)` in this phase.
- If malformed OR examples are documented, show that empty sides are ignored rather than
  treated as a new matching mode.

</specifics>

<deferred>
## Deferred Ideas

- Grouped negation support such as `-(@work|@home)` or a richer parenthesized query
  grammar belongs in a future filter-language phase if it becomes valuable.

</deferred>

---

*Phase: 46-filter-engine-upgrade*
*Context gathered: 2026-05-15*

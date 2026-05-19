# Phase 46: Filter Engine Upgrade - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-15
**Phase:** 46-filter-engine-upgrade
**Areas discussed:** OR syntax boundary, Negation semantics, Malformed input handling, Documentation/examples contract

---

## OR Syntax Boundary

| Option | Description | Selected |
|--------|-------------|----------|
| Token-local OR only | Support OR only inside one whitespace-delimited token such as `@work|@home` and `(A)|(B)` | ✓ |
| Token-local OR plus grouped negation | Also support forms like `-(@work|@home)` in this phase | |
| Something else | Define a custom supported syntax set | |

**User's choice:** Token-local OR only
**Notes:** Keep the grammar small for v1.6.3 and avoid a broader expression parser.

---

## Negation Semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Distribute negation across branches | Treat `-@work|@home` like "not work and not home" | |
| No grouped negation in v1.6.3 | Do not treat a leading `-` as special grouped OR negation | ✓ |
| Something else | Define a different negation rule | |

**User's choice:** No grouped negation in v1.6.3
**Notes:** The supported syntax should stay explicit and avoid implied grouped semantics.

---

## Malformed Input Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Be permissive | Ignore empty branches and evaluate remaining valid branches | ✓ |
| Fail closed | Treat malformed OR tokens as non-matching | |
| Hybrid | Tolerate simple cases but reject ambiguous malformed forms | |

**User's choice:** Be permissive
**Notes:** Examples like `@work|` and `(A)|` should still behave usefully by ignoring empty sides.

---

## Documentation/examples contract

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal | Document only happy-path examples | |
| Explicit contract | Document supported OR syntax, unsupported grouped negation, and empty-branch tolerance | ✓ |
| Something else | Use a custom level of explicitness | |

**User's choice:** Explicit contract
**Notes:** The edge-rule documentation is part of the phase value because future confusion will cluster around syntax boundaries.

---

## the agent's Discretion

- Internal OR representation and helper function structure in the filter parser/evaluator.
- Exact doc/help placement for syntax examples.

## Deferred Ideas

- Richer grouped filter grammar, including grouped negation, belongs in a later phase if needed.

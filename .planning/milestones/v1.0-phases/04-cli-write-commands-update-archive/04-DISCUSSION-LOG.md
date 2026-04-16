# Phase 4: CLI Write Commands — Discussion Log

**Phase:** 04 — CLI Write Commands
**Date:** 2026-04-15
**Status:** Ready for Planning (no open discussions)

---

## Discussion Notes

*(No open discussion items. All decisions resolved in CONTEXT.md.)*

---

## Key Decisions Made

All decisions were derived from codebase analysis (Phases 1–3 patterns), REQUIREMENTS.md, and
the project's stated goal of a first-class agent-friendly CLI.

| Decision | Resolution | Rationale |
|----------|------------|-----------|
| Creation date on `add` | Config flag, default off | Agent callers need explicit control |
| Multi-ID for do/undo/del | Variadic, descending order | Batch operations common in agent use |
| do/undo idempotency | No-op on already-done state, exit 0 | Retry-safe for automation |
| del confirmation | None | Anti-feature for scriptable CLIs |
| append semantics | Append to end of raw line | Simplest, matches todo.sh convention |
| prepend semantics | Insert before body via builder | Correct placement after prefix fields |
| edit semantics | Full replacement | Predictable; no merge ambiguity |
| Output after write | Task to stdout, info to stderr | Consistent with read commands |

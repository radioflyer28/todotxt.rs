# Phase 14: Compat Discovery + Spec Lock - Context

**Gathered:** 2026-04-23
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase produces **discovery documents and a spec lock** — its output is a contract governing Phases 15–17, not shipping code. Three deliverables:
1. todo.sh compatibility surface definition (commands + flag signatures)
2. `t:` deferred-task parity decision (documented and locked)
3. Implementation contract document(s) for downstream phases

</domain>

<decisions>
## Implementation Decisions

### todo.sh Compatibility Target

- **D-01:** Target BOTH `todo.sh` (original bash script) AND `todotxt.net` (.NET desktop app) — define compat surface separately for each, since they diverge on some commands and conventions.
- **D-02:** Compatibility depth = **commands + flag signatures**. Not just aliases, not full semantic parity. Command names, aliases (`add`/`a`, `do`/`x`, etc.), and flag signatures (e.g., `-+`, `-@`) are in scope; output format and error message verbatim matching are not.
- **D-03:** When todo.sh behavior conflicts with our design (positional args, implicit positional matching), **document the gap and skip it** — no shims, no bridges. Phase 15 implements what's in scope only.

### t: Threshold Date Behavior

- **D-04:** Tasks with a future `t:YYYY-MM-DD` are **hidden from `list` output by default**. This is full filtering behavior — matches the original todo.sh intent.
- **D-05:** Toggle: `list --all` (CLI) + a dedicated TUI key toggle. Both surfaces expose the same show/hide decision.
- **D-06:** When deferred tasks are shown (via `--all` or TUI toggle), they are **displayed with a greyed-out color scheme** — a visual distinction from active tasks, consistent with the existing priority/overdue color system (Phase 13 patterns).
- **D-07:** Hiding scope is **`list` only** — all other commands (`do`, `del`, `edit`, `append`, `pri`, etc.) operate on deferred tasks normally by line number or search. No hidden-from-all-commands behavior.
- **D-08:** `V12-TUI-DEFER-01` is **confirmed**: implement `t:` filtering. `V12-TUI-DEFER-02` is therefore in scope for Phases 15/17 as previously planned.

### Output Format + Exit Codes

- **D-09:** `list` output format: keep our current format (comfy-table / plain). A `--compat` flag (name TBD in Phase 15) emits todo.sh-style `{N} {task}` numbered output when explicitly requested. This is opt-in — not the default.
- **D-10:** Exit codes: keep our **0/1/2 scheme** (0 = success, 1 = not found, 2 = other error). todo.sh uses 0/1 only. Document this divergence in the compatibility surface spec — do not change our exit codes.

### Spec Lock Artifact Format

- **D-11:** The "implementation contract" for downstream phases is the decisions in this CONTEXT.md plus any per-phase SPEC.md or PLAN.md notes. No separate ADR file is required for this discovery phase — decisions are captured here and referenced by downstream PLAN.md files.

### the agent's Discretion

- Which specific todo.sh commands/flags appear in the compat surface doc — researcher investigates and proposes the full list; planner locks it in PLAN.md.
- Whether the compat surface document is a SPEC.md, a markdown table in PLAN.md, or a dedicated ADR — planner chooses format that best serves Phase 15's executor.
- Edge cases for `t:` (e.g., `t:` equal to today — treat as active or deferred?) — researcher should document the todo.sh convention; planner locks the decision.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Implementation
- `crates/todotxt-core/src/task.rs` — `Task` struct + `threshold_date` field + `extract_tags()` — how `t:` is currently parsed
- `crates/todotxt-cli/src/cli.rs` — current command + flag definitions (Clap 4.6)
- `crates/todotxt-core/src/filter.rs` — filter engine; threshold filtering needs to be added here
- `crates/todotxt-cli/src/commands/list.rs` — list command; `--all` flag and deferred-task hide logic goes here

### Requirements
- `.planning/REQUIREMENTS.md` — V12-TUI-DEFER-01 (decision), V12-COMPAT-01/02 (compat layer + tests)
- `.planning/ROADMAP.md` — Phase 14 goal and downstream phase summaries

### External References
- todo.sh source: https://github.com/todotxt/todo.txt-cli — canonical bash implementation for command surface and flag signature research
- todotxt.net source / docs: reference for .NET desktop app behavior differences

</canonical_refs>

<specifics>
## Specific Ideas

- `--all` is the chosen flag name for "show deferred tasks" on `list` — consistent with the convention of `--all` meaning "include hidden items" in many CLI tools.
- Greyed-out styling for deferred tasks follows the existing `owo-colors` + `StyleSheet` pattern from Phase 13 — reuse that system rather than introducing new color primitives.
- The compat mode output flag name (`--compat`?) is a placeholder — researcher should check if todo.sh users have a strong convention expectation here.

</specifics>

<deferred>
## Deferred Ideas

- Full semantic compat (output format, error message verbatim matching) — explicitly out of scope for v1.2.
- Hide deferred tasks from all commands by default — rejected; filtering scope is `list` only.
- Spec lock as a separate ADR file — not needed; decisions captured in CONTEXT.md + PLAN.md.

</deferred>

---

*Phase: 14-compat-discovery*
*Context gathered: 2026-04-23 via /gsd-discuss-phase 14*

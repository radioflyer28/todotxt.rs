# Phase 48: Recurring Workflow Core - Context

**Gathered:** 2026-05-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 48 adds first-class recurring task behavior to completion flows. The scope is limited
to recognizing `rec:` recurrence tokens, calculating the next occurrence correctly for
strict and relative recurrence modes, and making CLI and TUI completion paths produce the
same recurring result for single-task and multi-task completion.

</domain>

<decisions>
## Implementation Decisions

### Recurrence Anchoring
- **D-01:** Strict recurrence tokens such as `rec:+1d` anchor from the prior due date when
  a due date exists.
- **D-02:** Relative recurrence tokens such as `rec:1d` anchor from the completion date.
- **D-03:** If a recurring task has no due date to anchor from, both strict and relative
  recurrence fall back to the completion date.

### Completion Flow
- **D-04:** Completing a recurring task should automatically create the next occurrence.
  There is no extra prompt or confirmation step for recurring completion.
- **D-05:** This auto-create rule applies consistently in CLI single-ID, CLI multi-ID, TUI
  single-task completion, and TUI bulk mark-done flows.

### Carry-Forward Semantics
- **D-06:** The next occurrence should preserve nearly all task identity and metadata:
  description, projects, contexts, `rec:` token, priority, threshold date, and other
  non-completion metadata carry forward.
- **D-07:** Completion-only state resets on the next occurrence. The new task must be
  incomplete, clear any completion marker/date, and recalculate the due date from the
  recurrence rule.
- **D-08:** Each completion event creates exactly one next occurrence for each recurring
  task, including bulk completion flows.

### Requirement Reconciliation
- **D-09:** Earlier milestone wording said recurring completion should prompt before
  creating the next occurrence. That is now superseded by the newer decision above:
  recurrence creation is implicit on completion. Downstream planning should update the
  requirement/roadmap wording so implementation and docs follow the promptless model.

### the agent's Discretion
- Exact internal representation and parsing/validation strategy for recurrence interval
  syntax, as long as strict `rec:+...` and relative `rec:...` remain distinguishable.
- Exact naming and placement of shared recurring-completion helpers, provided CLI and TUI
  completion paths use one consistent behavior contract.
- Exact user-facing wording in CLI/TUI status messages, provided they do not introduce a
  confirmation prompt.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and roadmap
- `.planning/ROADMAP.md` - Phase 48 goal, requirement mapping, and success criteria.
- `.planning/REQUIREMENTS.md` - `REC-01` through `REC-04`, including the stale prompt-based
  wording that must be reconciled with this context.
- `.planning/PROJECT.md` - milestone-level intent for recurring tasks in v1.6.3.
- `.planning/STATE.md` - current workflow position and active milestone state.

### Core task model
- `crates/todotxt-core/src/task.rs` - current completion semantics, metadata-preserving
  builder methods, and normalization behavior that already preserves unknown tokens like
  `rec:+1w`.

### Existing completion paths
- `crates/todotxt-cli/src/commands/complete.rs` - current CLI `do` and `undo` behavior for
  single and multi-ID completion.
- `crates/todotxt-tui/src/app.rs` - current TUI single-task toggle-done and bulk mark-done
  behavior that Phase 48 must align with CLI semantics.

### Prior planning context
- `.planning/phases/47-tui-readability/47-CONTEXT.md` - recent example of milestone-phase
  context structure and decision capture style for downstream GSD agents.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `Task::with_completed` in `crates/todotxt-core/src/task.rs` already owns todo.txt
  completion semantics such as stamping completion date and stripping priority.
- Builder methods such as `with_due_date`, `with_threshold_date`, and `with_creation_date`
  provide a natural way to construct the next occurrence while preserving most task fields.
- CLI completion is already centralized in `run_do`, which makes it a clean seam for shared
  recurring completion behavior once the core contract is defined.

### Established Patterns
- Unknown body tokens are preserved through parsing and normalization today, so `rec:...`
  can participate in recurrence without requiring a separate non-todo.txt sidecar format.
- Completion flows already mutate tasks through the shared `TaskList` update path, so
  recurring behavior should attach to completion rather than inventing a parallel workflow.
- TUI has both single-task and bulk completion paths today, which means Phase 48 must make
  bulk recurrence behavior explicit rather than assuming only one-by-one completion.

### Integration Points
- `crates/todotxt-core/src/task.rs` is the likely home for recurrence-aware task mutation
  helpers or parsing support.
- `crates/todotxt-cli/src/commands/complete.rs` must apply recurring behavior for one or
  many IDs without diverging from TUI rules.
- `crates/todotxt-tui/src/app.rs` must route both `toggle_done` and `bulk_mark_done`
  through the same recurring completion contract used by CLI.

</code_context>

<specifics>
## Specific Ideas

- Recurring creation is implicit because the next occurrence is considered part of what
  “complete recurring task” means, not a separate optional action.
- Bulk completion should not degrade recurrence correctness or silently skip recurring
  regeneration.
- The strict vs relative split should match the todo.txt mental model: strict keeps cadence
  from the scheduled due date, relative slides from the actual completion date.

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within phase scope.

</deferred>

---

*Phase: 48-recurring-workflow-core*
*Context gathered: 2026-05-18*

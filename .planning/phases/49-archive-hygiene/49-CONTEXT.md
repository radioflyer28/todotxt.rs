# Phase 49: Archive Hygiene - Context

**Gathered:** 2026-05-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 49 adds rotation behavior to the existing `done.txt` archive flow so completed-task
archives stay organized over time. The scope is limited to time-based rotation, period-based
 archive filenames, and user-visible archive feedback when rotation occurs during archive
 writes.

</domain>

<decisions>
## Implementation Decisions

### Rotation Policy
- **D-01:** Rotation is time-based first, not size- or line-threshold based.
- **D-02:** Monthly rotation is the initial shipped policy.
- **D-03:** The implementation should leave room for future period-based variants such as
  weekly rotation, but this phase only needs to deliver the monthly behavior.

### Archive Naming
- **D-04:** Rotated archive files use period-based names that encode the archive bucket.
- **D-05:** For the monthly policy, the canonical shape should be like `done-2026-05.txt`.
- **D-06:** The naming scheme should extend naturally to other future period policies, such
  as weekly buckets, without requiring a redesign.

### Retention Behavior
- **D-07:** Automatic retention cleanup is out of scope for this phase.
- **D-08:** Keep all rotated archive files for now; rotation should not delete old rotated
  files.

### Rotation Timing and Feedback
- **D-09:** Rotation happens only when archive writes completed tasks.
- **D-10:** If an archive action crosses into a new period bucket, the system should rotate
  at that point and keep user messaging explicit that rotation occurred.
- **D-11:** Rotation behavior should fit into the existing archive workflows in CLI and TUI
  rather than becoming a separate proactive startup/background process.

### Requirement Reconciliation
- **D-12:** Earlier milestone wording described rotation in terms of size/line thresholds
  and retention policy. That is now superseded by the newer time-based monthly rotation
  decision and the explicit choice to defer cleanup retention. Downstream planning should
  update requirement and roadmap wording to match this phase discussion.

### the agent's Discretion
- Exact configuration shape for selecting the active rotation cadence, as long as monthly is
  the default delivered policy and the structure can later grow to weekly or similar periods.
- Exact messaging text in CLI and TUI when a rotation occurs.
- Exact internal helper placement for sharing rotation logic between existing archive entry
  points.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements and roadmap
- `.planning/ROADMAP.md` - Phase 49 goal, requirement mapping, and current success criteria.
- `.planning/REQUIREMENTS.md` - `DONE-01` through `DONE-03`, including the stale threshold
  and retention wording that must be reconciled with this context.
- `.planning/PROJECT.md` - milestone-level archive hygiene intent for v1.6.3.
- `.planning/STATE.md` - current workflow position and active milestone state.

### Existing archive behavior
- `crates/todotxt-cli/src/commands/archive.rs` - current CLI archive implementation, done
  path resolution, and atomic write ordering.
- `crates/todotxt-tui/src/app.rs` - current TUI archive workflow, atomic append behavior,
  and archive confirmation flow.
- `crates/todotxt-cli/src/config.rs` - current CLI config surface around `done_file`.
- `crates/todotxt-tui/src/config.rs` - current TUI config and archive path resolution.

### Prior planning context
- `.planning/phases/48-recurring-workflow-core/48-CONTEXT.md` - recent example of
  milestone-phase context capture and requirement reconciliation for downstream agents.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- CLI archive behavior is already centralized in `run_archive` inside
  `crates/todotxt-cli/src/commands/archive.rs`.
- TUI archive behavior is already centralized in `archive_tasks()` inside
  `crates/todotxt-tui/src/app.rs`.
- Both flows already resolve the effective `done.txt` path and write archive content
  atomically before mutating active tasks.

### Established Patterns
- Archive writes are treated as explicit user-triggered operations, not background
  maintenance tasks.
- Both CLI and TUI already preserve a strong crash-safety story around archive writes, so
  rotation should extend that behavior rather than weakening it.
- `done.txt` is the canonical archive sink today, which means rotation should feel like a
  continuation of existing archive semantics rather than a separate subsystem.

### Integration Points
- `crates/todotxt-cli/src/commands/archive.rs` needs rotation-aware write behavior and
  user-visible rotation messaging.
- `crates/todotxt-tui/src/app.rs` needs the same rotation-aware archive behavior during TUI
  archive actions.
- `crates/todotxt-cli/src/config.rs` and `crates/todotxt-tui/src/config.rs` are the likely
  homes for any new rotation-policy configuration surface.

</code_context>

<specifics>
## Specific Ideas

- The initial shipped archive policy should bucket completed tasks by month.
- A rotated file should read like a period ledger, so `done-YYYY-MM.txt` is preferable to
  sequence-based or timestamp-heavy names.
- Rotation should stay tied to user archive actions, with messaging explicit enough that the
  user knows when a new archive period was opened.

</specifics>

<deferred>
## Deferred Ideas

- Automatic retention cleanup of old rotated archives.
- Threshold-based size or line-count rotation policies.
- Proactive startup/open-time rotation before any archive write occurs.

</deferred>

---

*Phase: 49-archive-hygiene*
*Context gathered: 2026-05-19*

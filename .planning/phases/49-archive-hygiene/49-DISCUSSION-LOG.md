# Phase 49: Archive Hygiene - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-05-19
**Phase:** 49-archive-hygiene
**Areas discussed:** Rotation trigger policy, Rotated file naming, Retention behavior, User feedback and timing

---

## Rotation trigger policy

| Option | Description | Selected |
|--------|-------------|----------|
| Threshold OR | Rotate when either configured line-count or size threshold is exceeded. | |
| Threshold AND | Rotate only when both thresholds are exceeded. | |
| Single active threshold | Only one threshold type is active at a time. | |
| Something else | User provides a different trigger rule. | ✓ |

**User's choice:** Time-based rotation, monthly first, with room to support weekly and similar cadences later.
**Notes:** This replaces the earlier threshold-centric framing for this phase.

---

## Rotated file naming

| Option | Description | Selected |
|--------|-------------|----------|
| Period-based name | Deterministic names like `done-2026-05.txt` or `done-2026-W21.txt`. | ✓ |
| Timestamped export name | Precise timestamp names like `done-2026-05-19T1430.txt`. | |
| Sequence suffix | Numbered names like `done-001.txt`. | |
| Something else | User provides a custom naming rule. | |

**User's choice:** Period-based name
**Notes:** Monthly archives should read as named time buckets.

---

## Retention behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Retain by number of rotated archive files | Keep the newest N rotated files, excluding active `done.txt`. | |
| Retain by time window | Keep a fixed time span of rotated archives. | |
| Keep everything for now | Rotate archives but do not auto-delete old rotated files. | ✓ |
| Something else | User provides a custom retention rule. | |

**User's choice:** Keep everything for now
**Notes:** Automatic cleanup is deferred rather than partially shipped.

---

## User feedback and timing

| Option | Description | Selected |
|--------|-------------|----------|
| Rotate only when archive writes completed tasks | Archive actions trigger rotation and should say when rotation happened. | ✓ |
| Rotate proactively on startup or open | Opening the app or command can rotate even before new archive writes. | |
| Quiet rotation | Rotate during archive writes without explicit user messaging. | |
| Something else | User provides a custom timing or messaging rule. | |

**User's choice:** Rotate only when archive writes completed tasks
**Notes:** Rotation should stay inside existing archive workflows with explicit feedback.

---

## the agent's Discretion

- Exact config shape for selecting cadence.
- Exact messaging text for rotation events.
- Exact helper placement for shared rotation logic.

## Deferred Ideas

- Automatic retention cleanup.
- Threshold-based rotation.
- Proactive startup rotation.

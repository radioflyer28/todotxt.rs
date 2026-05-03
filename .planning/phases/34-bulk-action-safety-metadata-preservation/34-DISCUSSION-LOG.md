# Phase 34: Bulk Action Safety + Metadata Preservation — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in 34-CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-30
**Phase:** 34-bulk-action-safety-metadata-preservation
**Areas discussed:** Count preview scope, Preview UX for overwrite pickers, i-priority picker scope, Metadata preservation edge cases

---

## Count Preview Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Overwrite pickers only (s, i) | Only `s` and `i` get preview; D and T stay as-is | |
| All bulk mutations except @/+ | Add preview to s/i and T; D already has confirmation | |
| Unify all bulk actions | Consistent count preview for s, i, T, and D | ✓ |

**User's choice:** Unify all bulk actions

---

| Option | Description | Selected |
|--------|-------------|----------|
| Only when N > 1 tasks affected | Single-task operations apply directly without a preview | ✓ |
| Always preview (even N=1) | Always show preview — consistent but adds a keypress | |

**User's choice:** Only when N > 1 tasks affected

---

| Option | Description | Selected |
|--------|-------------|----------|
| Preserve selection after cancel | Selection stays intact so user can re-trigger a different action | ✓ |
| Clear selection after cancel | Cancel clears selection — clean slate | |

**User's choice:** Preserve selection after cancel

---

## Preview UX for Overwrite Pickers

| Option | Description | Selected |
|--------|-------------|----------|
| Inline in picker (count in picker header) | Count shown in overlay header throughout picker interaction | ✓ |
| Separate confirm step after selection | Picker → pick date/priority → separate y/n panel → apply | |
| Agent's discretion | As long as s and i match | |

**User's choice:** Inline in picker — count in picker header (e.g., "Setting due date — 5 tasks")

---

| Option | Description | Selected |
|--------|-------------|----------|
| Keep existing T and D patterns — just align wording | D stays as panel; T stays as direct text entry | |
| Give T a count banner before text entry; update D wording | T gets a count banner; D panel wording aligned to unified model | ✓ |

**User's choice:** T gets a count banner before text entry; D wording updated

---

## i-Priority Picker Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Add i-picker in Phase 34 | Phase 34 adds CAP-04 gap from Phase 33 | ✓ |
| Defer i-picker — not in Phase 34 scope | Leave for later; Phase 34 focuses on bulk safety only | |
| Opportunistic — add if it fits | Add only if it fits the 2-plan budget | |

**User's choice:** Add `i` picker in Phase 34

---

| Option | Description | Selected |
|--------|-------------|----------|
| Flat A–Z list, press letter or arrow+Enter | Flat list, single-key letter selection | |
| Scrollable picker overlay (same pattern as s) | Overlay + type-to-filter + Enter | ✓ |

**User's choice:** Scrollable picker overlay — same pattern as `s`, but with type-to-filter on top (type a priority letter and press Enter to set)

---

## Metadata Preservation Edge Cases

| Option | Description | Selected |
|--------|-------------|----------|
| Full field preservation — all non-target fields survive | All fields preserved: x, completion date, creation date, t:, due:, priority, @/+ | ✓ |
| Core fields only (priority, context, project, due) | Lower-traffic fields like t: and creation date can be normalizer-handled | |
| Agent's discretion | No specific constraint | |

**User's choice:** Full field preservation — all non-target fields survive

---

| Option | Description | Selected |
|--------|-------------|----------|
| Structured mutation via Task model | Parse to Task struct, mutate target field only, serialize back | ✓ |
| Raw token replacement (current approach in s picker) | Split raw string, filter out target token, re-append new token | |
| Agent's discretion | If current approach is safe, keep it | |

**User's choice:** Structured mutation via Task model

---

| Option | Description | Selected |
|--------|-------------|----------|
| Confirm normalizer must not alter non-target fields | Tests must verify no side effects | |
| Trust normalizer as-is | Normalizer is already field-aware and safe | ✓ |
| Skip normalizer for setter mutations | Write raw directly after structured mutation | |

**User's choice:** Trust normalizer as-is

---

| Option | Description | Selected |
|--------|-------------|----------|
| No-op with status hint on completed tasks | Property setters are read-only for completed tasks | |
| Allow mutation on completed tasks | Mutate the relevant field; preserve x prefix and completion date | ✓ |
| Agent's discretion | | |

**User's choice:** Allow mutation on completed tasks — preserve `x` prefix and completion date, mutate target field only.

**Notes:** User noted a future idea: completion date picker (marking tasks complete on dates other than today). Noted for backlog, not in Phase 34 scope.

---

## Agent's Discretion

- Exact overlay widget reuse strategy between `s` and `i` pickers
- Exact wording of count labels in the picker overlay header
- Whether `i` picker includes "no priority" item at top or bottom of list
- Visual styling of the count label

## Deferred Ideas

- **Completion date picker** — allow setting a completion date other than today when marking tasks done. Belongs in a future phase.

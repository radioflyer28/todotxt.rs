# Phase 21: Smart Text Normalization — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Session:** 2026-04-25

---

## Area: Append Merge Strategy

**Q:** When the user appends text like `(B) +newproject due:2026-05-01 and some plain text`,
how should the merge work?

**Options presented:**
- Parse-then-merge: parse appended text into tokens, merge fields, rebuild via `rebuild_raw()`
- Append-then-reparse: concatenate raw strings, parse combined as a fresh Task
- You decide

**Selected:** Parse-then-merge

**Notes:** Naive re-parse absorbs appended tokens into `body` — NORM-01 through NORM-04 require
field-aware merging. `rebuild_raw()` already exists and produces canonical format.

---

## Area: Priority Conflict During Append

**Q:** When appended text includes a priority token like `(B)` and the original task already
has `(A)`, what wins?

**Options presented:**
- Keep original priority: appended priority is dropped
- Replace priority: appended priority wins over existing
- Different rules for append vs edit

**Selected:** Replace priority (appended priority wins)

**Notes:** Lets the user reprioritize from the append bar without opening the full edit flow.

---

## Area: Edit Flow Normalization Scope

**Q:** For the edit flow (21-03): when user edits a task's full text in the inline editor
and hits Enter, what normalization applies?

**Options presented:**
- Re-parse only: normalize by re-parsing the full edited text (current behavior)
- Full normalize: apply same parse-then-merge to edited text
- You decide

**Selected (freetext):** Make the behavior configurable in `config.toml`. Not sure whether
this requires updating the CLI but likely does require updating the core crate.

**Follow-up Q:** What should the `config.toml` normalization setting look like?

**Options presented:**
- Single toggle: `normalize_on_edit = true/false`
- Two toggles: `normalize_append` and `normalize_edit` independently
- You decide

**Selected:** Two independent toggles

---

## Area: CLI Scope

**Q:** How far should Phase 21 go with the CLI `append` command?

**Options presented:**
- TUI only for now: CLI append stays unchanged
- Update CLI append too: share core helper, CLI gets a `--normalize` flag
- You decide

**Selected:** Update CLI append too, with `--normalize` flag (explicit opt-in, not breaking)

---

## Area: Core API Shape (NORM-06)

**Q:** What shape should the core API take in `todotxt-core`?

**Options presented:**
- Standalone function: `normalize_append(task, text) -> Task`
- Builder method: `Task::with_appended_normalized(text) -> Task`
- You decide

**Selected:** Standalone function `normalize_append(task, text) -> Task`

**Notes:** Multi-field merge is distinct from single-field builder mutations. Standalone fn
is cleaner and easier to test in isolation.

---

## Area: Unknown Metadata (NORM-05)

**Q:** What happens with custom/unrecognized metadata tokens like `rec:+1w` or `foo:bar`?

**Options presented:**
- Preserve unknown tokens in body verbatim (current `rebuild_raw()` behavior)
- Track all key:value as generic `custom_fields` in Task struct

**Selected:** Preserve verbatim in body — no `custom_fields` map needed.

---

*Log complete — all decisions captured in 21-CONTEXT.md*

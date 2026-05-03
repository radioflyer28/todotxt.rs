---
plan: 14-01
phase: 14-compat-discovery
status: complete
completed: "2026-04-23"
key-files:
  created:
    - .planning/phases/14-compat-discovery/14-COMPAT-SPEC.md
    - .planning/phases/14-compat-discovery/14-DEFER-SPEC.md
---

# Plan 14-01 Summary: Compat Surface Spec + t: Defer Contract

## What Was Produced

Two locked implementation-contract documents:

1. **14-COMPAT-SPEC.md** — todo.sh command/alias/gap surface contract for Phase 15. Covers 9 aliases to add, 3 new commands (`listpri`, `listall`, `deduplicate`), compat output mode (`--compat` flag), intentional gaps, commands with different semantics, global flag mapping, and exact files-to-modify in Phase 15.

2. **14-DEFER-SPEC.md** — `t:` threshold date implementation contract for Phase 15 (CLI) and Phase 17 (TUI). Documents current filter state, exact Rust code snippets for `cli.rs` and `list.rs` changes, TUI toggle and greyed styling spec, and decision log with V12-TUI-DEFER-01 confirmed.

## Key Findings

- **`t:` filtering already implemented** in `crates/todotxt-core/src/filter.rs` (`suppress_future_threshold: true` is the default). No core library changes are needed for Phase 15.
- **9 command aliases** are missing from our CLI (add→a, append→app, del→rm, depri→dp, do→done, prepend→prep, pri→p, contexts→lsc, projects→lsprj).
- **3 new commands** need to be implemented: `listpri` (priority filter with range support), `listall` (merged todo+done), `deduplicate` (exact-line deduplication with physical removal).
- **`t:TODAY` edge case confirmed**: tasks with `t:` equal to today are shown (check is `t > today`, not `>=`).

## Deviations from Plan

None. Both spec files produced exactly as specified in the plan.

## Self-Check: PASSED

- [x] `14-COMPAT-SPEC.md` exists with "## Aliases to Add" (9 entries)
- [x] `14-COMPAT-SPEC.md` contains `listpri` with `A-C` range format described
- [x] `14-COMPAT-SPEC.md` contains `deduplicate` with no-blank-line simplification
- [x] `14-COMPAT-SPEC.md` contains "## Files to Modify in Phase 15" with exact file paths
- [x] `14-COMPAT-SPEC.md` contains `--compat` flag documentation
- [x] `14-DEFER-SPEC.md` exists with `suppress_future_threshold` and code location cited
- [x] `14-DEFER-SPEC.md` contains "## Phase 15 Changes Required" with Rust code snippets
- [x] `14-DEFER-SPEC.md` contains "## Phase 17 Changes Required" with TUI toggle and greyed styling
- [x] `14-DEFER-SPEC.md` contains "V12-TUI-DEFER-01: CONFIRMED"
- [x] `14-DEFER-SPEC.md` documents `t:TODAY` edge case (shown, not hidden)
- [x] Both files are self-contained: Phase 15 and Phase 17 executors can implement from these docs alone

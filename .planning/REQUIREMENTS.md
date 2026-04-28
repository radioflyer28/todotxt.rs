# Requirements: v1.3 Feature/Hotkey Parity with todotxt.net

Defined: 2026-04-24
Core Value: A fast, cross-platform todo.txt tool with a first-class CLI for both human and AI agent use.

## v1 Requirements

### Selection

- [x] SEL-01: User can extend the current selection to a contiguous task range with Shift plus navigation keys.
- [x] SEL-02: User can enter a selection mode that allows adding or removing non-contiguous task rows without using the mouse.
- [x] SEL-03: User's selected tasks remain selected when the list is regrouped, resorted, filtered, or reloaded from disk, as long as those tasks still exist.
- [x] SEL-04: Non-task rows such as group headers are never directly selected or mutated by multi-selection actions.

### Bulk Editing

- [ ] BULK-01: User can bulk delete all currently selected tasks with a confirmation flow that clearly shows a multi-task action.
- [ ] BULK-02: User can append text to all currently selected tasks from the TUI.
- [ ] BULK-03: User can see how many tasks are currently selected from the TUI status or help surface before running a bulk action.

### Text Normalization

- [ ] NORM-01: When appended or edited text contains a valid priority token, the saved task places the priority in canonical todo.txt prefix position.
- [ ] NORM-02: When appended or edited text contains recognized +project metadata, the saved task places project tags after the task body in stable todo.txt form.
- [ ] NORM-03: When appended or edited text contains recognized @context metadata, the saved task preserves valid contexts without requiring them to be moved out of inline body text.
- [ ] NORM-04: When appended or edited text contains recognized due: or t: metadata, the saved task normalizes those fields consistently without discarding surrounding plain text.
- [ ] NORM-05: When appended or edited text contains plain text or unrecognized metadata, that content is preserved in the saved task instead of being discarded.
- [ ] NORM-06: Smart normalization uses shared parser or rebuild rules in todotxt-core, with any needed extensions made there rather than duplicating logic in the TUI.

### Parity and Discoverability

- [ ] PAR-01: User can invoke implemented multi-selection and bulk-edit workflows with hotkeys aligned with todotxt.net where practical.
- [ ] PAR-02: User can discover the implemented parity hotkeys and behaviors from in-app help or equivalent TUI guidance.
- [ ] PAR-03: Deliberate deviations from todotxt.net behavior are documented in milestone artifacts so parity choices are explicit.

### Keymap Configuration

- [ ] KEY-01: User can override TUI hotkeys in config.toml for implemented actions without recompiling.
- [ ] KEY-02: Invalid or conflicting key bindings fall back safely and surface a clear error or warning instead of breaking the TUI.
- [ ] KEY-03: Default key bindings remain todotxt.net-oriented, even when keymap overrides are available.

## v2 Requirements

### Broader Bulk Parity

- BULK-04: User can bulk toggle completion across selected tasks.
- BULK-05: User can bulk change priority or due or threshold dates across selected tasks with parity shortcuts.
- BULK-06: User can copy all selected tasks to the clipboard or edit field from the TUI.

### Extended Parity

- PAR-04: User can access a fuller todotxt.net-style shortcut surface beyond the multi-selection workflows targeted in v1.3.
- PAR-05: User can use parity shortcuts that also map to secondary keys like function keys where valuable.

## Out of Scope

| Feature | Reason |
| ------- | ------ |
| GUI parity work | This milestone is TUI-only. |
| CLI parity expansion unrelated to TUI interaction | The milestone goal is migration comfort in the Rust TUI. |
| Automatic rewriting of unknown key:value metadata | Normalize only recognized todo.txt fields; preserve unknown metadata verbatim. |

## Traceability

| Requirement | Phase | Status |
| ----------- | ----- | ------ |
| SEL-01 | Phase 19 | Complete |
| SEL-02 | Phase 19 | Complete |
| SEL-03 | Phase 19 | Complete |
| SEL-04 | Phase 19 | Complete |
| BULK-01 | Phase 20 | Implemented — verification gap (Phase 23) |
| BULK-02 | Phase 20 | Implemented — verification gap (Phase 23) |
| BULK-03 | Phase 20 | Implemented — verification gap (Phase 23) |
| NORM-01 | Phase 21 | Implemented — verification gap (Phase 23) |
| NORM-02 | Phase 21 | Implemented — verification gap (Phase 23) |
| NORM-03 | Phase 21 | Implemented — verification gap (Phase 23) |
| NORM-04 | Phase 21 | Implemented — verification gap (Phase 23) |
| NORM-05 | Phase 21 | Implemented — verification gap (Phase 23) |
| NORM-06 | Phase 21 | Implemented — verification gap (Phase 23) |
| PAR-01 | Phase 22 | Implemented — verification gap (Phase 23) |
| PAR-02 | Phase 22 | Implemented — verification gap (Phase 23) |
| PAR-03 | Phase 22 | Implemented — verification gap (Phase 23) |
| KEY-01 | Phase 22 | Implemented — verification gap (Phase 23) |
| KEY-02 | Phase 22 | Implemented — verification gap (Phase 23) |
| KEY-03 | Phase 22 | Implemented — verification gap (Phase 23) |

Coverage:

- v1 requirements: 19 total
- Mapped to phases: 19
- Unmapped: 0

---
Requirements defined: 2026-04-24
Last updated: 2026-04-28 — gap closure routing via Phase 23 (verification gap)

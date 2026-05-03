# Phase 11: Edit Mode — Discussion Log

**Date:** 2026-04-20
**Phase:** 11 — Edit Mode
**Status:** Complete — all gray areas resolved

---

## Gray Areas Discussed

### 1. Edit/Add Input Surface

**Question:** Where does the add/edit input field appear?

**Options presented:**
- A: Dedicated one-line input row at top (like legacy app's IntellisenseTextBox)
- B: Footer/status bar row replaced by input when in Add/Edit mode *(recommended)*
- C: Centered modal/overlay on top of the task list

**Decision:** **B — Footer swap**

Status bar is replaced by a `tui-textarea` widget in Adding/Editing mode. Layout stays `[Min(0), Length(1)]` — no extra rows. Clean and idiomatic for TUI apps.

---

### 2. Keybindings

**Question:** Which keybindings for add/edit?

**Options presented:**
- A: `a`=add, `e`=edit, `u`=edit-alias (roadmap + Phase 10 request) *(recommended)*
- B: Legacy-first: `n`=add, `u`=edit, `e`=alias
- C: Strict roadmap: `a`=add, `e`=edit only

**Decision:** **B — Legacy-first: n=add, u=edit, e=alias**

User prefers the legacy C# app keybinding convention. Deviates from REQUIREMENTS.md TUI-ACT-03 (`a`) but satisfies TUI-ACT-04 (`e` is wired as alias). `n` is primary for add; `u` is primary for edit (consistent with Phase 10 commit `2970e0c`).

---

### 3. Autocomplete Behavior

**Question:** How should @ and + autocomplete work?

**Options presented:**
- A: Full legacy popup — `@`/`+` trigger, filter-as-you-type, Down-to-focus, Tab/Enter/Space accept, Esc close *(recommended)*
- B: Inline Tab-cycle — no popup widget
- C: Popup + Tab-only accept

**Decision:** **A — Full legacy popup**

Full feature parity with legacy C# app's `IntellisenseTextBox`. Tokens sourced from existing tasks. Down arrow focuses popup; Tab/Enter/Space insert selected token; Esc closes without inserting. Most feature-complete option.

---

### 4. Delete Confirmation Style

**Question:** How should delete confirmation appear?

**Options presented:**
- A: Inline footer `y/N` prompt in status bar row *(recommended)*
- B: Modal overlay centered on screen (like legacy)
- C: Extra row above status bar with task preview

**Decision:** **C — Bottom panel row above status bar**

An extra row appears above the status bar during DeleteConfirm mode. Layout expands to `[Min(0), Length(1), Length(1)]`: task list + confirm panel + status bar. The confirm panel shows the task text preview and key hints. More visible than the footer-only approach while keeping the status bar always in view.

---

### 5. Reload Queuing (TUI-UX-03)

**Question:** What happens when an external file change is detected during add/edit?

**Options presented:**
- A: Silent queue — `pending_reload` flag set; reload applies on save/cancel, no indicator *(recommended)*
- B: Small `[file changed]` notice in status bar while editing
- C: Blocking warning before save/cancel

**Decision:** **A — Silent queue, apply on exit**

No user-visible indicator. The reload is fully invisible — `pending_reload: bool` is set, checked on Normal-mode re-entry, and applied there. Simplest and least disruptive. User is not concerned about external-change notifications during editing.

---

## Summary of Locked Decisions

| # | Decision | Chosen |
|---|----------|--------|
| D-01 | AppMode enum | Normal / Adding / Editing { original_idx } / DeleteConfirm |
| D-02 | Input surface | Footer swap (status bar → tui-textarea in Add/Edit) |
| D-03 | Input widget | tui-textarea (single-line, input_without_shortcuts) |
| D-04 | Edit pre-populate | Selected task raw text pre-filled in textarea |
| D-05 | Keybindings | n=add, u=edit, e=edit-alias, d=delete |
| D-06 | Delete layout | Extra row above status bar: [Min(0), Len(1), Len(1)] |
| D-07 | Delete confirm keys | y=confirm, any other/Esc=cancel |
| D-08 | Autocomplete | Full popup: @/+ trigger, Down-focus, Tab/Enter/Space accept, Esc close |
| D-09 | Autocomplete state | Option<AutocompleteState> on App |
| D-10 | Reload guard | pending_reload: bool; silent queue; apply on Normal re-entry |
| D-11 | Own-write guard | Covered by same pending_reload path |
| D-12 | Save path | append+save for add; update+save for edit |
| D-13 | Post-op selection | Add→last; edit→same idx; delete→clamped |


# Phase 26: Pane Management + Quick Hide/Show - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** 26-pane-management-quick-hide-show
**Areas discussed:** Hide/Show toggle semantics, Create/Delete guardrails, Hotkey naming and key assignments, Help/status bar discoverability

---

## Hide/Show Toggle Semantics

| Option | Description | Selected |
|--------|-------------|----------|
| Snapshot and restore | Remember pane count + layout, re-show them all on toggle | ✓ |
| Always collapse to 1 pane | Toggle hides all extra panes, show always starts from single-pane default | |

**User's choice:** Snapshot and restore

---

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — preserve filter/sort/group state while hidden | Full state preserved | ✓ |
| No — pane structure restored but filter/sort/group state resets | Simpler | |

**User's choice:** Full state preserved while hidden

---

| Option | Description | Selected |
|--------|-------------|----------|
| Hidden = single-pane view (VIEW-01 behavior) | Reverts to single-pane render, no borders/labels | ✓ |
| Hidden = panes still exist but invisible (overlay-style) | Panes structurally exist but not shown | |

**User's choice:** Single-pane view while hidden

---

| Option | Description | Selected |
|--------|-------------|----------|
| Toggle: one key hides and shows | Same key press alternates states | ✓ |
| Two separate keys | One to hide, one to show | |

**User's choice:** Single toggle key

---

## Create/Delete Guardrails

| Option | Description | Selected |
|--------|-------------|----------|
| Min 1 pane (can't delete last), no max | Unlimited panes | |
| Min 1 pane, max N panes | Cap at a defined limit | |
| Min 0, max 10 | 0 panes valid (e.g., no config panes); max 10 (0–9 index) | ✓ |

**User's choice:** Min 0, max 10 — 0 panes is valid (e.g., none in config.toml). Max is arbitrary at 10; terminal width dictates practical usability, user manages their own density.
**Notes:** User specified this directly: "0 panes should be allowed... max largely depends on display/terminal width so we should just arbitrarily set it to 10"

---

| Option | Description | Selected |
|--------|-------------|----------|
| Prompt for label on create | User names pane on creation | |
| Auto-label on create (e.g., 'Pane N') | Monotonically increasing counter | ✓ |

**User's choice:** Auto-label

---

| Option | Description | Selected |
|--------|-------------|----------|
| Delete active pane, shift focus to adjacent | Active pane is removed | ✓ |
| Always delete the last pane (rightmost) | Rightmost always deleted | |

**User's choice:** Delete active pane

---

| Option | Description | Selected |
|--------|-------------|----------|
| No confirmation — just delete immediately | Matches existing task delete behavior | ✓ |
| Confirm before deleting a pane | Dialog or status bar prompt | |

**User's choice:** No confirmation

---

| Option | Description | Selected |
|--------|-------------|----------|
| New pane opens at the right with default state | Empty filter, file-order sort | ✓ |
| New pane opens as a clone of active pane's state | Copies filter/sort | |

**User's choice:** Default state, appended to the right

---

## Hotkey Naming and Key Assignments

| Option | Description | Selected |
|--------|-------------|----------|
| Ctrl+N / Ctrl+W / Ctrl+P | Editor-style new/close/toggle. Tmux/screen safe. | ✓ |
| Special chars | Punctuation keys | |
| Shifted variants | Capital letter variants | |
| You decide | Agent picks | |

**User's choice:** Ctrl+N / Ctrl+W / Ctrl+P
**Notes:** User asked to verify tmux/screen safety first; confirmed these are not intercepted by default tmux (Ctrl+B prefix) or screen (Ctrl+A prefix) bindings.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Configurable (register in default_keymap) | Phase 22 pattern, user-remappable | ✓ |
| Hardcoded | Not configurable | |

**User's choice:** Configurable via config.toml

---

## Help/Status Bar Discoverability

| Option | Description | Selected |
|--------|-------------|----------|
| Add pane controls to existing help overlay (?) | New "Panes" section in existing overlay | ✓ |
| Separate pane help page | Dedicated pane help (? then p) | |

**User's choice:** Existing help overlay with a Panes section

---

| Option | Description | Selected |
|--------|-------------|----------|
| Show pane count + hidden state in status bar | Runtime indicator | |
| No pane status in status bar | Visual state is self-evident | ✓ |

**User's choice:** No status bar indicator

---

## the agent's Discretion

- Internal field name for hidden state tracking on App struct
- Auto-label counter strategy (monotonic vs slot-based) — decided in specifics: monotonic
- `reconcile_active_pane` 0-pane safety implementation
- Whether `Pane.id` is reassigned after delete or preserved

## Deferred Ideas

- Pane rename after creation
- Reorder panes interactively (PANE-06, v2)
- Persist hidden state across restarts

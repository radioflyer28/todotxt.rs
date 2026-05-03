# Phase 35: Basic Clipboard Workflows — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Session:** 2026-04-30
**Phase:** 35 — Basic Clipboard Workflows

---

## Area 1: Clipboard Backend

**Q:** Should clipboard operations use the system clipboard (arboard crate) or an internal in-process buffer only?

| Option | Selected |
|--------|----------|
| System clipboard (arboard) | ✅ |
| Internal buffer only | ☐ |

**Decision:** System clipboard via arboard crate. Enables cross-app paste and matches user expectation.

---

## Area 2: Keybindings — copy, cut, paste keys

**Q1:** What keybindings for copy, cut, and paste in Normal mode?

| Option | Selected |
|--------|----------|
| y / Y+D / p (vim-style) | ✅ |
| c / x / p (editor-style) | ☐ |
| Ctrl+C / Ctrl+X / Ctrl+V | ☐ |

**Q2 (follow-up):** What exactly triggers cut?

| Option | Selected |
|--------|----------|
| Y (Shift+y) = cut | ☐ |
| y then d = cut (two keystrokes) | ✅ |
| Other | ☐ |

**Decision:** `y` = copy, `p` = paste (Normal mode). Cut is user workflow of y then d (no dedicated cut mode).

---

## Area 3: Cut Confirmation — preview before removing

**Q:** When the user presses d to complete a cut after y, should there be a confirmation step for bulk selections?

| Option | Selected |
|--------|----------|
| Count preview when N > 1 (same as delete/append) | ✅ |
| No preview — cut applies immediately | ☐ |
| Full DeleteConfirm panel (y/n) | ☐ |

**Decision:** Count preview for bulk delete — reuses existing Phase 34 DeleteConfirm pattern. y and d are independent; delete follows its own rules.

---

## Area 4: Paste Behavior — single vs multi-task

**Q1:** If clipboard contains multiple task lines, how should p paste them?

| Option | Selected |
|--------|----------|
| Paste all clipboard lines at once | ✅ |
| Paste one task per keypress | ☐ |

**Q2:** Where do pasted tasks land?

| Option | Selected |
|--------|----------|
| Append to end of file | ✅ |
| Insert below cursor | ☐ |
| Agent's discretion | ☐ |

**Q3 (CLIP-04):** How should paste work during new-task entry (n)?

| Option | Selected |
|--------|----------|
| Ctrl+V in Adding mode editor | ✅ |
| p key inside Adding mode editor | ☐ |
| Agent's discretion | ☐ |

**Decision:** Paste all lines at once, appended to end of file. Ctrl+V in Adding mode inserts first clipboard line into editor.

---

*Log generated: 2026-04-30*

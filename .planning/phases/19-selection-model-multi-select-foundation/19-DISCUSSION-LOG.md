# Phase 19: Selection Model + Multi-Select Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-24
**Phase:** 19-selection-model-multi-select-foundation
**Areas discussed:** Disjoint selection mode design, Shift-range key bindings, Visual rendering for selected tasks, Selection identity for persistence (SEL-03)

---

## Disjoint Selection Mode Design

| Option | Description | Selected |
|--------|-------------|----------|
| New AppMode::Selecting | A new AppMode variant; nav keys move cursor and optionally toggle selection; Esc exits mode | |
| Flag in Normal mode (Space to mark) | `disjoint_select: bool` flag on App; stays in Normal mode; `Space` toggles the hovered task | ✓ |
| Agent's discretion | Whatever fits the codebase best | |

**User's choice:** Flag in Normal mode with Space to mark

| Option | Description | Selected |
|--------|-------------|----------|
| v (vi visual mode) | `v` key enters/exits disjoint mode (vi visual-line selection mnemonic) | ✓ |
| s (select) | `s` key (mnemonic: select) | |
| Tab | Cycles between normal and select mode | |
| Other | Something else | |

**User's choice:** `v` key (vi visual mode)

| Option | Description | Selected |
|--------|-------------|----------|
| Esc clears selection | Esc exits disjoint mode and clears the entire selection | ✓ |
| Esc exits mode, keeps selection | Esc exits disjoint mode but keeps selected tasks intact | |
| Agent's discretion | | |

**User's choice:** Esc clears selection entirely

---

## Shift-Range Key Bindings

| Option | Description | Selected |
|--------|-------------|----------|
| Shift+j/k only | Only j/k get shift treatment; arrow keys not modified | |
| Shift+arrow only | Only arrow keys extend range; j/k unaffected | |
| Both Shift+j/k and Shift+arrow | Both combos work identically | ✓ |

**User's choice:** Both Shift+j/k AND Shift+Down/Up

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, Shift+Ctrl+D/U extends range | Half-page range selection via Shift+Ctrl+D/U | ✓ |
| No, keep it simple for now | Only single-row shift keys in Phase 19 | |
| Agent's discretion | | |

**User's choice:** Yes — Shift+Ctrl+D/U extends by half-page

---

## Visual Rendering for Selected Tasks

| Option | Description | Selected |
|--------|-------------|----------|
| Bold | Bold modifier only — works in NO_COLOR mode | |
| Prefix glyph (* or ✓) | Checkmark or `*` prefix in the row | |
| Themed highlight color | Background/foreground from style system | |
| Bold + prefix glyph (recommended) | Bold + `>` prefix — clearly distinct from cursor REVERSED; works without color | ✓ |
| Agent's discretion | | |

**User's choice:** Bold + prefix glyph

| Option | Description | Selected |
|--------|-------------|----------|
| > | Vim-style visual selection feel | ✓ |
| * | Classic selected-item marker | |
| ✓ | Checkmark (unicode) | |
| Agent's discretion | | |

**User's choice:** `>` prefix glyph

| Option | Description | Selected |
|--------|-------------|----------|
| REVERSED for cursor, Bold+'>' for selected (recommended) | REVERSED for cursor; Bold+`>` for selected non-cursor; REVERSED+Bold when cursor is on selected | ✓ |
| REVERSED for both, '>' glyph differentiates selected | Both cursor and selected use REVERSED; glyph differentiates | |
| Agent's discretion | | |

**User's choice:** REVERSED for cursor, Bold+`>` for selected rows, REVERSED+Bold when both apply

---

## Selection Identity for Persistence (SEL-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Canonical file index (usize) | Fast, matches display_indices pattern; risk: external line-shift | ✓ |
| Task raw text (String) | More robust to external edits; costs a scan per rebuild | |
| Agent's discretion | | |

**User's choice:** Canonical file index (usize)

| Option | Description | Selected |
|--------|-------------|----------|
| Keep valid indices, drop gone ones (recommended) | On reload, retain indices still in task list; silently drop out-of-range ones | ✓ |
| Clear selection on external reload | Always clear on FileChanged | |
| Agent's discretion | | |

**User's choice:** Keep valid indices, drop indices that no longer exist

---

## Agent's Discretion

- Exact field names for new `App` struct fields
- Whether `Space` also advances cursor after marking
- Minimal status bar selection count indicator (Phase 20 owns polish; planner decides if a minimal count is added in Phase 19)

## Deferred Ideas

- Full status bar selection count/mode polish → Phase 20 (BULK-03)
- Shift+G / Shift+gg select-to-end range → not discussed
- Clipboard copy of selected tasks → v2 backlog (BULK-06)

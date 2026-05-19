---
created: 2026-05-15T00:00:00
title: Refine auto-select behavior for match-driven dialogs
area: tui
resolves_phase: 50
files:
  - Client/Controls/SetDueDateDialog.xaml.cs
  - Client/Controls/MainWindow.xaml
---

## Problem

Match-driven dialogs that present project/context tags or tokens should auto-select the
most relevant existing value or suggestion instead of making the user re-navigate manually.
This is separate from date entry: dates should stay on the date-specific picker rather than
reusing generic auto-select behavior.

## Solution

### 1. Auto-select the most relevant token or suggestion in match-driven dialogs

When the user opens a dialog that presents project/context matches or tokens, the dialog
should focus the current best match instead of leaving selection in an arbitrary place.

### 2. Prefer continuity over reset

If a current token/value already exists, select that first. Otherwise, focus the best
default starting point so the user can keep moving immediately.

### 3. Keep behavior predictable

The refinement should reduce keystrokes without making focus movement feel magical or
hard to follow, and it should not change the date-picker-specific workflow for dates.

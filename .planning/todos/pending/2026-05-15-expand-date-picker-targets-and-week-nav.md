---
created: 2026-05-15T00:00:00
title: Expand date picker targets and add week-jump navigation
area: tui
resolves_phase: 50
files:
  - Client/Controls/SetDueDateDialog.xaml.cs
  - Client/Controls/MainWindow.xaml
---

## Problem

The current date setter flow is too narrow and repetitive for real use. To reach the
desired target, the user keeps pressing `s` to cycle between due, recurring, threshold,
and completed modes, and calendar navigation is slower than it needs to be for dates
that are several days away.

## Solution

### 1. Expand the date setter workflow beyond due dates

Allow the same picker surface to target:

- due date
- threshold date
- completed date

`rec:` rule editing is not part of this date-picker expansion. This todo is about real
date-bearing fields only.

The active target should be visible in the dialog so the user always knows which field
will be changed when they confirm.

### 2. Make left/right arrow navigation jump by week

Update picker navigation so horizontal movement advances or rewinds by 7 days. This makes
calendar traversal much faster while keeping up/down available for day-level movement if
that pattern already exists.

### 3. Preserve existing confirmation/cancel behavior

The expanded workflow should still feel like the existing date setter rather than a new
control with a different mental model.

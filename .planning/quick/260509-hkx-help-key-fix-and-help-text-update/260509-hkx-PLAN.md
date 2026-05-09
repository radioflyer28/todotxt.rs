---
quick_id: 260509-hkx
slug: help-key-fix-and-help-text-update
date: 2026-05-09
description: Fix '?' help key not opening help menu and update help text to match current features
mode: quick-full
must_haves:
  truths:
    - "Pressing '?' when task list has focus opens the help dialog"
    - "Pressing '?' while taskText is focused types '?' instead of opening help"
    - "Help text includes S (set due date), Ctrl+S (set threshold date), Ctrl+T, Ctrl+H shortcuts"
    - "Help text shows Ctrl+Alt+M for hide/unhide window (not Win+Alt+T)"
    - "Help text includes threshold date syntax section"
    - "Help text corrects 'casesensistivity' typo to 'case sensitivity'"
  artifacts:
    - path: "Client/Controls/MainWindow.xaml"
      provides: "PreviewKeyDown handler wired to Window element"
      contains: "Window_PreviewKeyDown"
    - path: "Client/Controls/MainWindow.xaml.cs"
      provides: "Window_PreviewKeyDown method that opens help when Shift+? pressed outside taskText"
      contains: "Key.OemQuestion"
    - path: "Client/Resource.resx"
      provides: "Updated HelpText with all current shortcuts"
      contains: "Ctrl+Alt+M"
---

## Goal

Fix the `?` help key in the WPF client and update the help text to reflect features added since approximately v1.2.

## Root Cause

The `?` key binding (`Shift+OemQuestion`) for `HelpAbout` was in `ListBox.InputBindings` only. It only fires when `lbTasks` has keyboard focus. After various operations (task add/edit, filter dialogs, etc.), focus can shift to `taskText` or other elements, leaving `?` non-functional.

## Plan

### Task 1: Fix `?` key binding — add Window-level PreviewKeyDown handler

**Files**: `Client/Controls/MainWindow.xaml`, `Client/Controls/MainWindow.xaml.cs`

Add `PreviewKeyDown="Window_PreviewKeyDown"` to the `<Window>` element.

In the code-behind, add a handler that:
- Fires when `Shift+?` is pressed anywhere in the window
- Checks `!taskText.IsFocused` to preserve typing `?` in the text box
- Calls `ViewModel.ShowHelpDialog()` and marks the event handled

This uses `PreviewKeyDown` (tunnel phase) so it intercepts before any child element, preventing `?` from being typed in text boxes when help should open.

### Task 2: Update help text in Resource.resx

Add missing shortcuts:
- `S`: set due date (was undocumented)
- `Ctrl+S`: set threshold date (was undocumented)
- `Ctrl+Alt+P`: postpone threshold date (was undocumented)
- `Ctrl+T`: toggle future task filter (was undocumented)
- `Ctrl+H`: toggle hidden task visibility (was undocumented)
- `Ctrl+Left/Right`: remove threshold date (was missing, Ctrl+Up/Down were listed but not remove)

Fix errors:
- `Win+Alt+T` → `Ctrl+Alt+M` (wrong hotkey, code uses `Ctrl+Alt+M`)
- `casesensistivity` → `case sensitivity` (spelling fix)

Add new section:
- Threshold date syntax (`t:YYYY-MM-DD`, `t:today`, `t:tomorrow`)

Update descriptions:
- `Ctrl+4: Due Date` (was DueDate)
- `Ctrl+5: Creation Date` (was Created)
- `Ctrl+0: None (file order)` (added clarification)

---
quick_id: 260509-hkx
slug: help-key-fix-and-help-text-update
date: 2026-05-09
status: complete
commit: e858627
---

## Summary

Fixed the `?` help key not opening the help menu, and updated help text to reflect all features added since v1.2.

## Changes Made

### Fix 1: `?` key binding restored (Client/Controls/MainWindow.xaml + .cs)

**Root cause**: The `?` (Shift+OemQuestion) key binding was in `ListBox.InputBindings` only — it only fired when `lbTasks` had focus. After operations like adding tasks, filter dialogs, or UpdateDisplayedTasks with grouping enabled, focus shifts away from `lbTasks`, rendering `?` non-functional.

**Fix**: Added `PreviewKeyDown="Window_PreviewKeyDown"` to the Window element and a handler in the code-behind:
```csharp
private void Window_PreviewKeyDown(object sender, KeyEventArgs e)
{
    if (e.Key == Key.OemQuestion && Keyboard.Modifiers == ModifierKeys.Shift && !taskText.IsFocused)
    {
        ViewModel.ShowHelpDialog();
        e.Handled = true;
    }
}
```

Using `PreviewKeyDown` (tunnel phase) ensures the handler fires before any child element intercepts the key. The `!taskText.IsFocused` check preserves the ability to type `?` in the task text box.

### Fix 2: Help text updated (Client/Resource.resx)

Added 6 previously undocumented shortcuts:
- `S` — set due date
- `Ctrl+S` — set threshold date
- `Ctrl+Alt+P` — postpone threshold date
- `Ctrl+T` — toggle hiding future tasks
- `Ctrl+H` — toggle showing hidden tasks (h:1)
- `Ctrl+Left/Right` — remove threshold date

Fixed factual error:
- `Win+Alt+T` → `Ctrl+Alt+M` (the actual tray hotkey registered in HotKeyMainWindows.cs)

Added new section:
- Threshold date syntax (`t:YYYY-MM-DD`, `t:today`, `t:tomorrow`)

Fixed typo and improved formatting:
- `casesensistivity` → `case sensitivity`
- `DueDate` → `Due Date`, `Created` → `Creation Date`

## Verification

- No compile errors in modified files
- `Window_PreviewKeyDown` correctly placed in `#region help menu`
- `Resource.Designer.cs` auto-regenerates from Resource.resx — no manual changes needed
- Existing ListBox.InputBindings `?` binding remains as fallback (harmless redundancy)
